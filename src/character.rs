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
        
        // Check if the character is outside the dead zone
        let is_outside = should_rotate_sphere_3d(character_position, world_transform);
        
        // Update the debug state for consistent gizmo coloring
        debug_gizmos.character_outside_dead_zone = is_outside;
    }
}

/// System to rotate the world to keep the character centered and maintain hexagon orientation
pub fn follow_character_with_sphere_rotation(
    character_query: Query<(&Character, &Transform)>,
    mut world_parent_query: Query<&mut Transform, (With<crate::geotiles_bevy::WorldParent>, Without<Character>)>,
) {
    if let (Ok((_character, character_transform)), Ok(mut world_transform)) = 
        (character_query.single(), world_parent_query.single_mut()) {
        
        let character_position = character_transform.translation;
        
        // Check if the character has moved far enough from center to warrant rotation
        let is_outside = should_rotate_sphere_3d(character_position, &world_transform);
        
        if is_outside {
            println!("🎯 Character outside dead zone, rotating world to center");
            
            // Calculate the ideal 3D rotation to center the character
            if let Some(new_rotation) = calculate_ideal_3d_rotation(character_position, &world_transform) {
                let old_euler = world_transform.rotation.to_euler(EulerRot::YXZ);
                let new_euler = new_rotation.to_euler(EulerRot::YXZ);
                
                println!("🔄 Rotating world 3D: old({:.1}°,{:.1}°,{:.1}°) -> new({:.1}°,{:.1}°,{:.1}°)", 
                         old_euler.0.to_degrees(), old_euler.1.to_degrees(), old_euler.2.to_degrees(),
                         new_euler.0.to_degrees(), new_euler.1.to_degrees(), new_euler.2.to_degrees());
                
                // Apply 3D rotation to the world parent entity
                world_transform.rotation = new_rotation;
            }
        }
    }
}

/// Check if world should rotate based on character position using dead zone
fn should_rotate_sphere(character_position: Vec3, current_world_yaw: f32) -> bool {
    // Calculate the character's view-relative position by transforming by current world rotation
    let view_relative_position = rotate_point_around_y(-current_world_yaw, character_position);
    
    // The character is on the sphere surface. We want to calculate how far off-center it appears
    // from the camera's perspective. Since camera looks down -Z, we use X and Y coordinates
    // to determine the angular distance from center.
    
    // Calculate the 2D distance from camera center (X-Y plane) 
    let distance_from_center_2d = (view_relative_position.x.powi(2) + view_relative_position.y.powi(2)).sqrt();
    let distance_to_character = view_relative_position.length();
    
    // Calculate the angular distance using trigonometry
    // The angle is based on how far the character appears from the center in the camera view
    let angle_from_center = (distance_from_center_2d / distance_to_character).asin();
    let angle_degrees = angle_from_center.to_degrees();
    
    // Define dead zone: 20 degree radius from camera center
    let dead_zone_degrees = 20.0;
    let is_in_dead_zone = angle_degrees <= dead_zone_degrees;
    
    // Debug output
    println!("🔍 Dead zone check: char_pos=({:.2},{:.2},{:.2}) world_yaw={:.1}°", 
             character_position.x, character_position.y, character_position.z, current_world_yaw.to_degrees());
    println!("🔍 View relative: ({:.2},{:.2},{:.2}) | 2D distance: {:.2} | 3D distance: {:.2}", 
             view_relative_position.x, view_relative_position.y, view_relative_position.z,
             distance_from_center_2d, distance_to_character);
    println!("🔍 Angle from camera center: {:.1}° | Dead zone: {:.1}° | In dead zone: {} | Should rotate: {}", 
             angle_degrees, dead_zone_degrees, is_in_dead_zone, !is_in_dead_zone);
    
    // Only rotate if character is outside the dead zone
    !is_in_dead_zone
}

