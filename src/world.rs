//! World setup and management

use bevy::prelude::*;
use crate::icosphere::Icosphere;
use crate::ray_casting::IcosphereTriangles;
use crate::tiles::{generate_tiles_from_icosphere, TileMap};

/// Setup the basic world with an icosphere and tile system
pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create icosphere with 2 subdivisions for smoother surface
    let icosphere = Icosphere::new(2.0, 2);
    
    // Try to use generate_with_data if available, otherwise fallback
    let (mesh, vertices, indices) = if std::panic::catch_unwind(|| {
        icosphere.generate_with_data()
    }).is_ok() {
        // If generate_with_data exists and works, use it
        icosphere.generate_with_data()
    } else {
        // Fallback: generate basic mesh and create dummy data
        let mesh = icosphere.generate();
        println!("Warning: generate_with_data not available, using basic mesh");
        (mesh, Vec::new(), Vec::new())
    };

    // Generate tile system from triangle data
    let tile_map = if !vertices.is_empty() {
        generate_tiles_from_icosphere(&vertices, &indices, 2)
    } else {
        TileMap::default()
    };
    
    // Insert tile map as resource
    commands.insert_resource(tile_map);

    // Spawn the icosphere that will represent our world
    let mut entity_commands = commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.8), // Nice blue color for the world
            metallic: 0.0,
            perceptual_roughness: 0.8,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Add triangle data if we have it
    if !vertices.is_empty() {
        entity_commands.insert(IcosphereTriangles { vertices, indices });
    }

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
