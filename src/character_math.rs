use bevy::prelude::*;

/// Mathematical utilities for character centering rotation
/// 
/// This module contains the core mathematical logic for calculating
/// how to rotate the sphere to keep the character centered in the camera view.

/// Calculate the rotation needed to center a character in camera view
/// 
/// # Arguments
/// * `character_position` - Character position in world space (on sphere surface)
/// * `world_rotation` - Current rotation of the world/sphere
/// * `dead_zone_angle` - Half-angle of dead zone cone in radians
/// 
/// # Returns
/// * `Some(rotation)` - The new world rotation to apply
/// * `None` - No rotation needed (character is in dead zone)
pub fn calculate_centering_rotation(
    character_position: Vec3,
    world_rotation: Quat,
    dead_zone_angle: f32,
) -> Option<Quat> {
    // Step 1: Transform character to camera view space
    let view_position = world_rotation.inverse() * character_position;
    
    // Step 2: Calculate angular distance from camera center
    let distance_2d = (view_position.x.powi(2) + view_position.y.powi(2)).sqrt();
    let distance_3d = view_position.length();
    let angle_from_center = (distance_2d / distance_3d).asin();
    
    // Step 3: Check if character is outside dead zone
    if angle_from_center <= dead_zone_angle {
        return None; // Character is in dead zone, no rotation needed
    }
    
    // Step 4: Calculate rotation to move character toward center
    let rotation_step = 1.0_f32.to_radians(); // 1 degree step
    
    // For small angles, the rotation needed is approximately:
    // - Rotate around X-axis by -angle to move +Y points toward center
    // - Rotate around Y-axis by +angle to move +X points toward center
    let rotation_around_x = if view_position.y.abs() > view_position.x.abs() {
        // Primarily vertical offset
        Quat::from_rotation_x(-view_position.y.signum() * rotation_step)
    } else {
        // Primarily horizontal offset  
        Quat::from_rotation_y(view_position.x.signum() * rotation_step)
    };
    
    Some(rotation_around_x * world_rotation)
}

/// Check if character is outside the dead zone using raycasting approach
/// 
/// **Geometric concept**: 
/// 1. Cast a ray from camera (0,0,0) toward the character
/// 2. Find where this ray intersects the sphere surface (same Z-distance as character)
/// 3. Check if this intersection point falls outside the dead zone circle
/// 
/// This is more intuitive than angle calculations - we literally check if the
/// camera-to-character ray passes through the dead zone circle or not.
pub fn is_outside_dead_zone(
    character_position: Vec3,
    world_rotation: Quat,
    dead_zone_angle: f32,
) -> bool {
    // Transform character to camera view space
    let view_position = world_rotation.inverse() * character_position;
    
    // Camera is at origin (0,0,0), looking down -Z axis
    // Character is somewhere on the sphere surface
    let camera_pos = Vec3::ZERO;
    let character_view_pos = view_position;
    
    // Create ray from camera toward character
    let ray_direction = (character_view_pos - camera_pos).normalize();
    
    // Dead zone circle is on the sphere surface at the point where camera ray
    // intersects the sphere. For our setup, this is approximately at distance
    // equal to the character's distance from camera.
    let sphere_distance = character_view_pos.length();
    
    // Point where camera center ray hits the sphere (dead zone center)
    let dead_zone_center = Vec3::new(0.0, 0.0, -sphere_distance);
    
    // Calculate dead zone radius on sphere surface
    let dead_zone_radius = sphere_distance * dead_zone_angle.tan();
    
    // Find where character ray intersects the sphere at the same Z distance
    // Ray: camera_pos + t * ray_direction
    // We want the point where z = -sphere_distance
    let t = -sphere_distance / ray_direction.z;
    let intersection_point = camera_pos + ray_direction * t;
    
    // Check if intersection point is outside the dead zone circle
    let distance_from_dead_zone_center = (intersection_point - dead_zone_center).length();
    
    distance_from_dead_zone_center > dead_zone_radius
}

