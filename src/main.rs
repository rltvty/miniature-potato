//! A basic 3D scene with a spherical world for a hexagonal tile-based game.
//! Features a camera, directional light, and a sphere that will eventually be
//! tessellated with hexagons and pentagons.

use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, camera_controller)
        .run();
}

/// Component to mark our orbit camera
#[derive(Component)]
struct OrbitCamera {
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

/// Set up the basic 3D scene with camera, lighting, and sphere
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn the sphere that will represent our world
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(2.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.8), // Nice blue color for the world
            metallic: 0.0,
            perceptual_roughness: 0.8,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Add a directional light to illuminate the sphere
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // Add ambient light for better overall visibility
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.9, 0.9, 1.0),
        brightness: 0.3,
        affects_lightmapped_meshes: true,
    });

    // Spawn camera with orbit controls
    let orbit_camera = OrbitCamera::default();
    let transform = calculate_camera_transform(&orbit_camera);
    
    commands.spawn((
        Camera3d::default(),
        transform,
        orbit_camera,
    ));
}

/// Camera controller system for mouse orbit and zoom
fn camera_controller(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut camera_query: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    if let Ok((mut orbit_camera, mut transform)) = camera_query.single_mut() {
        // Handle mouse rotation (only when left mouse button is held)
        if mouse_button_input.pressed(MouseButton::Left) {
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
        
        // Update camera transform based on orbit parameters
        *transform = calculate_camera_transform(&orbit_camera);
    }
}

/// Calculate camera transform from orbit parameters
fn calculate_camera_transform(orbit_camera: &OrbitCamera) -> Transform {
    // Calculate position using spherical coordinates
    let x = orbit_camera.distance * orbit_camera.pitch.cos() * orbit_camera.yaw.sin();
    let y = orbit_camera.distance * orbit_camera.pitch.sin();
    let z = orbit_camera.distance * orbit_camera.pitch.cos() * orbit_camera.yaw.cos();
    
    let position = orbit_camera.target + Vec3::new(x, y, z);
    
    Transform::from_translation(position).looking_at(orbit_camera.target, Vec3::Y)
}
