//! Bevy integration for geotiles following the example from the README

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use geotiles::{Hexasphere, Point, RegularHexagonParams, ThickTile, Vector3};
use std::f32::consts::PI;

// Configuration
const SPHERE_RADIUS: f64 = 5.0;
const SUBDIVISIONS: usize = 10;
const TILE_SIZE: f64 = 0.99;
const TILE_THICKNESS: f64 = 0.1;
const USE_UNIFORM_TILES: bool = false;
const USE_THICK_TILES: bool = false;

/// Resource to store the hexasphere and related data
#[derive(Resource)]
pub struct HexasphereResource {
    pub hexasphere: Hexasphere,
    pub thick_tiles: Vec<ThickTile>,
    pub uniform_radius: f64,
    pub tile_entities: Vec<Entity>,
    pub hovered_tile: Option<usize>,
    pub selected_tile: Option<usize>,
}

/// Component to mark tile entities
#[derive(Component)]
pub struct TileComponent {
    pub index: usize,
    pub is_hexagon: bool,
}

fn vec3_from_point(p: &Point) -> Vec3 {
    Vec3::new(p.x as f32, p.y as f32, p.z as f32)
}

fn vec3_from_vector3(v: &Vector3) -> Vec3 {
    Vec3::new(v.x as f32, v.y as f32, v.z as f32)
}

fn transform_from_hexagon_params(hex_params: &RegularHexagonParams) -> Transform {
    let translation = vec3_from_point(&hex_params.center);
    let right = vec3_from_vector3(&hex_params.orientation.right).normalize();
    let up = vec3_from_vector3(&hex_params.orientation.up).normalize();
    let forward = vec3_from_vector3(&hex_params.orientation.forward).normalize();
    
    let matrix = Mat4::from_cols(
        Vec4::new(forward.x, forward.y, forward.z, 0.0), 
        Vec4::new(right.x, right.y, right.z, 0.0),  
        Vec4::new(up.x, up.y, up.z, 0.0),
        Vec4::new(translation.x, translation.y, translation.z, 1.0)
    );
    
    Transform::from_matrix(matrix)
}

fn get_color(is_hexagon: bool) -> Color {
    if is_hexagon {
        Color::srgb(0.4, 1.0, 0.4) // Brighter green for hexagons
    } else {
        Color::srgb(1.0, 0.4, 1.0) // Brighter magenta for pentagons
    }
}

fn get_material(materials: &mut Assets<StandardMaterial>, is_hexagon: bool) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: get_color(is_hexagon),
        metallic: 0.1, // Less metallic for brighter appearance
        perceptual_roughness: 0.8, // More rough for better light scattering
        cull_mode: Some(bevy::render::render_resource::Face::Back), // Enable back-face culling
        ..default()
    })
}

fn add_entity(commands: &mut Commands, meshes: &mut Assets<Mesh>, tile_entities: &mut Vec<Entity>, mesh: Mesh, material: Handle<StandardMaterial>, transform: Transform, tile_component: TileComponent) {
    let mesh_handle = meshes.add(mesh);

    let entity = commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(material),
        transform,
        tile_component,
    )).id();
    
    tile_entities.push(entity);
}

/// Setup the hexasphere world
pub fn setup_hexasphere_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("🌍 Setting up hexasphere world with geotiles...");
    
    // Create hexasphere and thick tiles
    let hexasphere = Hexasphere::new(SPHERE_RADIUS, SUBDIVISIONS, TILE_SIZE);
    let uniform_radius = hexasphere.get_uniform_hexagon_radius();
    
    
    println!("Generated {} tiles", hexasphere.tiles.len());
    println!("Uniform hexagon radius: {:.3}", uniform_radius);
    
    let mut tile_entities: Vec<Entity> = Vec::new();
    let mut thick_tiles: Vec<ThickTile> = Vec::new();

    if USE_UNIFORM_TILES {
        let approximations = hexasphere.get_regular_hexagon_approximations();
        println!("Generated {} approximation tiles", approximations.len());

        for (index, hex_params) in approximations.iter().enumerate() {
            let transform = transform_from_hexagon_params(&hex_params);            
            let material = get_material(&mut materials, true);
            let mesh = Extrusion::new(RegularPolygon::new(hex_params.radius as f32, 6), TILE_THICKNESS as f32);
            let tile_component = TileComponent { index, is_hexagon: true };
            
            add_entity(&mut commands, &mut meshes, &mut tile_entities, mesh.into(), material, transform, tile_component);
        } 
    } else {
        thick_tiles = hexasphere.create_thick_tiles(TILE_THICKNESS); // 0.2 units thickness
        println!("Generated {} thick tiles", thick_tiles.len());

        // Spawn thick tiles
        for (index, thick_tile) in thick_tiles.iter().enumerate() {
            // Don't apply additional transform - thick tile vertices are already in world coordinates
            let transform = Transform::IDENTITY;
            let material = get_material(&mut materials, thick_tile.is_hexagon);
            let mesh = create_thick_tile_mesh(thick_tile); // Create mesh from thick tile vertices
            let tile_component = TileComponent { index, is_hexagon: thick_tile.is_hexagon };
            
            add_entity(&mut commands, &mut meshes, &mut tile_entities, mesh.into(), material, transform, tile_component);
        }
    }
    
    // Store the hexasphere resource
    commands.insert_resource(HexasphereResource {
        hexasphere,
        thick_tiles,
        uniform_radius,
        tile_entities: tile_entities.clone(),
        hovered_tile: None,
        selected_tile: None,
    });
    let tile_count = tile_entities.len();
    println!("✅ Hexasphere world setup complete with {} tiles!", tile_count);

    // Insert border visibility resource - off by default, use wireframe mode instead
    commands.insert_resource(BorderVisibility { show_borders: false });
    
    // Insert normal visibility resource
    commands.insert_resource(ShowNormals { show_normals: false });
    
}


