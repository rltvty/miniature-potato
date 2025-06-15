//! Camera controls for orbital movement around the sphere

use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};

/// Component to mark our orbit camera
#[derive(Component)]
pub struct OrbitCamera {
    /// Distance from the target (sphere center)
    pub distance: f32,
    /// Horizontal rotation angle in radians
    pub yaw: f32,
    /// Vertical rotation angle in radians  
    pub pitch: f32,
    /// Point the camera orbits around
    pub target: Vec3,
    /// Camera movement sensitivity
    pub sensitivity: f32,
    /// Zoom sensitivity
    pub zoom_sensitivity: f32,
    /// Min/max zoom distances
    pub min_distance: f32,
    pub max_distance: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            distance: 6.0,
            yaw: 0.0,
            pitch: 0.3, // Start slightly above the sphere
            target: Vec3::ZERO,
            sensitivity: 0.005,
            zoom_sensitivity: 0.5,
            min_distance: 2.5,
            max_distance: 20.0,
        }
    }
}

/// Camera controller system for mouse orbit, zoom, and WASD panning
pub fn camera_controller(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<(&mut OrbitCamera, &mut Transform)>,
    time: Res<Time>,
) {
    if let Ok((mut orbit_camera, mut transform)) = camera_query.single_mut() {
        // Handle mouse rotation (only when right mouse button is held to avoid conflicts with tile selection)
        if mouse_button_input.pressed(MouseButton::Right) {
            for event in mouse_motion_events.read() {
                orbit_camera.yaw -= event.delta.x * orbit_camera.sensitivity;
                orbit_camera.pitch -= event.delta.y * orbit_camera.sensitivity;
                
                // Clamp pitch to prevent camera flipping
                orbit_camera.pitch = orbit_camera.pitch.clamp(-1.54, 1.54); // Almost ±90 degrees
            }
        } else {
            // Clear events if not rotating to prevent accumulation
            mouse_motion_events.clear();
        }
        
        // Handle mouse wheel zoom
        for event in mouse_wheel_events.read() {
            orbit_camera.distance -= event.y * orbit_camera.zoom_sensitivity;
            orbit_camera.distance = orbit_camera.distance.clamp(
                orbit_camera.min_distance, 
                orbit_camera.max_distance
            );
        }
        
        // Handle WASD panning
        let mut pan_vector = Vec3::ZERO;
        let pan_speed = 2.0 * time.delta_secs();
        
        if keyboard_input.pressed(KeyCode::KeyW) {
            pan_vector.z -= pan_speed;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            pan_vector.z += pan_speed;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            pan_vector.x -= pan_speed;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            pan_vector.x += pan_speed;
        }
        
        // Apply panning to the target (what we orbit around)
        if pan_vector.length() > 0.0 {
            // Transform the pan vector relative to the camera's current orientation
            let camera_transform = calculate_camera_transform(&orbit_camera);
            let right = camera_transform.rotation * Vec3::X;
            let forward = camera_transform.rotation * Vec3::Z;
            
            orbit_camera.target += right * pan_vector.x + forward * pan_vector.z;
        }
        
        // Update camera transform based on orbit parameters
        *transform = calculate_camera_transform(&orbit_camera);
    }
}

/// Calculate camera transform from orbit parameters
pub fn calculate_camera_transform(orbit_camera: &OrbitCamera) -> Transform {
    // Calculate position using spherical coordinates
    let x = orbit_camera.distance * orbit_camera.pitch.cos() * orbit_camera.yaw.sin();
    let y = orbit_camera.distance * orbit_camera.pitch.sin();
    let z = orbit_camera.distance * orbit_camera.pitch.cos() * orbit_camera.yaw.cos();
    
    let position = orbit_camera.target + Vec3::new(x, y, z);
    
    Transform::from_translation(position).looking_at(orbit_camera.target, Vec3::Y)
}
