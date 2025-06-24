//! Character system for the miniature-potato game

use bevy::prelude::*;
use geotiles::Point;
use crate::geotiles_bevy::HexasphereResource;
use std::f32::consts::PI;

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
    pub last_rotation_time: f32,
}

fn vec3_from_point(p: &Point) -> Vec3 {
    Vec3::new(p.x as f32, p.y as f32, p.z as f32)
}

/// Find the tile that is most facing the camera
fn find_camera_facing_tile(hexasphere: &geotiles::Hexasphere) -> usize {
    // Camera is looking towards -Z direction from +Z position
    // So we want the tile that faces towards +Z direction (towards the camera)
    let desired_normal = Vec3::Z; // Camera sees +Z face
    
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
    
    println!("🎯 Looking for tile facing +Z direction (towards camera)");
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
                
                // With fixed camera, use standard world directions
                let mut desired_direction: Option<Vec3> = None;
                
                if keyboard.just_pressed(KeyCode::KeyD) {
                    // Left: towards negative X
                    desired_direction = Some(-Vec3::X);
                } else if keyboard.just_pressed(KeyCode::KeyA) {
                    // Right: towards positive X
                    desired_direction = Some(Vec3::X);
                } else if keyboard.just_pressed(KeyCode::KeyE) {
                    // Up-left diagonal: towards +Y-X
                    desired_direction = Some((Vec3::Y - Vec3::X).normalize());
                } else if keyboard.just_pressed(KeyCode::KeyZ) {
                    // Down-right diagonal: towards -Y+X
                    desired_direction = Some((-Vec3::Y + Vec3::X).normalize());
                } else if keyboard.just_pressed(KeyCode::KeyW) {
                    // Up-right diagonal: towards +Y+X
                    desired_direction = Some((Vec3::Y + Vec3::X).normalize());
                } else if keyboard.just_pressed(KeyCode::KeyX) {
                    // Down-left diagonal: towards -Y-X
                    desired_direction = Some((-Vec3::Y - Vec3::X).normalize());
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

/// System to rotate the sphere to keep the character centered and maintain hexagon orientation
pub fn follow_character_with_sphere_rotation(
    character_query: Query<(&Character, &Transform)>,
    mut sphere_parent_query: Query<&mut Transform, (With<crate::geotiles_bevy::SphereParent>, Without<Character>)>,
    mut character_res: ResMut<CharacterResource>,
    time: Res<Time>,
) {
    if let (Ok((_character, character_transform)), Ok(mut sphere_transform)) = 
        (character_query.single(), sphere_parent_query.single_mut()) {
        
        let character_position = character_transform.translation;
        let current_time = time.elapsed_secs();
        
        // Add cooldown to prevent rapid successive rotations
        let rotation_cooldown = 2.0; // 2 seconds between rotations
        if current_time - character_res.last_rotation_time < rotation_cooldown {
            return;
        }
        
        // Get current sphere rotation (Y-axis rotation)
        let current_yaw = sphere_transform.rotation.to_euler(EulerRot::YXZ).0;
        
        // Check if the character has moved far enough from center to warrant rotation
        if should_rotate_sphere(character_position, current_yaw) {
            println!("🎯 Character at edge, checking for sphere rotation");
            
            // Calculate the ideal yaw to center the character
            if let Some(new_yaw) = calculate_ideal_yaw(character_position, current_yaw) {
                println!("🔄 Rotating sphere from {:.1}° to {:.1}° to follow character", 
                         current_yaw.to_degrees(), new_yaw.to_degrees());
                
                // Apply rotation to the sphere parent entity
                sphere_transform.rotation = Quat::from_rotation_y(new_yaw);
                character_res.last_rotation_time = current_time;
            }
        }
    }
}

/// Check if sphere should rotate based on character position
fn should_rotate_sphere(character_position: Vec3, current_sphere_yaw: f32) -> bool {
    // Apply current sphere rotation to get the character's position relative to camera view
    // Since sphere rotates around Y-axis, we need to counter-rotate to get view-relative position
    let view_relative_position = rotate_point_around_y(-current_sphere_yaw, character_position);
    
    // Check if character is too far from the center of the view (X axis in view space)
    // The camera looks down -Z, so X determines left/right position
    let distance_from_center = view_relative_position.x.abs();
    let forward_distance = view_relative_position.z.abs();
    
    // Trigger rotation if character is more than 70% of the way to the edge
    distance_from_center > forward_distance * 0.7
}

/// Calculate the ideal sphere rotation to center the character
fn calculate_ideal_yaw(character_position: Vec3, current_sphere_yaw: f32) -> Option<f32> {
    // Apply current sphere rotation to get the character's position relative to camera view
    let view_relative_position = rotate_point_around_y(-current_sphere_yaw, character_position);
    
    // Determine which direction to rotate the sphere based on character's X position
    // If character is to the right (positive X), we need to rotate sphere clockwise (positive yaw)
    // to bring the character back to center
    let increment = PI / 12.0; // 15 degrees
    let new_sphere_yaw = if view_relative_position.x > 0.0 {
        // Character is to the right, rotate sphere clockwise (positive yaw)
        current_sphere_yaw + increment
    } else {
        // Character is to the left, rotate sphere counter-clockwise (negative yaw)
        current_sphere_yaw - increment
    };
    
    // Normalize to 0-2π range
    let normalized_yaw = ((new_sphere_yaw % (2.0 * PI)) + 2.0 * PI) % (2.0 * PI);
    
    Some(normalized_yaw)
}

/// Rotate a point around the Y axis by the given angle
fn rotate_point_around_y(angle: f32, point: Vec3) -> Vec3 {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    
    Vec3::new(
        point.x * cos_a - point.z * sin_a,
        point.y,
        point.x * sin_a + point.z * cos_a,
    )
}

