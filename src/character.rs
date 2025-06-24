//! Character system for the miniature-potato game

use bevy::{color::palettes::css::{BLUE, DARK_CYAN, MAGENTA, PINK, RED}, prelude::*};
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
}

/// Resource to control debug gizmo visibility
#[derive(Resource, Default)]
pub struct DebugGizmosResource {
    pub show_axes: bool,
    pub show_dead_zone: bool,
    pub character_outside_dead_zone: bool, // Track current state for consistent gizmo coloring
}

pub fn vec3_from_point(p: &Point) -> Vec3 {
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
pub fn find_neighbor_in_direction(
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
                    // Up (toward top of screen/sphere)
                    camera_relative_direction = Some(Vec3::Y);
                } else if keyboard.just_pressed(KeyCode::KeyS) {
                    // Down (toward bottom of screen/sphere)
                    camera_relative_direction = Some(-Vec3::Y);
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

/// System to update dead zone state for gizmo coloring (runs before gizmo drawing)
pub fn update_dead_zone_state(
    character_query: Query<(&Character, &Transform)>,
    world_parent_query: Query<&Transform, (With<crate::geotiles_bevy::WorldParent>, Without<Character>)>,
    mut debug_gizmos: ResMut<DebugGizmosResource>,
) {
    if let (Ok((_character, character_transform)), Ok(world_transform)) = 
        (character_query.single(), world_parent_query.single()) {
        
        let character_position = character_transform.translation;
        
        // // Check if the character is outside the dead zone
        // let is_outside = should_rotate_sphere_3d(character_position, world_transform);
        
        // // Update the debug state for consistent gizmo coloring
        // debug_gizmos.character_outside_dead_zone = is_outside;
    }
}





/// Draw the dead zone as a circle in screen space with state-based coloring
fn draw_dead_zone_gizmo(
    gizmos: &mut Gizmos, 
    debug_gizmos: &DebugGizmosResource
) {
    // Draw a circle on the sphere surface to show the dead zone
    // Position it at the front face of the sphere (facing camera)
    let sphere_radius = 5.0; // Same as SPHERE_RADIUS in geotiles_bevy.rs
    let dead_zone_degrees: f32 = 20.0;
    let dead_zone_radius = sphere_radius * (dead_zone_degrees.to_radians()).sin();
    
    // Use the stored state from the rotation system for consistent coloring
    let circle_color = if debug_gizmos.character_outside_dead_zone {
        BLUE // Blue when character is outside dead zone
    } else {
        MAGENTA // Magenta when character is inside dead zone
    };

    gizmos.circle(Isometry3d::from_xyz(0.0, 0.0, sphere_radius), dead_zone_radius, circle_color);
}

/// Draw debug ray from camera to character
fn draw_camera_ray_gizmo(
    gizmos: &mut Gizmos,
    character_position: Vec3,
    world_transform: &Transform,
) {
    // Camera position in world space (always at origin)
    let camera_pos = Vec3::new(0.0, 0.0, 15.0);
    
    // Character position is already in local space, transform to world space
    let world_character_pos = world_transform.transform_point(character_position);
    
    // Draw larger sphere at camera position for visibility
    gizmos.sphere(camera_pos, 0.3, Color::srgb(1.0, 1.0, 1.0)); // Large white camera dot

    // Draw arrow from camera to character
    gizmos.arrow(camera_pos, world_character_pos, DARK_CYAN);
}

/// System to draw debug gizmos for sphere and character coordinate systems
pub fn debug_gizmos_system(
    mut gizmos: Gizmos,
    debug_gizmos: Res<DebugGizmosResource>,
    world_parent_query: Query<&Transform, (With<crate::geotiles_bevy::WorldParent>, Without<Character>)>,
    character_query: Query<(&Character, &Transform), With<Character>>,
) {
    // Get character and world transforms
    if let (Ok(world_transform), Ok((_character, character_transform))) = 
        (world_parent_query.single(), character_query.single()) {
        
        let character_position = character_transform.translation;
        
        // Draw dead zone circle if enabled (with state-based coloring)
        if debug_gizmos.show_dead_zone {
            draw_dead_zone_gizmo(&mut gizmos, &debug_gizmos);
        }
        
        // Draw camera ray if enabled
        if debug_gizmos.show_dead_zone { // Use same toggle for now
            draw_camera_ray_gizmo(&mut gizmos, character_position, world_transform);
        }
    }
    
    if !debug_gizmos.show_axes {
        return;
    }
    // Draw world/sphere axes (larger scale so we can see them outside the sphere)
    if let Ok(world_transform) = world_parent_query.single() {
        let sphere_center = world_transform.translation;
        let sphere_rotation = world_transform.rotation;
        
        // Draw axes for the world/sphere coordinate system with larger scale
        let axis_length = 8.0; // Much larger than sphere radius (5.0)
        
        // Transform standard axes by the world rotation
        let x_axis = sphere_rotation * Vec3::X * axis_length;
        let y_axis = sphere_rotation * Vec3::Y * axis_length;
        let z_axis = sphere_rotation * Vec3::Z * axis_length;
        
        // Draw sphere axes - Red=X, Green=Y, Blue=Z with bold lines
        gizmos.line(sphere_center, sphere_center + x_axis, Color::srgb(1.0, 0.0, 0.0)); // Red X
        gizmos.line(sphere_center, sphere_center + y_axis, Color::srgb(0.0, 1.0, 0.0)); // Green Y
        gizmos.line(sphere_center, sphere_center + z_axis, Color::srgb(0.0, 0.0, 1.0)); // Blue Z
        
        // Add axis labels using small spheres
        gizmos.sphere(sphere_center + x_axis, 0.2, Color::srgb(1.0, 0.0, 0.0)); // Red X end
        gizmos.sphere(sphere_center + y_axis, 0.2, Color::srgb(0.0, 1.0, 0.0)); // Green Y end
        gizmos.sphere(sphere_center + z_axis, 0.2, Color::srgb(0.0, 0.0, 1.0)); // Blue Z end
    }
    
    // Draw character axes (smaller scale, local to character) if we have the transforms
    if let (Ok(_world_transform), Ok((_character, character_transform))) = 
        (world_parent_query.single(), character_query.single()) {
        

        gizmos.axes(*character_transform, 0.75);
    }
    
    // Draw fixed camera/world coordinate system for reference (always at origin)
    let camera_axis_length = 6.0;
    let origin = Vec3::ZERO;
    
    // Draw fixed world axes - Bright colors, always pointing in world directions with bold lines
    gizmos.line_gradient(origin, origin + Vec3::X * camera_axis_length, Color::srgb(1.0, 0.0, 1.0), Color::srgb(1.0, 0.0, 1.0)); // Magenta X (camera right)
    gizmos.line_gradient(origin, origin + Vec3::Y * camera_axis_length, Color::srgb(1.0, 1.0, 0.0), Color::srgb(1.0, 1.0, 0.0)); // Yellow Y (camera up)
    gizmos.line_gradient(origin, origin + Vec3::Z * camera_axis_length, Color::srgb(0.0, 1.0, 1.0), Color::srgb(0.0, 1.0, 1.0)); // Cyan Z (camera forward)
    
    // Add markers at the end of camera axes
    gizmos.sphere(origin + Vec3::X * camera_axis_length, 0.15, Color::srgb(1.0, 0.0, 1.0)); // Magenta X end
    gizmos.sphere(origin + Vec3::Y * camera_axis_length, 0.15, Color::srgb(1.0, 1.0, 0.0)); // Yellow Y end
    gizmos.sphere(origin + Vec3::Z * camera_axis_length, 0.15, Color::srgb(0.0, 1.0, 1.0)); // Cyan Z end
}

/// System to toggle debug gizmos with 'G' key
pub fn toggle_debug_gizmos(
    mut debug_gizmos: ResMut<DebugGizmosResource>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyG) {
        // Toggle both axes and dead zone together
        debug_gizmos.show_axes = !debug_gizmos.show_axes;
        debug_gizmos.show_dead_zone = debug_gizmos.show_axes; // Same state as axes
        println!(
            "Debug gizmos: {} (axes: {}, dead zone: {})",
            if debug_gizmos.show_axes { "ON" } else { "OFF" },
            debug_gizmos.show_axes,
            debug_gizmos.show_dead_zone
        );
    }
}

