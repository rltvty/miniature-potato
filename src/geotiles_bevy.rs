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
    pub hovered_tile: Option<usize>,
    pub selected_tile: Option<usize>,
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
    
    let mut tile_entities = Vec::new();
    
    // Spawn tiles
    for (index, tile) in hexasphere.tiles.iter().enumerate() {
        let is_pentagon = tile.boundary.len() == 5;
        
        // Skip if we can't get orientation
        let Some(orientation) = tile.get_orientation() else {
            continue;
        };
        
        let center = Vec3::new(
            tile.center_point.x as f32,
            tile.center_point.y as f32,
            tile.center_point.z as f32,
        );
        
        // Calculate the outward normal (from sphere center to tile center)
        let normal = center.normalize();
        
        // Convert orientation to transform
        let transform = Transform {
            translation: center,
            rotation: orientation_to_quat(&orientation),
            scale: Vec3::ONE,
        };
        
        // Create individual material for each tile
        let material = materials.add(StandardMaterial {
            base_color: if is_pentagon {
                Color::srgb(0.8, 0.3, 0.8) // Magenta for pentagons
            } else {
                Color::srgb(0.3, 0.7, 0.3) // Green for hexagons
            },
            metallic: 0.2,
            perceptual_roughness: 0.5,
            cull_mode: Some(bevy::render::render_resource::Face::Back), // Enable back-face culling
            ..default()
        });
        
        // Create individual mesh with proper normal for each tile
        let sides = if is_pentagon { 5 } else { 6 };
        let radius = if is_pentagon { 
            uniform_radius as f32 * 0.9 
        } else { 
            uniform_radius as f32 
        };
        let mesh = create_regular_polygon_mesh_with_normal(sides, radius, normal);
        let mesh_handle = meshes.add(mesh);
        
        let entity = commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material),
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
        hovered_tile: None,
        selected_tile: None,
    });
    
    // Insert border visibility resource
    commands.insert_resource(BorderVisibility { show_borders: true });
    
    // Insert normal visibility resource
    commands.insert_resource(ShowNormals { show_normals: false });
    
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
        brightness: 0.1, // Reduced ambient light to improve contrast
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

/// Create a regular polygon mesh (hexagon or pentagon) with proper outward normals
fn create_regular_polygon_mesh_with_normal(sides: usize, radius: f32, normal: Vec3) -> Mesh {
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
    // All vertices get the same normal (pointing outward from sphere center)
    let normals: Vec<[f32; 3]> = vec![[normal.x, normal.y, normal.z]; vertices.len()];
    let uvs: Vec<[f32; 2]> = vertices.iter()
        .map(|v| [(v.x / radius + 1.0) * 0.5, (v.z / radius + 1.0) * 0.5])
        .collect();
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}

/// Create a regular polygon mesh (hexagon or pentagon) - legacy version
fn create_regular_polygon_mesh(sides: usize, radius: f32) -> Mesh {
    create_regular_polygon_mesh_with_normal(sides, radius, Vec3::Y)
}

/// System to handle tile hover with materials
pub fn tile_hover_system(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut hexasphere_res: ResMut<HexasphereResource>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window>,
    tile_query: Query<(&TileComponent, &GlobalTransform, &MeshMaterial3d<StandardMaterial>)>,
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
            
            if distance < closest_distance && distance < 1.0 {
                closest_distance = distance;
                closest_tile = Some(tile_component.index);
            }
        }
    }
    
    // Update hover state
    if closest_tile != hexasphere_res.hovered_tile {
        // Reset previous tile color
        if let Some(prev_index) = hexasphere_res.hovered_tile {
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
        
        hexasphere_res.hovered_tile = closest_tile;
    }
}

/// System to draw gizmos for tile visualization
pub fn tile_gizmos_system(
    mut gizmos: Gizmos,
    hexasphere_res: Res<HexasphereResource>,
    show_borders: Res<BorderVisibility>,
    show_normals: Res<ShowNormals>,
) {
    // Draw tile centers
    for (index, tile) in hexasphere_res.hexasphere.tiles.iter().enumerate() {
        let center = Vec3::new(
            tile.center_point.x as f32,
            tile.center_point.y as f32,
            tile.center_point.z as f32,
        );
        
        let is_pentagon = tile.boundary.len() == 5;
        let is_hovered = hexasphere_res.hovered_tile == Some(index);
        let is_selected = hexasphere_res.selected_tile == Some(index);
        
        // Choose color based on state
        let color = if is_selected {
            Color::srgb(1.0, 0.0, 0.0) // Red for selected
        } else if is_hovered {
            Color::srgb(1.0, 1.0, 0.0) // Yellow for hovered
        } else if is_pentagon {
            Color::srgb(0.8, 0.3, 0.8) // Magenta for pentagons
        } else {
            Color::srgb(0.3, 0.7, 0.3) // Green for hexagons
        };
        
        // Draw center sphere
        let radius = if is_pentagon { 0.05 } else { 0.03 };
        gizmos.sphere(center, radius, color);
        
        // Draw borders if enabled
        if show_borders.show_borders {
            draw_tile_border(&mut gizmos, tile, color);
        }
        
        // Draw selection highlight (larger sphere)
        if is_selected {
            let highlight_radius = if is_pentagon { 0.08 } else { 0.06 };
            gizmos.sphere(center, highlight_radius, Color::srgb(1.0, 1.0, 1.0));
        }
        
        // Draw normal vectors if enabled
        if show_normals.show_normals {
            let normal_end = center + center.normalize() * 0.2; // Normal pointing outward from sphere center
            gizmos.line(center, normal_end, Color::srgb(0.0, 1.0, 1.0)); // Cyan for normals
        }
    }
}

/// Resource to control border visibility
#[derive(Resource, Default)]
pub struct BorderVisibility {
    pub show_borders: bool,
}

/// Resource to control normal visualization
#[derive(Resource, Default)]
pub struct ShowNormals {
    pub show_normals: bool,
}

/// Draw borders around a tile
fn draw_tile_border(gizmos: &mut Gizmos, tile: &geotiles::Tile, color: Color) {
    let boundary_points: Vec<Vec3> = tile.boundary.iter()
        .map(|p| Vec3::new(p.x as f32, p.y as f32, p.z as f32))
        .collect();
    
    // Draw lines between boundary points
    for i in 0..boundary_points.len() {
        let start = boundary_points[i];
        let end = boundary_points[(i + 1) % boundary_points.len()];
        gizmos.line(start, end, color);
    }
}

/// System to toggle border visibility with 'B' key
pub fn toggle_borders(
    mut border_visibility: ResMut<BorderVisibility>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyB) {
        border_visibility.show_borders = !border_visibility.show_borders;
        println!("Border visibility: {}", if border_visibility.show_borders { "ON" } else { "OFF" });
    }
}

/// System to toggle normal visibility with 'N' key
pub fn toggle_normals(
    mut show_normals: ResMut<ShowNormals>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyN) {
        show_normals.show_normals = !show_normals.show_normals;
        println!("Normal visibility: {}", if show_normals.show_normals { "ON" } else { "OFF" });
    }
}