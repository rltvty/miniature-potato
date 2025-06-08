//! World setup and management

use bevy::prelude::*;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use crate::goldberg::GoldbergPolyhedron;
use crate::goldberg_tiles::generate_tiles_from_goldberg;
use crate::ray_casting::IcosphereTriangles;

/// Setup the basic world with a Goldberg polyhedron and tile system
pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    // Configuration for our world
    const RADIUS: f32 = 2.0;
    
    // Goldberg parameters:
    // (1,0) = Soccer ball pattern (12 pentagons, 20 hexagons) - simplest form
    // (2,0) = More detailed version (12 pentagons, 40 hexagons)
    // (1,1) = Twisted version (12 pentagons, 30 hexagons)
    // Comment/uncomment the desired configuration:
    
    // Soccer ball pattern
    const H: u32 = 3;
    const K: u32 = 1;
    
    // More detailed version
    // const H: u32 = 2;
    // const K: u32 = 0;
    
    // Twisted version
    // const H: u32 = 1;
    // const K: u32 = 1;
    
    // Create Goldberg polyhedron with specified parameters
    let goldberg = GoldbergPolyhedron::new(RADIUS, H, K);
    
    // Generate mesh with vertex data
    let (mesh, vertices, indices) = goldberg.generate_with_data();

    // Generate tile system from triangle data
    let tile_map = generate_tiles_from_goldberg(&vertices, &indices, H, K, RADIUS);
    
    // Insert tile map as resource
    commands.insert_resource(tile_map);

    // Spawn the polyhedron that will represent our world
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

    // Add triangle data for ray casting
    entity_commands.insert(IcosphereTriangles { vertices, indices });

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
    
    // Initialize wireframe configuration (off by default)
    wireframe_config.global = false;
}

/// Toggle wireframe mode with spacebar
pub fn toggle_wireframe(
    mut wireframe_config: ResMut<WireframeConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        wireframe_config.global = !wireframe_config.global;
        println!("Wireframe mode: {}", if wireframe_config.global { "ON" } else { "OFF" });
    }
}
