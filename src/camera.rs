//! Camera controls with sphere rotation

use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};

/// Component to mark our camera
#[derive(Component)]
pub struct OrbitCamera {
    /// Camera position
    pub position: Vec3,
    /// Camera movement sensitivity for WASD
    pub move_sensitivity: f32,
    /// Zoom sensitivity
    pub zoom_sensitivity: f32,
    /// Min/max zoom distances from origin
    pub min_distance: f32,
    pub max_distance: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 15.0), // Camera starts 15 units back on Z axis
            move_sensitivity: 5.0,
            zoom_sensitivity: 1.0,
            min_distance: 2.5,
            max_distance: 50.0,
        }
    }
}

/// Resource to track sphere rotation
#[derive(Resource, Default)]
pub struct SphereRotation {
    /// Current rotation of the sphere
    pub rotation: Quat,
    /// Mouse sensitivity for rotation
    pub sensitivity: f32,
}

impl SphereRotation {
    pub fn new() -> Self {
        Self {
            rotation: Quat::IDENTITY,
            sensitivity: 0.005,
        }
    }
}

/// Camera controller system - mouse rotates sphere, WASD moves camera freely
pub fn camera_controller(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<(&mut OrbitCamera, &mut Transform)>,
    mut sphere_rotation: ResMut<SphereRotation>,
    time: Res<Time>,
) {
    if let Ok((mut orbit_camera, mut transform)) = camera_query.single_mut() {
        // Handle mouse rotation - rotates the sphere, not the camera
        if mouse_button_input.pressed(MouseButton::Right) {
            for event in mouse_motion_events.read() {
                // Create rotation deltas
                let yaw_delta = -event.delta.x * sphere_rotation.sensitivity;
                let pitch_delta = -event.delta.y * sphere_rotation.sensitivity;
                
                // Apply rotations to sphere
                let yaw_rotation = Quat::from_axis_angle(Vec3::Y, yaw_delta);
                let pitch_rotation = Quat::from_axis_angle(Vec3::X, pitch_delta);
                
                // Combine rotations with existing sphere rotation
                sphere_rotation.rotation = yaw_rotation * sphere_rotation.rotation * pitch_rotation;
            }
        } else {
            // Clear events if not rotating to prevent accumulation
            mouse_motion_events.clear();
        }
        
        // Handle mouse wheel zoom - move camera along its forward direction
        for event in mouse_wheel_events.read() {
            let forward = transform.rotation * Vec3::NEG_Z; // Forward is -Z in Bevy
            let zoom_amount = event.y * orbit_camera.zoom_sensitivity;
            orbit_camera.position += forward * zoom_amount;
            
            // Clamp distance from origin to prevent getting too close/far
            let distance_from_origin = orbit_camera.position.length();
            if distance_from_origin < orbit_camera.min_distance {
                orbit_camera.position = orbit_camera.position.normalize() * orbit_camera.min_distance;
            } else if distance_from_origin > orbit_camera.max_distance {
                orbit_camera.position = orbit_camera.position.normalize() * orbit_camera.max_distance;
            }
        }
        
        // Handle WASD camera movement - move in camera-relative directions
        let mut move_vector = Vec3::ZERO;
        let move_speed = orbit_camera.move_sensitivity * time.delta_secs();
        
        if keyboard_input.pressed(KeyCode::KeyW) {
            move_vector.z -= move_speed; // Forward
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            move_vector.z += move_speed; // Backward
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            move_vector.x -= move_speed; // Left
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            move_vector.x += move_speed; // Right
        }
        
        // Apply camera movement in world space relative to camera orientation
        if move_vector.length() > 0.0 {
            let right = transform.rotation * Vec3::X;
            let forward = transform.rotation * Vec3::NEG_Z; // Forward is -Z in Bevy
            let up = transform.rotation * Vec3::Y;
            
            orbit_camera.position += right * move_vector.x + up * move_vector.y + forward * move_vector.z;
        }
        
        // Update camera transform - keep current orientation, don't always look at origin
        transform.translation = orbit_camera.position;
        // Don't call look_at - maintain the camera's current forward direction
    }
}

/// System to apply sphere rotation to all tile entities
pub fn rotate_sphere_system(
    sphere_rotation: Res<SphereRotation>,
    mut tile_query: Query<&mut Transform, (With<crate::geotiles_bevy::TileComponent>, Without<OrbitCamera>)>,
) {
    for mut transform in tile_query.iter_mut() {
        // Apply the sphere rotation to each tile's base transform
        transform.rotation = sphere_rotation.rotation;
    }
}