/// Check if world should rotate based on character position using 3D dead zone
fn should_rotate_sphere_3d(character_position: Vec3, world_transform: &Transform) -> bool {
    use crate::character_math;
    
    let dead_zone_angle = 20.0_f32.to_radians();
    
    // Use the tested mathematical approach
    let is_outside = character_math::is_outside_dead_zone(
        character_position, 
        world_transform.rotation, 
        dead_zone_angle
    );
    
    // Only print debug output when character is outside dead zone
    if is_outside {
        let view_relative_position = world_transform.rotation.inverse() * character_position;
        let distance_from_center_2d = (view_relative_position.x.powi(2) + view_relative_position.y.powi(2)).sqrt();
        let distance_to_character = view_relative_position.length();
        let angle_degrees = (distance_from_center_2d / distance_to_character).asin().to_degrees();
        
        println!("🔍 3D Dead zone check: char_pos=({:.2},{:.2},{:.2})", 
                 character_position.x, character_position.y, character_position.z);
        println!("🔍 View relative: ({:.2},{:.2},{:.2}) | 2D distance: {:.2} | 3D distance: {:.2}", 
                 view_relative_position.x, view_relative_position.y, view_relative_position.z,
                 distance_from_center_2d, distance_to_character);
        println!("🔍 Angle from camera center: {:.1}° | Dead zone: 20.0° | In dead zone: {} | Should rotate: {}", 
                 angle_degrees, false, is_outside);
    }
    
    is_outside
}

/// Calculate the ideal 3D rotation to bring character back into dead zone
fn calculate_ideal_3d_rotation(character_position: Vec3, world_transform: &Transform) -> Option<Quat> {
    use crate::character_math;
    
    let dead_zone_angle = 20.0_f32.to_radians();
    
    // Use the tested mathematical approach
    if let Some(new_rotation) = character_math::calculate_centering_rotation(
        character_position, 
        world_transform.rotation, 
        dead_zone_angle
    ) {
        // Calculate view position for debug output
        let view_relative_position = world_transform.rotation.inverse() * character_position;
        let distance_from_center_2d = (view_relative_position.x.powi(2) + view_relative_position.y.powi(2)).sqrt();
        
        println!("🎯 3D Centering rotation: offset=({:.2},{:.2}) distance={:.3} axis={} amount=1.0°", 
                 view_relative_position.x, view_relative_position.y, distance_from_center_2d,
                 if view_relative_position.y.abs() > view_relative_position.x.abs() { "X" } else { "Y" });
        
        Some(new_rotation)
    } else {
        println!("🎯 Character close enough to center, no rotation needed");
        None
    }
}

