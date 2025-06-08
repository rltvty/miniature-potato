//! World setup and management using true Goldberg polyhedron

use bevy::prelude::*;
use bevy::pbr::wireframe::WireframeConfig;
use crate::goldberg_polyhedron::GoldbergPolyhedron;
use crate::game_tiles::GameTileSystem;
use crate::ray_casting::IcosphereTriangles;

/// Resource to control border visibility
#[derive(Resource, Default)]
pub struct BorderVisibility {
    pub show_borders: bool,
}

/// Setup the world with a true Goldberg polyhedron
pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    println!("🌍 Setting up world with true Goldberg polyhedron...");
    
    // Configuration for our world
    const RADIUS: f32 = 2.0;
    
    // Goldberg parameters - GP(m,n):
    // GP(1,0) = Dodecahedron (12 pentagons, 0 hexagons) - simplest non-trivial case
    // GP(1,1) = Truncated icosahedron/soccer ball (12 pentagons, 20 hexagons)
    // GP(2,0) = More detailed (12 pentagons, 40 hexagons)
    // GP(2,1) = Even more detailed (12 pentagons, 70 hexagons)
    
    // Start with soccer ball pattern for familiarity
    const M: u32 = 1;
    const N: u32 = 1;
    
    println!("   Using GP({},{}) - {}", M, N, goldberg_description(M, N));
    
    // Create the true Goldberg polyhedron
    let polyhedron = GoldbergPolyhedron::new(M, N, RADIUS);
    
    // Create the game tile system
    let tile_system = GameTileSystem::new(polyhedron);
    
    // Extract mesh data for rendering
    let mesh = tile_system.mesh_data.mesh.clone();
    let vertices: Vec<Vec3> = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap()
        .iter()
        .map(|&[x, y, z]| Vec3::new(x, y, z))
        .collect();
    
    let indices: Vec<u32> = match mesh.indices() {
        Some(bevy::render::mesh::Indices::U32(indices)) => indices.clone(),
        Some(bevy::render::mesh::Indices::U16(indices)) => {
            indices.iter().map(|&i| i as u32).collect()
        },
        None => Vec::new(),
    };
    
    // Print detailed statistics
    tile_system.print_statistics();
    println!("   Mesh: {} vertices, {} triangles", vertices.len(), indices.len() / 3);
    
    // Spawn the polyhedron entity
    let mut entity_commands = commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.6, 0.8), // Nice blue color for the world
            metallic: 0.0,
            perceptual_roughness: 0.7,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Add triangle data for ray casting (reusing existing system)
    entity_commands.insert(IcosphereTriangles { vertices, indices });

    // Insert the game tile system as a resource
    commands.insert_resource(tile_system);
    
    // Insert border visibility resource
    commands.insert_resource(BorderVisibility { show_borders: true });

    // Add lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.9, 0.9, 1.0),
        brightness: 0.3,
        affects_lightmapped_meshes: true,
    });
    
    // Initialize wireframe configuration (off by default)
    wireframe_config.global = false;
    
    println!("✅ World setup complete!");
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

/// Toggle border visibility with 'B' key
pub fn toggle_borders(
    mut border_visibility: ResMut<BorderVisibility>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyB) {
        border_visibility.show_borders = !border_visibility.show_borders;
        println!("Border visibility: {}", if border_visibility.show_borders { "ON" } else { "OFF" });
    }
}

/// Get a human-readable description of a Goldberg polyhedron
fn goldberg_description(m: u32, n: u32) -> &'static str {
    match (m, n) {
        (0, 0) => "Degenerate case",
        (1, 0) => "Dodecahedron",
        (1, 1) => "Soccer ball (Truncated icosahedron)",
        (2, 0) => "Detailed geodesic dome",
        (2, 1) => "Very detailed geodesic dome",
        (3, 0) => "High-detail geodesic dome",
        _ => "Custom Goldberg polyhedron",
    }
}

/// System to print tile information when 'I' is pressed
pub fn print_tile_info(
    keyboard: Res<ButtonInput<KeyCode>>,
    tile_system: Res<GameTileSystem>,
) {
    if keyboard.just_pressed(KeyCode::KeyI) {
        println!("\n📊 === TILE SYSTEM INFO ===");
        tile_system.print_statistics();
        
        println!("\n🔢 Expected vs Actual counts:");
        println!("   Expected pentagons: 12, Actual: {}", 
                 tile_system.polyhedron.pentagons.len());
        println!("   Expected hexagons: {}, Actual: {}", 
                 tile_system.polyhedron.expected_hexagon_count(),
                 tile_system.polyhedron.hexagons.len());
        println!("   Expected total faces: {}, Actual: {}", 
                 tile_system.polyhedron.expected_face_count(),
                 tile_system.tiles.len());
        
        println!("\n📐 Goldberg parameters:");
        println!("   GP({},{}) - {}", 
                 tile_system.polyhedron.m, 
                 tile_system.polyhedron.n,
                 goldberg_description(tile_system.polyhedron.m, tile_system.polyhedron.n));
        println!("   Radius: {}", tile_system.polyhedron.radius);
    }
}
