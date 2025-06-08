//! Hexagonal and pentagonal tile system for the icosphere

use bevy::prelude::*;
use std::collections::HashMap;

/// Represents the type of tile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Hexagon,
    Pentagon,
}

/// A tile on the spherical surface
#[derive(Debug, Clone)]
pub struct Tile {
    pub tile_type: TileType,
    pub center: Vec3,
    pub neighbors: Vec<usize>, // Indices of neighboring tiles
    pub triangle_indices: Vec<usize>, // Triangle faces that belong to this tile
}

/// Resource containing all tiles on the sphere
#[derive(Resource, Default)]
pub struct TileMap {
    pub tiles: Vec<Tile>,
    pub triangle_to_tile: HashMap<usize, usize>, // Maps triangle index to tile index
    pub pentagon_indices: Vec<usize>, // Indices of pentagon tiles
    pub hexagon_indices: Vec<usize>, // Indices of hexagon tiles
}

/// Component for visual tile representation
#[derive(Component)]
pub struct TileVisual {
    pub tile_index: usize,
    pub is_highlighted: bool,
}

/// Generate tiles based on vertex connectivity - proper hex/pentagon identification
pub fn generate_tiles_from_icosphere(
    vertices: &[Vec3],
    indices: &[u32],
    _subdivisions: usize, // Parameter kept for future use
    radius: f32,
) -> TileMap {
    let mut tile_map = TileMap::default();
    
    println!("Analyzing vertex connectivity for pentagon/hexagon identification...");
    
    // Build vertex connectivity map
    let vertex_connections = build_vertex_connectivity(indices);
    
    // Find pentagon centers (vertices with exactly 5 connections)
    let mut pentagon_centers = Vec::new();
    
    for (vertex_idx, connections) in vertex_connections.iter().enumerate() {
        if connections.len() == 5 {
            pentagon_centers.push(vertices[vertex_idx]);
        }
    }
    
    // Find hexagon centers (vertices with exactly 6 connections)
    let mut hexagon_centers = Vec::new();
    
    for (vertex_idx, connections) in vertex_connections.iter().enumerate() {
        if connections.len() == 6 {
            hexagon_centers.push(vertices[vertex_idx]);
        }
    }
    
    println!("Found {} pentagon centers (5-connected vertices)", pentagon_centers.len());
    println!("Found {} hexagon centers (6-connected vertices)", hexagon_centers.len());
    
    // Initialize triangle collections for each tile
    let mut pentagon_triangles: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut hexagon_triangles: HashMap<usize, Vec<usize>> = HashMap::new();
    
    for i in 0..pentagon_centers.len() {
        pentagon_triangles.insert(i, Vec::new());
    }
    for i in 0..hexagon_centers.len() {
        hexagon_triangles.insert(i, Vec::new());
    }
    
    // For each triangle, find which tile center it's closest to
    for (triangle_idx, triangle) in indices.chunks(3).enumerate() {
        let v0 = vertices[triangle[0] as usize];
        let v1 = vertices[triangle[1] as usize];
        let v2 = vertices[triangle[2] as usize];
        
        // Calculate triangle center (projected to sphere surface)
        let center = ((v0 + v1 + v2) / 3.0).normalize() * radius;
        
        // Find closest tile center
        let (tile_type, tile_idx) = find_closest_tile_center(center, &pentagon_centers, &hexagon_centers);
        
        match tile_type {
            TileType::Pentagon => {
                pentagon_triangles.get_mut(&tile_idx).unwrap().push(triangle_idx);
            }
            TileType::Hexagon => {
                hexagon_triangles.get_mut(&tile_idx).unwrap().push(triangle_idx);
            }
        }
    }
    
    // Create pentagon tiles
    for (pentagon_idx, &pentagon_center) in pentagon_centers.iter().enumerate() {
        let triangle_indices = pentagon_triangles.get(&pentagon_idx).unwrap().clone();
        
        if !triangle_indices.is_empty() {
            let tile = Tile {
                tile_type: TileType::Pentagon,
                center: pentagon_center,
                neighbors: Vec::new(),
                triangle_indices: triangle_indices.clone(),
            };
            
            let tile_idx = tile_map.tiles.len();
            tile_map.pentagon_indices.push(tile_idx);
            tile_map.tiles.push(tile);
            
            for triangle_idx in triangle_indices {
                tile_map.triangle_to_tile.insert(triangle_idx, tile_idx);
            }
        }
    }
    
    // Create hexagon tiles
    for (hexagon_idx, &hexagon_center) in hexagon_centers.iter().enumerate() {
        let triangle_indices = hexagon_triangles.get(&hexagon_idx).unwrap().clone();
        
        if !triangle_indices.is_empty() {
            let tile = Tile {
                tile_type: TileType::Hexagon,
                center: hexagon_center,
                neighbors: Vec::new(),
                triangle_indices: triangle_indices.clone(),
            };
            
            let tile_idx = tile_map.tiles.len();
            tile_map.hexagon_indices.push(tile_idx);
            tile_map.tiles.push(tile);
            
            for triangle_idx in triangle_indices {
                tile_map.triangle_to_tile.insert(triangle_idx, tile_idx);
            }
        }
    }
    
    println!("Generated {} pentagon tiles", tile_map.pentagon_indices.len());
    println!("Generated {} hexagon tiles", tile_map.hexagon_indices.len());
    println!("Total tiles: {}", tile_map.tiles.len());
    println!("Total triangles assigned: {}", tile_map.triangle_to_tile.len());
    println!("Total triangles in mesh: {}", indices.len() / 3);
    
    // Debug analysis
    println!("\n=== TILE ANALYSIS ===");
    let mut pentagon_triangle_counts = Vec::new();
    let mut hexagon_triangle_counts = Vec::new();
    
    for (i, tile) in tile_map.tiles.iter().enumerate() {
        let count = tile.triangle_indices.len();
        match tile.tile_type {
            TileType::Pentagon => {
                pentagon_triangle_counts.push(count);
                println!("Pentagon {}: {} triangles", i, count);
            }
            TileType::Hexagon => {
                hexagon_triangle_counts.push(count);
                println!("Hexagon {}: {} triangles", i, count);
            }
        }
    }
    
    if !pentagon_triangle_counts.is_empty() {
        let avg_pentagon = pentagon_triangle_counts.iter().sum::<usize>() as f32 / pentagon_triangle_counts.len() as f32;
        println!("Pentagon triangles - Min: {}, Max: {}, Avg: {:.1}", 
            pentagon_triangle_counts.iter().min().unwrap_or(&0),
            pentagon_triangle_counts.iter().max().unwrap_or(&0),
            avg_pentagon);
    }
    
    if !hexagon_triangle_counts.is_empty() {
        let avg_hexagon = hexagon_triangle_counts.iter().sum::<usize>() as f32 / hexagon_triangle_counts.len() as f32;
        println!("Hexagon triangles - Min: {}, Max: {}, Avg: {:.1}", 
            hexagon_triangle_counts.iter().min().unwrap_or(&0),
            hexagon_triangle_counts.iter().max().unwrap_or(&0),
            avg_hexagon);
    }
    
    println!("Expected: Always 12 pentagons, variable hexagons based on subdivision level");
    println!("======================\n");
    
    tile_map
}