/// Calculate the ideal world rotation to bring character back into dead zone
fn calculate_ideal_yaw(character_position: Vec3, current_world_yaw: f32) -> Option<f32> {
    // Apply current world rotation to get the character's position relative to camera view
    let view_relative_position = rotate_point_around_y(-current_world_yaw, character_position);
    
    // Calculate the angle needed to bring character back toward camera center
    // We want to rotate the world so the character appears closer to the -Z axis (camera forward)
    let char_horizontal_distance = (view_relative_position.x.powi(2) + view_relative_position.z.powi(2)).sqrt();
    
    if char_horizontal_distance < 0.001 {
        return None; // Character is essentially at center, no rotation needed
    }
    
    // Calculate the angle the character is at relative to camera forward (-Z)
    let _current_angle = view_relative_position.z.atan2(view_relative_position.x);
    
    // We want to rotate to bring the character closer to the dead zone center
    // Small incremental rotation toward center
    let rotation_increment = PI / 24.0; // 7.5 degrees - smaller increments for smoother movement
    
    // Determine rotation direction to bring character toward center
    let new_world_yaw = if view_relative_position.x > 0.0 {
        // Character is to the right, rotate world LEFT (negative yaw) to center them
        current_world_yaw - rotation_increment
    } else {
        // Character is to the left, rotate world RIGHT (positive yaw) to center them
        current_world_yaw + rotation_increment
    };
    
    // Normalize to 0-2π range
    let normalized_yaw = ((new_world_yaw % (2.0 * PI)) + 2.0 * PI) % (2.0 * PI);
    
    println!("🎯 Centering rotation: char at ({:.2},{:.2}) -> rotating from {:.1}° to {:.1}°", 
             view_relative_position.x, view_relative_position.z, 
             current_world_yaw.to_degrees(), normalized_yaw.to_degrees());
    
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
    
    // Draw circle on the front face of the sphere (at z = sphere_radius)
    let circle_center = Vec3::new(0.0, 0.0, sphere_radius);
    
    // Use the stored state from the rotation system for consistent coloring
    let circle_color = if debug_gizmos.character_outside_dead_zone {
        Color::srgb(0.0, 0.5, 1.0) // Blue when character is outside dead zone
    } else {
        Color::srgb(1.0, 0.0, 1.0) // Magenta when character is inside dead zone
    };
    
    // Draw the dead zone circle with thick lines
    let circle_resolution = 64; // More segments for smoother circle
    for i in 0..circle_resolution {
        let angle1 = (i as f32 / circle_resolution as f32) * 2.0 * PI;
        let angle2 = ((i + 1) as f32 / circle_resolution as f32) * 2.0 * PI;
        
        let point1 = circle_center + Vec3::new(
            dead_zone_radius * angle1.cos(),
            dead_zone_radius * angle1.sin(),
            0.0
        );
        let point2 = circle_center + Vec3::new(
            dead_zone_radius * angle2.cos(),
            dead_zone_radius * angle2.sin(),
            0.0
        );
        
        gizmos.line(point1, point2, circle_color);
    }
    
    // Draw crosshairs for center reference
    let crosshair_size = dead_zone_radius * 0.3;
    
    // Main crosshairs
    gizmos.line(
        circle_center + Vec3::new(-crosshair_size, 0.0, 0.0),
        circle_center + Vec3::new(crosshair_size, 0.0, 0.0),
        Color::srgb(1.0, 1.0, 0.0) // Yellow crosshair
    );
    gizmos.line(
        circle_center + Vec3::new(0.0, -crosshair_size, 0.0),
        circle_center + Vec3::new(0.0, crosshair_size, 0.0),
        Color::srgb(1.0, 1.0, 0.0) // Yellow crosshair
    );
    
    // Center dot for visibility
    gizmos.sphere(circle_center, 0.05, Color::srgb(1.0, 1.0, 0.0)); // Yellow center dot
}

