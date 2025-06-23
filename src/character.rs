//! Character system for the miniature-potato game

use bevy::prelude::*;
use geotiles::Point;
use crate::geotiles_bevy::HexasphereResource;

/// Component to mark the character entity
#[derive(Component)]
pub struct Character {
    pub current_tile: usize,
    pub hover_height: f32,
}

/// Resource to store character-related data
#[derive(Resource)]
pub struct CharacterResource {
    pub entity: Option<Entity>,
}

fn vec3_from_point(p: &Point) -> Vec3 {
    Vec3::new(p.x as f32, p.y as f32, p.z as f32)
}

/// Find the tile that is most facing the camera (with 90-degree rotation)
fn find_camera_facing_tile(hexasphere: &geotiles::Hexasphere) -> usize {
    // After 90-degree yaw rotation, the camera is effectively looking from a different direction
    // With yaw = π/2, the camera view is rotated 90 degrees around Y axis
    // Original camera direction was looking towards -Z, now it's looking towards -X
    // So we want the tile that faces towards +X direction (towards the rotated camera)
    let desired_normal = Vec3::X; // After 90-degree rotation, camera sees +X face
    
    let mut best_tile_index = 0;
    let mut best_dot_product = -2.0; // Start below -1
    
    for (index, tile) in hexasphere.tiles.iter().enumerate() {
        let tile_center = vec3_from_point(&tile.center_point);
        let tile_normal = tile_center.normalize(); // Normal pointing outward from sphere center
        
        // Calculate how well this tile's normal aligns with the desired direction
        // Higher dot product means the tile is more facing the camera
        let dot_product = tile_normal.dot(desired_normal);
        
        if dot_product > best_dot_product {
            best_dot_product = dot_product;
            best_tile_index = index;
        }
    }
    
    println!("🎯 With 90° rotation, looking for tile facing +X direction");
    println!("🎯 Best camera-facing tile: #{} with alignment {:.3}", best_tile_index, best_dot_product);
    println!("🎯 Selected tile center: {:?}", vec3_from_point(&hexasphere.tiles[best_tile_index].center_point));
    
    best_tile_index
}

/// Setup the character as a blue sphere hovering over the camera-facing tile
pub fn setup_character(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    hexasphere_res: Option<Res<HexasphereResource>>,
    mut character_res: ResMut<CharacterResource>,
) {
    if let Some(hexasphere) = hexasphere_res {
        // Find the tile most facing the camera
        let camera_facing_tile_index = find_camera_facing_tile(&hexasphere.hexasphere);
        
        if let Some(tile) = hexasphere.hexasphere.tiles.get(camera_facing_tile_index) {
            let tile_center = vec3_from_point(&tile.center_point);
            let hover_height = 0.3; // Height above the tile surface
            
            // Position character above tile center, slightly outward from sphere center
            let character_position = tile_center + tile_center.normalize() * hover_height;
            
            let tile_type = if tile.boundary.len() == 5 { "pentagon" } else { "hexagon" };
            println!("🧍 Spawning character at camera-facing {} tile #{}: {:?}", 
                     tile_type, camera_facing_tile_index, character_position);
            
            // Create blue sphere for character
            let character_entity = commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.1))), // Small blue sphere
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.2, 0.5, 1.0), // Blue color
                    metallic: 0.0,
                    perceptual_roughness: 0.3,
                    ..default()
                })),
                Transform::from_translation(character_position),
                Character {
                    current_tile: camera_facing_tile_index,
                    hover_height,
                },
            )).id();
            
            // Store character entity in resource
            character_res.entity = Some(character_entity);
            
            println!("✅ Character setup complete!");
        } else {
            println!("⚠️ Could not find camera-facing tile to position character");
        }
    } else {
        println!("⚠️ Hexasphere not ready for character placement");
    }
}

