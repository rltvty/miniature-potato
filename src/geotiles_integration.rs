//! Integration layer between geotiles crate and our game systems

use bevy::prelude::*;
use geotiles::Hexasphere;
use std::collections::HashMap;

/// Wrapper around geotiles Hexasphere for game integration
#[derive(Resource)]
pub struct GeotilesWorld {
    pub hexasphere: Hexasphere,
    /// Mapping from our entity IDs to geotiles tile indices
    pub entity_to_tile: HashMap<Entity, usize>,
    /// Mapping from geotiles tile indices to our entities
    pub tile_to_entity: HashMap<usize, Entity>,
    /// Currently hovered tile
    pub hovered_tile: Option<usize>,
    /// Currently selected tile
    pub selected_tile: Option<usize>,
}

impl GeotilesWorld {
    pub fn new(radius: f32, subdivisions: u8, tile_size: f32) -> Self {
        let hexasphere = Hexasphere::new(radius as f64, subdivisions as usize, tile_size as f64);
        
        println!("Created Hexasphere with {} tiles", hexasphere.tiles.len());
        println!("  - Radius: {}", radius);
        println!("  - Subdivisions: {}", subdivisions);
        println!("  - Tile size factor: {}", tile_size);
        
        // Count pentagons and hexagons
        let pentagon_count = hexasphere.tiles.iter()
            .filter(|t| t.boundary.len() == 5)
            .count();
        let hexagon_count = hexasphere.tiles.iter()
            .filter(|t| t.boundary.len() == 6)
            .count();
            
        println!("  - {} pentagons, {} hexagons", pentagon_count, hexagon_count);
        
        Self {
            hexasphere,
            entity_to_tile: HashMap::new(),
            tile_to_entity: HashMap::new(),
            hovered_tile: None,
            selected_tile: None,
        }
    }
    
    /// Get the center position of a tile
    pub fn get_tile_center(&self, tile_index: usize) -> Option<Vec3> {
        self.hexasphere.tiles.get(tile_index)
            .map(|tile| Vec3::new(
                tile.center_point.x as f32,
                tile.center_point.y as f32,
                tile.center_point.z as f32,
            ))
    }
    
    /// Check if a tile is a pentagon (5 sides) or hexagon (6 sides)
    pub fn is_pentagon(&self, tile_index: usize) -> bool {
        self.hexasphere.tiles.get(tile_index)
            .map(|tile| tile.boundary.len() == 5)
            .unwrap_or(false)
    }
    
    /// Get the boundary points of a tile
    pub fn get_tile_boundary(&self, tile_index: usize) -> Option<Vec<Vec3>> {
        self.hexasphere.tiles.get(tile_index)
            .map(|tile| {
                tile.boundary.iter()
                    .map(|p| Vec3::new(p.x as f32, p.y as f32, p.z as f32))
                    .collect()
            })
    }
    
    /// Find the closest tile to a given point
    pub fn find_closest_tile(&self, point: Vec3) -> Option<usize> {
        let point_f64 = geotiles::Point {
            x: point.x as f64,
            y: point.y as f64,
            z: point.z as f64,
        };
        
        self.hexasphere.tiles.iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let dist_a = (a.center_point.x - point_f64.x).powi(2) +
                           (a.center_point.y - point_f64.y).powi(2) +
                           (a.center_point.z - point_f64.z).powi(2);
                let dist_b = (b.center_point.x - point_f64.x).powi(2) +
                           (b.center_point.y - point_f64.y).powi(2) +
                           (b.center_point.z - point_f64.z).powi(2);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .map(|(idx, _)| idx)
    }
}

/// Component to mark tile entities
#[derive(Component)]
pub struct TileEntity {
    pub tile_index: usize,
    pub is_pentagon: bool,
}

/// Create mesh for a tile
pub fn create_tile_mesh(boundary: &[Vec3], center: Vec3) -> Mesh {
    let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::all());
    
    let mut vertices = vec![center];
    vertices.extend_from_slice(boundary);
    
    let mut indices = Vec::new();
    for i in 0..boundary.len() {
        let next = (i + 1) % boundary.len();
        indices.extend_from_slice(&[0, i as u32 + 1, next as u32 + 1]);
    }
    
    let normals: Vec<[f32; 3]> = vertices.iter()
        .map(|v| {
            let n = v.normalize();
            [n.x, n.y, n.z]
        })
        .collect();
    
    let uvs: Vec<[f32; 2]> = vertices.iter()
        .map(|_| [0.5, 0.5]) // Simple UV mapping for now
        .collect();
    
    let positions: Vec<[f32; 3]> = vertices.iter()
        .map(|v| [v.x, v.y, v.z])
        .collect();
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    
    mesh
}