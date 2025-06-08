//! Ray casting system for tile picking in Bevy 0.16.1

use bevy::{
    picking::backend::ray::RayMap,
    prelude::*,
};
use crate::game_tiles::GameTileSystem;

/// Resource to track the currently hovered triangle  
#[derive(Resource, Default)]
pub struct HoveredTriangle {
    pub triangle_index: Option<usize>,
    pub hit_point: Option<Vec3>,
    pub hit_normal: Option<Vec3>,
}

/// Component to store triangle data for identification
#[derive(Component)]
pub struct IcosphereTriangles {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
}

/// System to visualize tiles and handle mouse interaction
pub fn simple_tile_visualization(
    mut gizmos: Gizmos,
    tile_system: Option<ResMut<GameTileSystem>>,
    mut ray_cast: MeshRayCast,
    ray_map: Res<RayMap>,
    mut hovered_triangle: ResMut<HoveredTriangle>,
    border_visibility: Res<crate::world::BorderVisibility>,
) {
    let Some(mut tile_system) = tile_system else {
        return;
    };

    // Cast rays from cursor and find intersections
    let mut current_hovered_tile = None;
    
    for (_, ray) in ray_map.iter() {
        // Cast ray against the mesh
        if let Some((_entity, hit)) = ray_cast
            .cast_ray(*ray, &MeshRayCastSettings::default())
            .first()
        {
            // Update hovered triangle resource
            hovered_triangle.hit_point = Some(hit.point);
            hovered_triangle.hit_normal = Some(hit.normal);
            
            // Simple approach: just pick the first tile for now
            if tile_system.tiles.len() > 0 {
                let triangle_idx = 0; // Simplified
                hovered_triangle.triangle_index = Some(triangle_idx);
                
                // Map triangle to tile
                if let Some(tile_idx) = tile_system.get_tile_from_triangle(triangle_idx) {
                    current_hovered_tile = Some(tile_idx);
                }
            }
            
            // Draw hit point
            gizmos.sphere(hit.point, 0.02, Color::srgb(1.0, 1.0, 1.0));
            
            // Draw normal at hit point
            let normal_end = hit.point + hit.normal * 0.1;
            gizmos.line(hit.point, normal_end, Color::srgb(0.0, 1.0, 1.0));
        }
    }
    
    // Update hovered tile in the system
    tile_system.set_hovered_tile(current_hovered_tile);

    // Draw all tile centers with appropriate colors
    for (tile_idx, tile) in tile_system.tiles.iter().enumerate() {
        let center = tile_system.get_tile_center(tile_idx).unwrap_or(Vec3::ZERO);
        let color = tile.get_display_color();
        
        let radius = match tile.face_type {
            crate::goldberg_polyhedron::FaceType::Pentagon(_) => 0.08,
            crate::goldberg_polyhedron::FaceType::Hexagon(_) => 0.05,
        };
        
        // Draw center sphere
        gizmos.sphere(center, radius, color);
        
        // Draw tile borders if enabled
        if border_visibility.show_borders {
            draw_tile_border(&mut gizmos, &tile_system, tile_idx);
        }
        
        // Draw extra highlight for hovered tile
        if Some(tile_idx) == current_hovered_tile {
            gizmos.sphere(center, radius * 1.3, Color::srgb(1.0, 1.0, 1.0));
        }
    }
    
    // Debug: Show ray directions
    for (_, ray) in ray_map.iter() {
        let end_point = ray.origin + ray.direction * 3.0;
        gizmos.line(ray.origin, end_point, Color::srgb(0.0, 1.0, 0.0));
    }
}

/// Draw borders around pentagon and hexagon tiles
fn draw_tile_border(gizmos: &mut Gizmos, tile_system: &GameTileSystem, tile_idx: usize) {
    let tile = tile_system.get_tile(tile_idx).unwrap();
    
    match tile.face_type {
        crate::goldberg_polyhedron::FaceType::Pentagon(pentagon_idx) => {
            draw_pentagon_border(gizmos, &tile_system.polyhedron, pentagon_idx);
        }
        crate::goldberg_polyhedron::FaceType::Hexagon(hexagon_idx) => {
            draw_hexagon_border(gizmos, &tile_system.polyhedron, hexagon_idx);
        }
    }
}

/// Draw a pentagon border
fn draw_pentagon_border(gizmos: &mut Gizmos, polyhedron: &crate::goldberg_polyhedron::GoldbergPolyhedron, pentagon_idx: usize) {
    if let Some(pentagon) = polyhedron.pentagons.get(pentagon_idx) {
        let center = pentagon.center;
        let normal = pentagon.normal;
        let radius = 0.3; // Pentagon border radius
        
        // Create 5 vertices around the pentagon center
        let vertices = create_pentagon_vertices(center, normal, radius);
        
        // Draw pentagon outline
        let border_color = Color::srgb(1.0, 0.2, 0.2); // Red border for pentagons
        for i in 0..5 {
            let start = vertices[i];
            let end = vertices[(i + 1) % 5];
            gizmos.line(start, end, border_color);
        }
    }
}

