//! Ray casting system for tile picking in Bevy 0.16.1

use bevy::{
    picking::backend::ray::RayMap,
    prelude::*,
};
use crate::goldberg_tiles::{TileMap, TileType};

/// Resource to track the currently hovered triangle  
#[derive(Resource, Default)]
pub struct HoveredTriangle {
    pub triangle_index: Option<usize>,
    pub hit_point: Option<Vec3>,
    pub hit_normal: Option<Vec3>,
}

/// Component to store icosphere triangle data for identification
#[derive(Component)]
pub struct IcosphereTriangles {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
}

/// Simple system to visualize all tile centers and provide basic cursor feedback  
pub fn simple_tile_visualization(
    mut gizmos: Gizmos,
    tile_map: Option<Res<TileMap>>,
    ray_map: Res<RayMap>,
) {
    let Some(tile_map) = tile_map else {
        return;
    };

    // Draw all tile centers
    for tile in tile_map.tiles.iter() {
        let (color, radius) = match tile.tile_type {
            TileType::Pentagon => (Color::srgb(1.0, 0.0, 1.0), 0.08), // Magenta
            TileType::Hexagon => (Color::srgb(1.0, 1.0, 0.0), 0.05),  // Yellow
        };
        
        gizmos.sphere(tile.center, radius, color);
    }
    
    // Show ray direction as simple indicator
    for (_, ray) in ray_map.iter() {
        let end_point = ray.origin + ray.direction * 10.0;
        gizmos.line(ray.origin, end_point, Color::srgb(0.0, 1.0, 0.0));
    }
}
