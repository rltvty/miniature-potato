//! World setup and management

use bevy::prelude::*;
use crate::icosphere::Icosphere;

/// Setup the basic world with an icosphere
pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create icosphere with 1 subdivision for now
    let icosphere = Icosphere::new(2.0, 2);
    let mesh = icosphere.generate();

    // Spawn the icosphere that will represent our world
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
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
}