/// Draw a hexagon border
fn draw_hexagon_border(gizmos: &mut Gizmos, polyhedron: &crate::goldberg_polyhedron::GoldbergPolyhedron, hexagon_idx: usize) {
    if let Some(hexagon) = polyhedron.hexagons.get(hexagon_idx) {
        let center = hexagon.center;
        let normal = hexagon.normal;
        let radius = 0.25; // Hexagon border radius (slightly smaller than pentagon)
        
        // Create 6 vertices around the hexagon center
        let vertices = create_hexagon_vertices(center, normal, radius);
        
        // Draw hexagon outline
        let border_color = Color::srgb(0.2, 1.0, 0.2); // Green border for hexagons
        for i in 0..6 {
            let start = vertices[i];
            let end = vertices[(i + 1) % 6];
            gizmos.line(start, end, border_color);
        }
    }
}

/// Create pentagon vertices around a center point
fn create_pentagon_vertices(center: Vec3, normal: Vec3, radius: f32) -> [Vec3; 5] {
    let mut vertices = [Vec3::ZERO; 5];
    
    // Create a coordinate system on the sphere surface
    let up = Vec3::Y;
    let tangent1 = normal.cross(up).normalize();
    let tangent2 = normal.cross(tangent1).normalize();
    
    for i in 0..5 {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / 5.0;
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        // Create vertex in the plane of the pentagon
        let local_pos = tangent1 * cos_a * radius + tangent2 * sin_a * radius;
        let world_pos = center + local_pos;
        
        // Project back to sphere surface
        vertices[i] = world_pos.normalize() * center.length();
    }
    
    vertices
}

/// Create hexagon vertices around a center point
fn create_hexagon_vertices(center: Vec3, normal: Vec3, radius: f32) -> [Vec3; 6] {
    let mut vertices = [Vec3::ZERO; 6];
    
    // Create a coordinate system on the sphere surface
    let up = Vec3::Y;
    let tangent1 = normal.cross(up).normalize();
    let tangent2 = normal.cross(tangent1).normalize();
    
    for i in 0..6 {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / 6.0;
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        // Create vertex in the plane of the hexagon
        let local_pos = tangent1 * cos_a * radius + tangent2 * sin_a * radius;
        let world_pos = center + local_pos;
        
        // Project back to sphere surface
        vertices[i] = world_pos.normalize() * center.length();
    }
    
    vertices
}

/// System to handle tile selection with mouse clicks
pub fn handle_tile_selection(
    tile_system: Option<ResMut<GameTileSystem>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    hovered_triangle: Res<HoveredTriangle>,
) {
    let Some(mut tile_system) = tile_system else {
        return;
    };
    
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(triangle_idx) = hovered_triangle.triangle_index {
            if let Some(tile_idx) = tile_system.get_tile_from_triangle(triangle_idx) {
                // Toggle selection
                let new_selection = if tile_system.selected_tile == Some(tile_idx) {
                    None // Deselect if already selected
                } else {
                    Some(tile_idx) // Select this tile
                };
                
                tile_system.set_selected_tile(new_selection);
                
                if let Some(selected) = new_selection {
                    let tile = tile_system.get_tile(selected).unwrap();
                    println!("🎯 Selected tile {} ({:?})", selected, tile.face_type);
                } else {
                    println!("🎯 Deselected tile");
                }
            }
        }
    }
}

/// System to print information about the currently hovered tile
pub fn print_hovered_tile_info(
    keyboard: Res<ButtonInput<KeyCode>>,
    tile_system: Option<Res<GameTileSystem>>,
    hovered_triangle: Res<HoveredTriangle>,
) {
    let Some(tile_system) = tile_system else {
        return;
    };
    
    if keyboard.just_pressed(KeyCode::KeyH) {
        if let Some(hovered_tile_idx) = tile_system.hovered_tile {
            let tile = tile_system.get_tile(hovered_tile_idx).unwrap();
            let center = tile_system.get_tile_center(hovered_tile_idx).unwrap();
            
            println!("\n🎯 Hovered Tile Info:");
            println!("   Index: {}", hovered_tile_idx);
            println!("   Type: {:?}", tile.face_type);
            println!("   State: {:?}", tile.state);
            println!("   Center: {:.3}", center);
            println!("   Highlighted: {}", tile.highlighted);
            
            if let Some(hit_point) = hovered_triangle.hit_point {
                println!("   Hit point: {:.3}", hit_point);
                println!("   Distance to center: {:.3}", center.distance(hit_point));
            }
        } else {
            println!("🎯 No tile currently hovered");
        }
    }
}