/// System to handle tile hover with materials using improved distance-based detection
pub fn tile_hover_system(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut hexasphere_res: ResMut<HexasphereResource>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window>,
    tile_query: Query<(Entity, &TileComponent, &MeshMaterial3d<StandardMaterial>, &GlobalTransform)>,
) {
    if USE_UNIFORM_TILES {
        return;
    }
    let Ok(window) = window_query.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { return };
    
    // Find closest tile to ray using improved distance calculation
    let mut closest_tile = None;
    let mut closest_distance = f32::MAX;
    
    for (_entity, tile_component, _, _tile_transform) in tile_query.iter() {
        // Get the tile center from the thick tile data and apply sphere rotation
        let thick_tile = &hexasphere_res.thick_tiles[tile_component.index];
        let original_center = vec3_from_point(&thick_tile.center_point);
        
        // Apply the current sphere rotation to get the actual world position
        let rotated_center =  original_center;
        
        // Calculate closest point on ray to tile center
        let to_center = rotated_center - ray.origin;
        let proj_length = to_center.dot(*ray.direction).max(0.0);
        let closest_point_on_ray = ray.origin + ray.direction * proj_length;
        let distance_to_center = (closest_point_on_ray - rotated_center).length();
        
        // Use a reasonable detection radius based on tile size
        let detection_radius = if thick_tile.is_hexagon { 0.8 } else { 0.7 }; // Pentagon slightly smaller
        
        if distance_to_center < detection_radius && proj_length < closest_distance {
            closest_distance = proj_length;
            closest_tile = Some(tile_component.index);
        }
    }
    
    // Update hover state
    if closest_tile != hexasphere_res.hovered_tile {
        // Reset previous tile color
        if let Some(prev_index) = hexasphere_res.hovered_tile {
            for (_, tile_component, material_handle, _) in tile_query.iter() {
                if tile_component.index == prev_index {
                    if let Some(material) = materials.get_mut(&material_handle.0) {
                        material.base_color = get_color(tile_component.is_hexagon);
                        material.emissive = LinearRgba::BLACK;
                    }
                    break;
                }
            }
        }
        
        // Highlight new tile
        if let Some(tile_index) = closest_tile {
            for (_, tile_component, material_handle, _) in tile_query.iter() {
                if tile_component.index == tile_index {
                    if let Some(material) = materials.get_mut(&material_handle.0) {
                        material.base_color = Color::srgb(1.0, 1.0, 0.3);
                        material.emissive = LinearRgba::rgb(0.5, 0.5, 0.0);
                    }
                    break;
                }
            }
        }
        
        hexasphere_res.hovered_tile = closest_tile;
    }
}

/// System to handle tile selection with mouse clicks
pub fn handle_tile_selection(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut hexasphere_res: ResMut<HexasphereResource>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(hovered_index) = hexasphere_res.hovered_tile {
            // Toggle selection
            if hexasphere_res.selected_tile == Some(hovered_index) {
                // Deselect
                hexasphere_res.selected_tile = None;
                println!("🎯 Deselected tile");
            } else {
                // Select
                hexasphere_res.selected_tile = Some(hovered_index);
                if let Some(tile) = hexasphere_res.hexasphere.tiles.get(hovered_index) {
                    println!("🎯 Selected {} tile {}", 
                        if tile.boundary.len() == 5 { "pentagon" } else { "hexagon" },
                        hovered_index
                    );
                }
            }
        }
    }
}

