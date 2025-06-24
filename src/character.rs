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
            
            // Create blue sphere for character as child of world parent
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
                ChildOf(hexasphere.world_parent),
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
    world_transform: &Transform,
) -> Option<usize> {
    // Transform tile centers to world space to account for world rotation
    let local_current_center = vec3_from_point(&current_tile.center_point);
    let current_center = world_transform.transform_point(local_current_center);
    let mut best_neighbor = None;
    let mut best_dot_product = -2.0; // Start below -1 to ensure we find something
    
    for &neighbor_index in &current_tile.neighbors {
        if let Some(neighbor_tile) = hexasphere.tiles.get(neighbor_index) {
            let local_neighbor_center = vec3_from_point(&neighbor_tile.center_point);
            let neighbor_center = world_transform.transform_point(local_neighbor_center);
            
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

/// System to handle character movement using camera-relative WASD controls
pub fn handle_character_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    hexasphere_res: Option<Res<HexasphereResource>>,
    world_parent_query: Query<&Transform, (With<crate::geotiles_bevy::WorldParent>, Without<Character>)>,
    mut character_query: Query<(&mut Character, &mut Transform)>,
) {
    if let Some(hexasphere) = hexasphere_res {
        if let (Ok((mut character, mut transform)), Ok(world_transform)) = 
            (character_query.single_mut(), world_parent_query.single()) {
            
            let current_tile_index = character.current_tile;
            
            // Get current tile
            if let Some(current_tile) = hexasphere.hexasphere.tiles.get(current_tile_index) {
                // Get tile center and transform it by world rotation to get actual world position
                let local_center = vec3_from_point(&current_tile.center_point);
                let current_center = world_transform.transform_point(local_center);
                
                // Define movement directions in camera space (camera looks down -Z)
                let mut camera_relative_direction: Option<Vec3> = None;
                
                if keyboard.just_pressed(KeyCode::KeyW) {
                    // Forward (away from camera)
                    camera_relative_direction = Some(-Vec3::Z);
                } else if keyboard.just_pressed(KeyCode::KeyS) {
                    // Backward (toward camera)
                    camera_relative_direction = Some(Vec3::Z);
                } else if keyboard.just_pressed(KeyCode::KeyA) {
                    // Left
                    camera_relative_direction = Some(-Vec3::X);
                } else if keyboard.just_pressed(KeyCode::KeyD) {
                    // Right
                    camera_relative_direction = Some(Vec3::X);
                }
                
                if let Some(camera_direction) = camera_relative_direction {
                    // Use camera direction directly in world space - don't apply world rotation
                    // This keeps WASD movement consistent relative to camera view regardless of world rotation
                    let world_direction = camera_direction;
                    
                    // Project the desired direction onto the sphere's tangent plane at current position
                    let normal = current_center.normalize(); // Surface normal at current position
                    let tangent_direction = (world_direction - normal * world_direction.dot(normal)).normalize();
                    
                    // Find the neighbor that best matches this direction
                    if let Some(target_tile_index) = find_neighbor_in_direction(current_tile, tangent_direction, &hexasphere.hexasphere, world_transform) {
                        if let Some(target_tile) = hexasphere.hexasphere.tiles.get(target_tile_index) {
                            // Transform target tile center to world space
                            let local_target_center = vec3_from_point(&target_tile.center_point);
                            let target_center = world_transform.transform_point(local_target_center);
                            let new_position = target_center + target_center.normalize() * character.hover_height;
                            
                            // Convert world position back to local space relative to world parent
                            // Create inverse transform manually
                            let inv_rotation = world_transform.rotation.inverse();
                            let inv_translation = inv_rotation * (-world_transform.translation);
                            let local_position = inv_rotation * new_position + inv_translation;
                            
                            // Update character position and current tile
                            transform.translation = local_position;
                            character.current_tile = target_tile_index;
                            
                            let tile_type = if target_tile.boundary.len() == 5 { "pentagon" } else { "hexagon" };
                            println!("🚶 Character moved to {} tile #{}", tile_type, target_tile_index);
                        }
                    } else {
                        println!("🚫 No valid neighbor found in that direction");
                    }
                }
            }
        }
    }
}

/// System to rotate the world to keep the character centered and maintain hexagon orientation
pub fn follow_character_with_sphere_rotation(
    character_query: Query<(&Character, &Transform)>,
    mut world_parent_query: Query<&mut Transform, (With<crate::geotiles_bevy::WorldParent>, Without<Character>)>,
    mut character_res: ResMut<CharacterResource>,
    time: Res<Time>,
) {
    if let (Ok((_character, character_transform)), Ok(mut world_transform)) = 
        (character_query.single(), world_parent_query.single_mut()) {
        
        let character_position = character_transform.translation;
        let current_time = time.elapsed_secs();
        
        // Add cooldown to prevent rapid successive rotations
        let rotation_cooldown = 2.0; // 2 seconds between rotations
        if current_time - character_res.last_rotation_time < rotation_cooldown {
            return;
        }
        
        // Get current world rotation (Y-axis rotation)
        let current_yaw = world_transform.rotation.to_euler(EulerRot::YXZ).0;
        
        // Check if the character has moved far enough from center to warrant rotation
        if should_rotate_sphere(character_position, current_yaw) {
            println!("🎯 Character at edge, checking for world rotation");
            
            // Calculate the ideal yaw to center the character
            if let Some(new_yaw) = calculate_ideal_yaw(character_position, current_yaw) {
                println!("🔄 Rotating world from {:.1}° to {:.1}° to follow character", 
                         current_yaw.to_degrees(), new_yaw.to_degrees());
                
                // Apply rotation to the world parent entity (both sphere and character will rotate)
                world_transform.rotation = Quat::from_rotation_y(new_yaw);
                character_res.last_rotation_time = current_time;
            }
        }
    }
}

/// Check if world should rotate based on character position
fn should_rotate_sphere(character_position: Vec3, current_world_yaw: f32) -> bool {
    // Apply current world rotation to get the character's position relative to camera view
    // Since world rotates around Y-axis, we need to counter-rotate to get view-relative position
    let view_relative_position = rotate_point_around_y(-current_world_yaw, character_position);
    
    // Check if character is too far from the center of the view (X axis in view space)
    // The camera looks down -Z, so X determines left/right position
    let distance_from_center = view_relative_position.x.abs();
    let forward_distance = view_relative_position.z.abs();
    
    // Trigger rotation if character is more than 70% of the way to the edge
    distance_from_center > forward_distance * 0.7
}

/// Calculate the ideal world rotation to center the character
fn calculate_ideal_yaw(character_position: Vec3, current_world_yaw: f32) -> Option<f32> {
    // Apply current world rotation to get the character's position relative to camera view
    let view_relative_position = rotate_point_around_y(-current_world_yaw, character_position);
    
    // Determine which direction to rotate the world based on character's X position
    // If character is to the right (positive X), we need to rotate world LEFT (negative yaw)
    // to bring the character back to center
    let increment = PI / 12.0; // 15 degrees
    let new_world_yaw = if view_relative_position.x > 0.0 {
        // Character is to the right, rotate world LEFT (negative yaw)
        current_world_yaw - increment
    } else {
        // Character is to the left, rotate world RIGHT (positive yaw)
        current_world_yaw + increment
    };
    
    // Normalize to 0-2π range
    let normalized_yaw = ((new_world_yaw % (2.0 * PI)) + 2.0 * PI) % (2.0 * PI);
    
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