/// Find the neighbor tile that is closest to the desired world direction
fn find_neighbor_in_direction(
    current_tile: &geotiles::Tile,
    desired_direction: Vec3,
    hexasphere: &geotiles::Hexasphere,
) -> Option<usize> {
    let current_center = vec3_from_point(&current_tile.center_point);
    let mut best_neighbor = None;
    let mut best_dot_product = -2.0; // Start below -1 to ensure we find something
    
    for &neighbor_index in &current_tile.neighbors {
        if let Some(neighbor_tile) = hexasphere.tiles.get(neighbor_index) {
            // Skip pentagons for now
            if neighbor_tile.boundary.len() == 5 {
                continue;
            }
            
            let neighbor_center = vec3_from_point(&neighbor_tile.center_point);
            
            // Calculate direction from current tile to neighbor (on sphere surface)
            let to_neighbor = neighbor_center - current_center;
            let to_neighbor_normalized = to_neighbor.normalize();
            
            // Calculate how well this direction matches our desired direction
            let dot_product = to_neighbor_normalized.dot(desired_direction);
            
            if dot_product > best_dot_product {
                best_dot_product = dot_product;
                best_neighbor = Some(neighbor_index);
            }
        }
    }
    
    best_neighbor
}

/// System to handle character movement using simplified directional controls
/// With locked camera orientation: A/D = left/right, W/X = one diagonal, E/Z = other diagonal
pub fn handle_character_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    hexasphere_res: Option<Res<HexasphereResource>>,
    mut character_query: Query<(&mut Character, &mut Transform)>,
) {
    if let Some(hexasphere) = hexasphere_res {
        if let Ok((mut character, mut transform)) = character_query.single_mut() {
            let current_tile_index = character.current_tile;
            
            // Get current tile
            if let Some(current_tile) = hexasphere.hexasphere.tiles.get(current_tile_index) {
                let current_center = vec3_from_point(&current_tile.center_point);
                
                // With locked camera orientation (90 degrees rotated), adjust directions
                // The sphere is rotated 90 degrees, so we need to rotate our direction vectors too
                let mut desired_direction: Option<Vec3> = None;
                
                if keyboard.just_pressed(KeyCode::KeyD) {
                    // Left: towards negative Z (swapped from A)
                    desired_direction = Some(-Vec3::Z);
                } else if keyboard.just_pressed(KeyCode::KeyA) {
                    // Right: towards positive Z (swapped from D)
                    desired_direction = Some(Vec3::Z);
                } else if keyboard.just_pressed(KeyCode::KeyE) {
                    // Up-left diagonal: towards +Y-Z (swapped from W)
                    desired_direction = Some((Vec3::Y - Vec3::Z).normalize());
                } else if keyboard.just_pressed(KeyCode::KeyZ) {
                    // Down-right diagonal: towards -Y+Z (swapped from X)
                    desired_direction = Some((-Vec3::Y + Vec3::Z).normalize());
                } else if keyboard.just_pressed(KeyCode::KeyW) {
                    // Up-right diagonal: towards +Y+Z (swapped from E)
                    desired_direction = Some((Vec3::Y + Vec3::Z).normalize());
                } else if keyboard.just_pressed(KeyCode::KeyX) {
                    // Down-left diagonal: towards -Y-Z (swapped from Z)
                    desired_direction = Some((-Vec3::Y - Vec3::Z).normalize());
                }
                
                if let Some(direction) = desired_direction {
                    // Project the desired direction onto the sphere's tangent plane at current position
                    // This ensures we move along the sphere surface rather than through it
                    let normal = current_center.normalize(); // Surface normal at current position
                    let tangent_direction = (direction - normal * direction.dot(normal)).normalize();
                    
                    // Find the neighbor that best matches this direction
                    if let Some(target_tile_index) = find_neighbor_in_direction(current_tile, tangent_direction, &hexasphere.hexasphere) {
                        if let Some(target_tile) = hexasphere.hexasphere.tiles.get(target_tile_index) {
                            let target_center = vec3_from_point(&target_tile.center_point);
                            let new_position = target_center + target_center.normalize() * character.hover_height;
                            
                            // Update character position and current tile
                            transform.translation = new_position;
                            character.current_tile = target_tile_index;
                            
                            println!("🚶 Character moved to hexagon tile #{}", target_tile_index);
                        }
                    } else {
                        println!("🚫 No valid hexagon neighbor found in that direction");
                    }
                }
            }
        }
    }
}