/// System to draw gizmos for tile visualization
pub fn tile_gizmos_system(
    mut gizmos: Gizmos,
    hexasphere_res: Res<HexasphereResource>,
    show_normals: Res<ShowNormals>,
) {
    if USE_UNIFORM_TILES {
        return;
    }
    // Only draw normals and selection highlights, not borders (borders are handled by wireframe mode)
    for (index, thick_tile) in hexasphere_res.thick_tiles.iter().enumerate() {
        let original_center = vec3_from_point(&thick_tile.center_point);
        
        // Apply sphere rotation to get current world position
        let rotated_center = original_center;
        
        let is_selected = hexasphere_res.selected_tile == Some(index);
        
        // Draw selection highlight as a wireframe outline with rotation applied
        if is_selected {
            draw_thick_tile_border_rotated(&mut gizmos, thick_tile, Color::srgb(1.0, 1.0, 1.0), Quat::IDENTITY);
        }
        
        // Draw normal vectors if enabled
        if show_normals.show_normals {
            let normal_end = rotated_center + rotated_center.normalize() * 0.2; // Normal pointing outward from sphere center
            gizmos.line(rotated_center, normal_end, Color::srgb(0.0, 1.0, 1.0)); // Cyan for normals
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


/// Draw borders around a thick tile with rotation applied
fn draw_thick_tile_border_rotated(gizmos: &mut Gizmos, thick_tile: &ThickTile, color: Color, rotation: Quat) {
    let boundary_points: Vec<Vec3> = thick_tile.outer_boundary.iter()
        .map(|p| {
            let original_point = Vec3::new(p.x as f32, p.y as f32, p.z as f32);
            rotation * original_point // Apply rotation
        })
        .collect();
    
    // Draw lines between rotated boundary points
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

/// Create a Bevy mesh from a ThickTile - creates a proper flat extruded tile
fn create_thick_tile_mesh(thick_tile: &ThickTile) -> Mesh {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    // Calculate surface normal (pointing outward from sphere center)
    let center = vec3_from_point(&thick_tile.center_point);
    let outward_normal = center.normalize();
    
    // Create flat tile by triangulating the outer boundary (no center point)
    let outer_boundary: Vec<Vec3> = thick_tile.outer_boundary.iter()
        .map(|p| Vec3::new(p.x as f32, p.y as f32, p.z as f32))
        .collect();

        // Add outer boundary vertices with outward normals
    for vertex in &outer_boundary {
        vertices.push([vertex.x, vertex.y, vertex.z]);
        normals.push([outward_normal.x, outward_normal.y, outward_normal.z]);
    }
    
    // Create outer face triangles (flat triangulation)
    let boundary_len = outer_boundary.len();
    for i in 1..boundary_len - 1 {
        indices.extend_from_slice(&[0, i as u32, (i + 1) as u32]);
    }
    

    if USE_THICK_TILES {
        let inward_normal = -outward_normal;
        
        let inner_boundary: Vec<Vec3> = thick_tile.inner_boundary.iter()
            .map(|p| Vec3::new(p.x as f32, p.y as f32, p.z as f32))
            .collect();

        // Add inner boundary vertices with inward normals
        for vertex in &inner_boundary {
            vertices.push([vertex.x, vertex.y, vertex.z]);
            normals.push([inward_normal.x, inward_normal.y, inward_normal.z]);
        }

        // Create inner face triangles (reverse winding for inward normal)
        for i in 1..boundary_len - 1 {
            let base_idx = boundary_len as u32;
            indices.extend_from_slice(&[
                base_idx, 
                base_idx + (i + 1) as u32, 
                base_idx + i as u32
            ]);
        }

        // Add vertices for side walls with proper normals
        let side_vertex_start = vertices.len();
        for i in 0..boundary_len {
            let next_i = (i + 1) % boundary_len;
            
            // Calculate edge normal for side wall
            let edge = outer_boundary[next_i] - outer_boundary[i];
            let side_normal = outward_normal.cross(edge).normalize();
            
            // Add vertices for this side wall (4 vertices per wall)
            vertices.push([outer_boundary[i].x, outer_boundary[i].y, outer_boundary[i].z]);
            normals.push([side_normal.x, side_normal.y, side_normal.z]);
            
            vertices.push([inner_boundary[i].x, inner_boundary[i].y, inner_boundary[i].z]);
            normals.push([side_normal.x, side_normal.y, side_normal.z]);
            
            vertices.push([outer_boundary[next_i].x, outer_boundary[next_i].y, outer_boundary[next_i].z]);
            normals.push([side_normal.x, side_normal.y, side_normal.z]);
            
            vertices.push([inner_boundary[next_i].x, inner_boundary[next_i].y, inner_boundary[next_i].z]);
            normals.push([side_normal.x, side_normal.y, side_normal.z]);
            
            // Create two triangles for this side wall
            let base = (side_vertex_start + i * 4) as u32;
            indices.extend_from_slice(&[base, base + 1, base + 2]);
            indices.extend_from_slice(&[base + 2, base + 1, base + 3]);
        }
    }


    // Generate UV coordinates
    let uvs: Vec<[f32; 2]> = vertices.iter()
        .map(|pos| {
            let normalized = Vec3::from(*pos).normalize();
            let u = 0.5 + normalized.z.atan2(normalized.x) / (2.0 * PI);
            let v = 0.5 - normalized.y.asin() / PI;
            [u, v]
        })
        .collect();
    
    // Create and populate the mesh
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::all(),
    );
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}