/// Build a map of vertex connectivity
fn build_vertex_connectivity(indices: &[u32]) -> Vec<std::collections::HashSet<usize>> {
    let mut connections: Vec<std::collections::HashSet<usize>> = Vec::new();
    
    let max_vertex = indices.iter().max().unwrap_or(&0);
    connections.resize((*max_vertex as usize) + 1, std::collections::HashSet::new());
    
    for triangle in indices.chunks(3) {
        let v0 = triangle[0] as usize;
        let v1 = triangle[1] as usize;
        let v2 = triangle[2] as usize;
        
        connections[v0].insert(v1);
        connections[v0].insert(v2);
        connections[v1].insert(v0);
        connections[v1].insert(v2);
        connections[v2].insert(v0);
        connections[v2].insert(v1);
    }
    
    connections
}

/// Find the closest tile center (pentagon or hexagon) to a given point
fn find_closest_tile_center(point: Vec3, pentagon_centers: &[Vec3], hexagon_centers: &[Vec3]) -> (TileType, usize) {
    let mut min_distance = f32::INFINITY;
    let mut closest_type = TileType::Pentagon;
    let mut closest_idx = 0;
    
    for (idx, &center) in pentagon_centers.iter().enumerate() {
        let distance = (point - center).length();
        if distance < min_distance {
            min_distance = distance;
            closest_type = TileType::Pentagon;
            closest_idx = idx;
        }
    }
    
    for (idx, &center) in hexagon_centers.iter().enumerate() {
        let distance = (point - center).length();
        if distance < min_distance {
            min_distance = distance;
            closest_type = TileType::Hexagon;
            closest_idx = idx;
        }
    }
    
    (closest_type, closest_idx)
}