/// Calculate the apparent screen position of a character (for testing)
pub fn character_screen_position(
    character_position: Vec3,
    world_rotation: Quat,
) -> Vec2 {
    let view_position = world_rotation.inverse() * character_position;
    Vec2::new(view_position.x, view_position.y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    const SPHERE_RADIUS: f32 = 5.0;
    const DEAD_ZONE_ANGLE: f32 = 20.0_f32.to_radians();
    const EPSILON: f32 = 0.01;

    #[test]
    fn test_character_at_center_is_in_dead_zone() {
        // Character directly facing camera should be in dead zone
        let character_pos = Vec3::new(0.0, 0.0, SPHERE_RADIUS);
        let world_rotation = Quat::IDENTITY;
        
        assert!(!is_outside_dead_zone(character_pos, world_rotation, DEAD_ZONE_ANGLE));
    }

    #[test]
    fn test_character_far_up_is_outside_dead_zone() {
        // Character moved far up should be outside dead zone
        let character_pos = Vec3::new(0.0, 3.0, 4.0); // ~37 degrees from center
        let world_rotation = Quat::IDENTITY;
        
        assert!(is_outside_dead_zone(character_pos, world_rotation, DEAD_ZONE_ANGLE));
    }

    #[test]
    fn test_rotation_brings_up_character_toward_center() {
        // Character moved up should be rotated down (toward center)
        let character_pos = Vec3::new(0.0, 2.0, 4.5); // Character above center
        let world_rotation = Quat::IDENTITY;
        
        let initial_screen_pos = character_screen_position(character_pos, world_rotation);
        assert!(initial_screen_pos.y > 0.5); // Character is up from center
        
        if let Some(new_rotation) = calculate_centering_rotation(character_pos, world_rotation, DEAD_ZONE_ANGLE) {
            let new_screen_pos = character_screen_position(character_pos, new_rotation);
            
            // After rotation, character should be closer to center (lower Y)
            assert!(new_screen_pos.y < initial_screen_pos.y, 
                   "Character should move toward center. Initial Y: {}, New Y: {}", 
                   initial_screen_pos.y, new_screen_pos.y);
        } else {
            panic!("Should need rotation for character outside dead zone");
        }
    }

    #[test]
    fn test_rotation_brings_right_character_toward_center() {
        // Character moved right should be rotated left (toward center)
        let character_pos = Vec3::new(2.0, 0.0, 4.5); // Character right of center
        let world_rotation = Quat::IDENTITY;
        
        let initial_screen_pos = character_screen_position(character_pos, world_rotation);
        assert!(initial_screen_pos.x > 0.5); // Character is right of center
        
        if let Some(new_rotation) = calculate_centering_rotation(character_pos, world_rotation, DEAD_ZONE_ANGLE) {
            let new_screen_pos = character_screen_position(character_pos, new_rotation);
            
            // After rotation, character should be closer to center (lower X)
            assert!(new_screen_pos.x < initial_screen_pos.x,
                   "Character should move toward center. Initial X: {}, New X: {}", 
                   initial_screen_pos.x, new_screen_pos.x);
        } else {
            panic!("Should need rotation for character outside dead zone");
        }
    }

    #[test]
    fn test_repeated_rotations_converge_to_center() {
        // Apply rotation multiple times and verify character gets closer to center
        let mut character_pos = Vec3::new(1.5, 1.5, 4.0); // Character diagonal from center
        let mut world_rotation = Quat::IDENTITY;
        
        let initial_distance = character_screen_position(character_pos, world_rotation).length();
        
        // Apply up to 10 rotation steps
        for i in 0..10 {
            if let Some(new_rotation) = calculate_centering_rotation(character_pos, world_rotation, DEAD_ZONE_ANGLE) {
                world_rotation = new_rotation;
                
                let current_distance = character_screen_position(character_pos, world_rotation).length();
                println!("Step {}: distance from center = {:.3}", i, current_distance);
                
                // Each step should bring character closer to center (or at least not further)
                assert!(current_distance <= initial_distance + EPSILON,
                       "Distance should not increase. Initial: {:.3}, Current: {:.3}", 
                       initial_distance, current_distance);
            } else {
                // Character is now in dead zone - success!
                println!("Converged after {} steps", i);
                return;
            }
        }
        
        // If we get here, check that we at least made progress
        let final_distance = character_screen_position(character_pos, world_rotation).length();
        assert!(final_distance < initial_distance * 0.8,
               "Should have made significant progress. Initial: {:.3}, Final: {:.3}",
               initial_distance, final_distance);
    }

    #[test]
    fn test_no_rotation_needed_for_centered_character() {
        // Character at center should not need rotation
        let character_pos = Vec3::new(0.0, 0.0, SPHERE_RADIUS);
        let world_rotation = Quat::IDENTITY;
        
        let result = calculate_centering_rotation(character_pos, world_rotation, DEAD_ZONE_ANGLE);
        assert!(result.is_none(), "No rotation should be needed for centered character");
    }

    #[test]
    fn test_raycasting_approach_matches_angular_approach() {
        // Test that raycasting gives same results as angular calculation
        let test_positions = vec![
            Vec3::new(0.0, 0.0, SPHERE_RADIUS),     // center - should be in dead zone
            Vec3::new(0.0, 1.0, 4.9),               // slightly up - should be in dead zone
            Vec3::new(0.0, 2.0, 4.5),               // far up - should be outside dead zone
            Vec3::new(1.5, 0.0, 4.8),               // right - should be outside dead zone
            Vec3::new(1.0, 1.0, 4.6),               // diagonal - should be outside dead zone
        ];
        
        let world_rotation = Quat::IDENTITY;
        
        for pos in test_positions {
            let raycasting_result = is_outside_dead_zone(pos, world_rotation, DEAD_ZONE_ANGLE);
            
            // Compare with angular approach for verification
            let view_position = world_rotation.inverse() * pos;
            let distance_2d = (view_position.x.powi(2) + view_position.y.powi(2)).sqrt();
            let distance_3d = view_position.length();
            let angle_from_center = (distance_2d / distance_3d).asin();
            let angular_result = angle_from_center > DEAD_ZONE_ANGLE;
            
            println!("Position: {:?} | Angle: {:.1}° | Raycasting: {} | Angular: {}", 
                     pos, angle_from_center.to_degrees(), raycasting_result, angular_result);
            
            assert_eq!(raycasting_result, angular_result, 
                      "Raycasting and angular approaches should match for position {:?}", pos);
        }
    }
}