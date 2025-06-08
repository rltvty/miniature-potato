//! Hexagonal and pentagonal tile system for the icosphere

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

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
}

/// Component for visual tile representation
#[derive(Component)]
pub struct TileVisual {
    pub tile_index: usize,
    pub is_highlighted: bool,
}

/// Generate tiles from icosphere triangle data
pub fn generate_tiles_from_icosphere(
    vertices: &[Vec3],
    indices: &[u32],
    subdivisions: usize,
) -> TileMap {
    let mut tile_map = TileMap::default();
    
    // For now, let's start simple: each triangle becomes a tile
    // Later we'll implement proper hex/pentagon clustering
    for (triangle_idx, triangle) in indices.chunks(3).enumerate() {
        let v0 = vertices[triangle[0] as usize];
        let v1 = vertices[triangle[1] as usize];
        let v2 = vertices[triangle[2] as usize];
        
        // Calculate triangle center (projected to sphere surface)
        let center = ((v0 + v1 + v2) / 3.0).normalize() * v0.length();
        
        // For now, all tiles are hexagons (we'll identify pentagons later)
        let tile = Tile {
            tile_type: TileType::Hexagon,
            center,
            neighbors: Vec::new(), // Will compute later
            triangle_indices: vec![triangle_idx],
        };
        
        tile_map.tiles.push(tile);
        tile_map.triangle_to_tile.insert(triangle_idx, triangle_idx);
    }
    
    // TODO: Identify the 12 pentagon positions
    // TODO: Cluster triangles around pentagon/hexagon centers
    // TODO: Compute neighbor relationships
    
    println!("Generated {} tiles from icosphere", tile_map.tiles.len());
    tile_map
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
                println!("Selected {:?} tile at position {:.2}", 
                    tile.tile_type, tile.center);
            }
        }
    } else {
        *selected_tile = None;
    }
}

/// System to visualize tile boundaries
pub fn visualize_tiles(
    mut gizmos: Gizmos,
    tile_map: Res<TileMap>,
    hovered_triangle: Res<crate::ray_casting::HoveredTriangle>,
) {
    // Draw all tile centers as small dots
    for (tile_idx, tile) in tile_map.tiles.iter().enumerate() {
        let color = match tile.tile_type {
            TileType::Hexagon => Color::srgb(0.0, 1.0, 0.0), // Green for hexagons
            TileType::Pentagon => Color::srgb(1.0, 0.0, 1.0), // Magenta for pentagons
        };
        gizmos.sphere(tile.center, 0.02, color);
    }
    
    // Highlight the currently hovered tile
    if let Some(triangle_idx) = hovered_triangle.triangle_index {
        if let Some(&tile_idx) = tile_map.triangle_to_tile.get(&triangle_idx) {
            let tile = &tile_map.tiles[tile_idx];
            gizmos.sphere(tile.center, 0.05, Color::srgb(1.0, 1.0, 0.0)); // Yellow highlight
        }
    }
}