/// Draw debug ray from camera to character
fn draw_camera_ray_gizmo(
    gizmos: &mut Gizmos,
    character_position: Vec3,
    world_transform: &Transform,
) {
    // Camera position in world space (always at origin)
    let camera_pos = Vec3::ZERO;
    
    // Character position is already in local space, transform to world space
    let world_character_pos = world_transform.transform_point(character_position);
    
    // Draw larger sphere at camera position for visibility
    gizmos.sphere(camera_pos, 0.3, Color::srgb(1.0, 1.0, 1.0)); // Large white camera dot
    
    // Draw large sphere at character position for visibility 
    gizmos.sphere(world_character_pos, 0.2, Color::srgb(0.0, 1.0, 0.0)); // Green character dot
    
    // Only draw ray if character is not exactly at camera position
    let ray_length = (world_character_pos - camera_pos).length();
    if ray_length > 0.1 {
        // Draw multiple thick rays to make it very visible
        let ray_color = Color::srgb(1.0, 0.0, 0.0); // Bright red ray
        
        // Draw main ray
        gizmos.line(camera_pos, world_character_pos, ray_color);
        
        // Draw parallel rays for thickness
        let offset = 0.1;
        let perpendicular1 = Vec3::new(offset, 0.0, 0.0);
        let perpendicular2 = Vec3::new(0.0, offset, 0.0);
        
        gizmos.line(camera_pos + perpendicular1, world_character_pos + perpendicular1, ray_color);
        gizmos.line(camera_pos - perpendicular1, world_character_pos - perpendicular1, ray_color);
        gizmos.line(camera_pos + perpendicular2, world_character_pos + perpendicular2, ray_color);
        gizmos.line(camera_pos - perpendicular2, world_character_pos - perpendicular2, ray_color);
        
        // Draw many spheres along the ray to make it more visible
        for i in 1..10 {
            let t = i as f32 / 10.0;
            let point_on_ray = camera_pos + (world_character_pos - camera_pos) * t;
            gizmos.sphere(point_on_ray, 0.08, ray_color);
        }
        
        // Calculate where the camera-to-character ray intersects the dead zone plane
        let sphere_radius = 5.0;
        let dead_zone_plane_z = sphere_radius; // Front face of sphere
        
        // Ray direction from camera to character
        let ray_direction = (world_character_pos - camera_pos).normalize();
        
        // Find intersection with plane at z = sphere_radius
        if ray_direction.z.abs() > 0.001 { // Avoid division by zero
            let t = dead_zone_plane_z / ray_direction.z;
            if t > 0.0 { // Only if intersection is in front of camera
                let intersection_point = camera_pos + ray_direction * t;
                
                // Draw large intersection point
                gizmos.sphere(intersection_point, 0.25, Color::srgb(1.0, 1.0, 0.0)); // Large yellow intersection dot
                
                // Draw crosshairs at intersection
                let cross_size = 0.5;
                gizmos.line(
                    intersection_point + Vec3::new(-cross_size, 0.0, 0.0),
                    intersection_point + Vec3::new(cross_size, 0.0, 0.0),
                    Color::srgb(1.0, 1.0, 0.0)
                );
                gizmos.line(
                    intersection_point + Vec3::new(0.0, -cross_size, 0.0),
                    intersection_point + Vec3::new(0.0, cross_size, 0.0),
                    Color::srgb(1.0, 1.0, 0.0)
                );
            }
        }
    }
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
        gizmos.line_gradient(sphere_center, sphere_center + x_axis, Color::srgb(1.0, 0.0, 0.0), Color::srgb(1.0, 0.0, 0.0)); // Red X
        gizmos.line_gradient(sphere_center, sphere_center + y_axis, Color::srgb(0.0, 1.0, 0.0), Color::srgb(0.0, 1.0, 0.0)); // Green Y
        gizmos.line_gradient(sphere_center, sphere_center + z_axis, Color::srgb(0.0, 0.0, 1.0), Color::srgb(0.0, 0.0, 1.0)); // Blue Z
        
        // Add axis labels using small spheres
        gizmos.sphere(sphere_center + x_axis, 0.2, Color::srgb(1.0, 0.0, 0.0)); // Red X end
        gizmos.sphere(sphere_center + y_axis, 0.2, Color::srgb(0.0, 1.0, 0.0)); // Green Y end
        gizmos.sphere(sphere_center + z_axis, 0.2, Color::srgb(0.0, 0.0, 1.0)); // Blue Z end
    }
    
    // Draw character axes (smaller scale, local to character) if we have the transforms
    if let (Ok(world_transform), Ok((_character, character_transform))) = 
        (world_parent_query.single(), character_query.single()) {
        
        // Transform character position and rotation to world space
        let world_character_position = world_transform.transform_point(character_transform.translation);
        let world_character_rotation = world_transform.rotation * character_transform.rotation;
        
        // Draw character local axes with smaller scale
        let char_axis_length = 0.5;
        
        // Transform standard axes by world character rotation
        let char_x_axis = world_character_rotation * Vec3::X * char_axis_length;
        let char_y_axis = world_character_rotation * Vec3::Y * char_axis_length;
        let char_z_axis = world_character_rotation * Vec3::Z * char_axis_length;
        
        // Draw character axes - Lighter colors to distinguish from sphere axes with bold lines
        gizmos.line_gradient(world_character_position, world_character_position + char_x_axis, Color::srgb(1.0, 0.5, 0.5), Color::srgb(1.0, 0.5, 0.5)); // Light Red X
        gizmos.line_gradient(world_character_position, world_character_position + char_y_axis, Color::srgb(0.5, 1.0, 0.5), Color::srgb(0.5, 1.0, 0.5)); // Light Green Y
        gizmos.line_gradient(world_character_position, world_character_position + char_z_axis, Color::srgb(0.5, 0.5, 1.0), Color::srgb(0.5, 0.5, 1.0)); // Light Blue Z
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

