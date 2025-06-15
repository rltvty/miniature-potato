//! Bevy integration for geotiles following the example from the README

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use geotiles::{Hexasphere, tile::TileOrientation};
use std::f32::consts::PI;

/// Resource to store the hexasphere and related data
#[derive(Resource)]
pub struct HexasphereResource {
    pub hexasphere: Hexasphere,
    pub uniform_radius: f64,
    pub tile_entities: Vec<Entity>,
}

/// Component to mark tile entities
#[derive(Component)]
pub struct TileComponent {
    pub index: usize,
    pub is_pentagon: bool,
}

/// Setup the hexasphere world
pub fn setup_hexasphere_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("🌍 Setting up hexasphere world with geotiles...");
    
    // Configuration
    const SPHERE_RADIUS: f64 = 5.0;
    const SUBDIVISIONS: usize = 3;
    const TILE_SIZE: f64 = 0.9;
    
    // Create hexasphere
    let hexasphere = Hexasphere::new(SPHERE_RADIUS, SUBDIVISIONS, TILE_SIZE);
    let uniform_radius = hexasphere.get_uniform_hexagon_radius();
    
    println!("Generated {} tiles", hexasphere.tiles.len());
    println!("Uniform hexagon radius: {:.3}", uniform_radius);
    
    // Materials
    let hexagon_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.7, 0.3),
        metallic: 0.2,
        perceptual_roughness: 0.5,
        ..default()
    });
    
    let pentagon_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.3, 0.8),
        metallic: 0.2,
        perceptual_roughness: 0.5,
        ..default()
    });
    
    // Create pre-computed meshes
    let hexagon_mesh = meshes.add(create_regular_polygon_mesh(6, uniform_radius as f32));
    let pentagon_mesh = meshes.add(create_regular_polygon_mesh(5, uniform_radius as f32 * 0.9));
    
    let mut tile_entities = Vec::new();
    
    // Spawn tiles
    for (index, tile) in hexasphere.tiles.iter().enumerate() {
        let is_pentagon = tile.boundary.len() == 5;
        
        // Skip if we can't get orientation
        let Some(orientation) = tile.get_orientation() else {
            continue;
        };
        
        // Convert orientation to transform
        let transform = Transform {
            translation: Vec3::new(
                tile.center_point.x as f32,
                tile.center_point.y as f32,
                tile.center_point.z as f32,
            ),
            rotation: orientation_to_quat(&orientation),
            scale: Vec3::ONE,
        };
        
        // Choose mesh and material
        let (mesh_handle, material_handle) = if is_pentagon {
            (pentagon_mesh.clone(), pentagon_material.clone())
        } else {
            (hexagon_mesh.clone(), hexagon_material.clone())
        };
        
        let entity = commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
            transform,
            TileComponent { index, is_pentagon },
        )).id();
        
        tile_entities.push(entity);
    }
    
    let tile_count = tile_entities.len();
    
    // Store the hexasphere resource
    commands.insert_resource(HexasphereResource {
        hexasphere,
        uniform_radius,
        tile_entities,
    });
    
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
    
    println!("✅ Hexasphere world setup complete with {} tiles!", tile_count);
}

/// Convert TileOrientation to Bevy quaternion
fn orientation_to_quat(orientation: &TileOrientation) -> Quat {
    // The TileOrientation provides a local coordinate system
    // We need to convert this to a quaternion for Bevy
    
    // Get the vectors from the orientation
    let right = Vec3::new(
        orientation.right.x as f32,
        orientation.right.y as f32,
        orientation.right.z as f32,
    );
    
    let up = Vec3::new(
        orientation.up.x as f32,
        orientation.up.y as f32,
        orientation.up.z as f32,
    );
    
    let forward = Vec3::new(
        orientation.forward.x as f32,
        orientation.forward.y as f32,
        orientation.forward.z as f32,
    );
    
    // Create rotation matrix and convert to quaternion
    Quat::from_mat3(&Mat3::from_cols(right, up, forward))
}

/// Create a regular polygon mesh (hexagon or pentagon)
fn create_regular_polygon_mesh(sides: usize, radius: f32) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList, 
        bevy::render::render_asset::RenderAssetUsages::all()
    );
    
    // Generate vertices
    let mut vertices = vec![Vec3::ZERO]; // Center vertex
    for i in 0..sides {
        let angle = (i as f32) * 2.0 * PI / (sides as f32);
        vertices.push(Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius));
    }
    
    // Generate indices
    let mut indices = Vec::new();
    for i in 0..sides {
        let next = (i + 1) % sides;
        indices.extend_from_slice(&[0, i as u32 + 1, next as u32 + 1]);
    }
    
    // Set mesh attributes
    let positions: Vec<[f32; 3]> = vertices.iter().map(|v| [v.x, v.y, v.z]).collect();
    let normals: Vec<[f32; 3]> = vec![[0.0, 1.0, 0.0]; vertices.len()];
    let uvs: Vec<[f32; 2]> = vertices.iter()
        .map(|v| [(v.x / radius + 1.0) * 0.5, (v.z / radius + 1.0) * 0.5])
        .collect();
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}

/// System to handle tile hover with materials
pub fn tile_hover_system(
    mut materials: ResMut<Assets<StandardMaterial>>,
    hexasphere_res: Res<HexasphereResource>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window>,
    tile_query: Query<(&TileComponent, &GlobalTransform, &MeshMaterial3d<StandardMaterial>)>,
    mut hovered_tile: Local<Option<usize>>,
) {
    let Ok(window) = window_query.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { return };
    
    // Find closest tile to ray
    let mut closest_tile = None;
    let mut closest_distance = f32::MAX;
    
    for (tile_component, tile_transform, _) in tile_query.iter() {
        let tile_pos = tile_transform.translation();
        let to_tile = tile_pos - ray.origin;
        let proj_length = to_tile.dot(*ray.direction);
        
        if proj_length > 0.0 {
            let closest_point = ray.origin + ray.direction * proj_length;
            let distance = (closest_point - tile_pos).length();
            
            if distance < closest_distance && distance < hexasphere_res.uniform_radius as f32 {
                closest_distance = distance;
                closest_tile = Some(tile_component.index);
            }
        }
    }
    
    // Update hover state
    if closest_tile != *hovered_tile {
        // Reset previous tile color
        if let Some(prev_index) = *hovered_tile {
            for (tile_component, _, material_handle) in tile_query.iter() {
                if tile_component.index == prev_index {
                    if let Some(material) = materials.get_mut(material_handle) {
                        material.base_color = if tile_component.is_pentagon {
                            Color::srgb(0.8, 0.3, 0.8)
                        } else {
                            Color::srgb(0.3, 0.7, 0.3)
                        };
                        material.emissive = LinearRgba::BLACK;
                    }
                    break;
                }
            }
        }
        
        // Highlight new tile
        if let Some(tile_index) = closest_tile {
            for (tile_component, _, material_handle) in tile_query.iter() {
                if tile_component.index == tile_index {
                    if let Some(material) = materials.get_mut(material_handle) {
                        material.base_color = Color::srgb(1.0, 1.0, 0.3);
                        material.emissive = LinearRgba::rgb(0.5, 0.5, 0.0);
                    }
                    
                    println!("Hovering {} tile {}", 
                        if tile_component.is_pentagon { "pentagon" } else { "hexagon" },
                        tile_index
                    );
                    break;
                }
            }
        }
        
        *hovered_tile = closest_tile;
    }
}