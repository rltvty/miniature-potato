//! World setup using geotiles crate

use bevy::prelude::*;
use crate::geotiles_integration::{GeotilesWorld, TileEntity, create_tile_mesh};

/// Setup world using geotiles
pub fn setup_geotiles_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("🌍 Setting up world with geotiles...");
    
    // Configuration
    const RADIUS: f32 = 2.0;
    const SUBDIVISIONS: u8 = 2; // Similar detail level to previous implementation
    const TILE_SIZE: f32 = 0.9; // 90% size for visible gaps between tiles
    
    // Create the geotiles world
    let mut geotiles_world = GeotilesWorld::new(RADIUS, SUBDIVISIONS, TILE_SIZE);
    
    // Materials for different tile types
    let pentagon_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 1.0), // Magenta for pentagons
        metallic: 0.2,
        perceptual_roughness: 0.5,
        ..default()
    });
    
    let hexagon_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.8, 0.0), // Green for hexagons
        metallic: 0.2,
        perceptual_roughness: 0.5,
        ..default()
    });
    
    let hover_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 0.0), // Yellow for hover
        metallic: 0.3,
        perceptual_roughness: 0.4,
        emissive: LinearRgba::rgb(0.5, 0.5, 0.0),
        ..default()
    });
    
    // Create entities for each tile
    for (tile_index, tile) in geotiles_world.hexasphere.tiles.iter().enumerate() {
        let center = Vec3::new(
            tile.center_point.x as f32,
            tile.center_point.y as f32,
            tile.center_point.z as f32,
        );
        
        let boundary: Vec<Vec3> = tile.boundary.iter()
            .map(|p| Vec3::new(p.x as f32, p.y as f32, p.z as f32))
            .collect();
        
        let is_pentagon = tile.boundary.len() == 5;
        
        // Create mesh for this tile
        let mesh = create_tile_mesh(&boundary, center);
        let mesh_handle = meshes.add(mesh);
        
        // Choose material based on tile type
        let material = if is_pentagon {
            pentagon_material.clone()
        } else {
            hexagon_material.clone()
        };
        
        // Spawn the tile entity
        let entity = commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material),
            Transform::from_translation(Vec3::ZERO),
            TileEntity {
                tile_index,
                is_pentagon,
            },
        )).id();
        
        // Store the mapping
        geotiles_world.entity_to_tile.insert(entity, tile_index);
        geotiles_world.tile_to_entity.insert(tile_index, entity);
    }
    
    // Store materials for later use
    commands.insert_resource(TileMaterials {
        pentagon: pentagon_material,
        hexagon: hexagon_material,
        hover: hover_material,
    });
    
    // Insert the geotiles world as a resource
    commands.insert_resource(geotiles_world);
    
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
        ..default()
    });
    
    println!("✅ Geotiles world setup complete!");
}

/// Resource to store material handles
#[derive(Resource)]
pub struct TileMaterials {
    pub pentagon: Handle<StandardMaterial>,
    pub hexagon: Handle<StandardMaterial>,
    pub hover: Handle<StandardMaterial>,
}

/// System to handle tile hover effects
pub fn tile_hover_system(
    mut commands: Commands,
    mut geotiles_world: ResMut<GeotilesWorld>,
    tile_materials: Res<TileMaterials>,
    tile_query: Query<(Entity, &TileEntity)>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window>,
) {
    // Get cursor position
    let Ok(window) = window_query.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    
    // Create ray from camera through cursor
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { return };
    
    // Find intersected tile using sphere intersection
    let sphere_center = Vec3::ZERO;
    let sphere_radius = 2.0; // Should match RADIUS constant
    
    // Simple sphere ray intersection
    let oc = ray.origin - sphere_center;
    let a: f32 = ray.direction.dot(*ray.direction);
    let b: f32 = 2.0 * oc.dot(*ray.direction);
    let c: f32 = oc.dot(oc) - sphere_radius * sphere_radius;
    let discriminant: f32 = b * b - 4.0 * a * c;
    
    let mut hovered_tile = None;
    
    if discriminant >= 0.0 {
        let t = (-b - discriminant.sqrt()) / (2.0 * a);
        if t > 0.0 {
            let hit_point = ray.origin + ray.direction * t;
            
            // Find closest tile to hit point
            if let Some(tile_index) = geotiles_world.find_closest_tile(hit_point) {
                hovered_tile = Some(tile_index);
            }
        }
    }
    
    // Update materials based on hover state
    if hovered_tile != geotiles_world.hovered_tile {
        // Reset previous hover
        if let Some(prev_tile) = geotiles_world.hovered_tile {
            if let Some(&entity) = geotiles_world.tile_to_entity.get(&prev_tile) {
                if let Ok((_, tile_entity)) = tile_query.get(entity) {
                    // Restore original material
                    let new_material = if tile_entity.is_pentagon {
                        tile_materials.pentagon.clone()
                    } else {
                        tile_materials.hexagon.clone()
                    };
                    commands.entity(entity).insert(MeshMaterial3d(new_material));
                }
            }
        }
        
        // Apply hover to new tile
        if let Some(tile_index) = hovered_tile {
            if let Some(&entity) = geotiles_world.tile_to_entity.get(&tile_index) {
                commands.entity(entity).insert(MeshMaterial3d(tile_materials.hover.clone()));
                
                // Debug output (only on change)
                let is_pentagon = geotiles_world.is_pentagon(tile_index);
                println!("Hovering {} tile {}", 
                    if is_pentagon { "pentagon" } else { "hexagon" },
                    tile_index
                );
            }
        }
        
        geotiles_world.hovered_tile = hovered_tile;
    }
}