/// System to handle tile selection from ray casting
pub fn tile_selection_system(
    hovered_triangle: Res<crate::ray_casting::HoveredTriangle>,
    tile_map: Res<TileMap>,
    mut selected_tile: Local<Option<usize>>,
) {
    if let Some(triangle_idx) = hovered_triangle.triangle_index {
        if let Some(&tile_idx) = tile_map.triangle_to_tile.get(&triangle_idx) {
            if *selected_tile != Some(tile_idx) {
                *selected_tile = Some(tile_idx);
                let tile = &tile_map.tiles[tile_idx];
                println!("Selected {:?} tile at position {:.2} (covers {} triangles)", 
                    tile.tile_type, tile.center, tile.triangle_indices.len());
            }
        }
    } else {
        *selected_tile = None;
    }
}

/// System to visualize tile boundaries with pentagon and hexagon highlighting
pub fn visualize_tiles(
    mut gizmos: Gizmos,
    tile_map: Res<TileMap>,
    hovered_triangle: Res<crate::ray_casting::HoveredTriangle>,
    mut last_hovered_tile: Local<Option<usize>>, // Track last hovered tile to prevent spam
) {
    // Draw all tile centers with different colors and sizes
    for tile in tile_map.tiles.iter() {
        let (color, size) = match tile.tile_type {
            TileType::Hexagon => (Color::srgb(0.0, 1.0, 0.0), 0.03), // Green, medium
            TileType::Pentagon => (Color::srgb(1.0, 0.0, 1.0), 0.05), // Magenta, larger
        };
        gizmos.sphere(tile.center, size, color);
    }
    
    // Highlight the currently hovered tile
    if let Some(triangle_idx) = hovered_triangle.triangle_index {
        if let Some(&tile_idx) = tile_map.triangle_to_tile.get(&triangle_idx) {
            let tile = &tile_map.tiles[tile_idx];
            let highlight_size = match tile.tile_type {
                TileType::Pentagon => 0.10, // Larger highlight for pentagons
                TileType::Hexagon => 0.07,  // Medium for hexagons
            };
            gizmos.sphere(tile.center, highlight_size, Color::srgb(1.0, 1.0, 0.0)); // Yellow highlight
            
            // Only print debug info when we hover a new tile (not continuously)
            if *last_hovered_tile != Some(tile_idx) {
                *last_hovered_tile = Some(tile_idx);
                println!("Hovered {:?} {} covers {} triangles", 
                    tile.tile_type, tile_idx, tile.triangle_indices.len());
            }
            
            // Draw different shapes around tile types when hovered
            match tile.tile_type {
                TileType::Pentagon => {
                    draw_circle_around_tile(&mut gizmos, tile.center, 0.18, Color::srgb(1.0, 0.8, 0.0)); // Orange circle
                }
                TileType::Hexagon => {
                    draw_hexagon_around_tile(&mut gizmos, tile.center, 0.15, Color::srgb(0.0, 1.0, 1.0)); // Cyan hexagon
                }
            }
        }
    } else {
        // Reset when not hovering any tile
        if last_hovered_tile.is_some() {
            *last_hovered_tile = None;
        }
    }
}

/// Helper function to draw a circle around a tile
fn draw_circle_around_tile(gizmos: &mut Gizmos, center: Vec3, radius: f32, color: Color) {
    let up = Vec3::Y;
    let right = center.cross(up).normalize();
    let forward = right.cross(center).normalize();
    
    for i in 0..32 {
        let angle1 = (i as f32 / 32.0) * 2.0 * std::f32::consts::PI;
        let angle2 = ((i + 1) as f32 / 32.0) * 2.0 * std::f32::consts::PI;
        
        let p1 = center + radius * (right * angle1.cos() + forward * angle1.sin());
        let p2 = center + radius * (right * angle2.cos() + forward * angle2.sin());
        
        gizmos.line(p1, p2, color);
    }
}

/// Helper function to draw a hexagon around a tile
fn draw_hexagon_around_tile(gizmos: &mut Gizmos, center: Vec3, radius: f32, color: Color) {
    let up = Vec3::Y;
    let right = center.cross(up).normalize();
    let forward = right.cross(center).normalize();
    
    for i in 0..6 {
        let angle1 = (i as f32 / 6.0) * 2.0 * std::f32::consts::PI;
        let angle2 = ((i + 1) as f32 / 6.0) * 2.0 * std::f32::consts::PI;
        
        let p1 = center + radius * (right * angle1.cos() + forward * angle1.sin());
        let p2 = center + radius * (right * angle2.cos() + forward * angle2.sin());
        
        gizmos.line(p1, p2, color);
    }
}
