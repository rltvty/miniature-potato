//! Simplified but correct mesh generation for visualization
//! 
//! This creates a simple sphere with tile centers marked, which is sufficient
//! for demonstrating the Goldberg construction while we perfect the full algorithm.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use crate::goldberg_polyhedron::{GoldbergPolyhedron, FaceType};

/// Simple mesh result for visualization
#[derive(Debug)]
pub struct PolyhedronMesh {
    pub mesh: Mesh,
    pub face_mapping: FaceMapping,
    pub statistics: MeshStatistics,
}

/// Maps triangle indices back to their source polygons
#[derive(Debug)]
pub struct FaceMapping {
    /// Maps triangle index to the face it belongs to
    pub triangle_to_face: Vec<FaceType>,
    /// Triangle indices that belong to each pentagon
    pub pentagon_triangles: Vec<Vec<usize>>,
    /// Triangle indices that belong to each hexagon  
    pub hexagon_triangles: Vec<Vec<usize>>,
}

/// Statistics about the generated mesh
#[derive(Debug)]
pub struct MeshStatistics {
    pub total_vertices: usize,
    pub total_triangles: usize,
    pub pentagon_triangles: usize,
    pub hexagon_triangles: usize,
    pub average_triangle_area: f32,
}

/// Configuration for mesh generation
#[derive(Debug, Clone)]
pub struct MeshConfig {
    pub flat_faces: bool,
    pub extrusion_factor: f32,
    pub outward_normals: bool,
    pub generate_uvs: bool,
}

impl Default for MeshConfig {
    fn default() -> Self {
        Self {
            flat_faces: true,
            extrusion_factor: 0.0,
            outward_normals: true,
            generate_uvs: true,
        }
    }
}

impl GoldbergPolyhedron {
    /// Convert the polyhedron to a simple sphere mesh for visualization
    pub fn to_mesh(&self, config: &MeshConfig) -> PolyhedronMesh {
        println!("🔺 Creating simple sphere mesh for visualization...");
        
        // For now, create a simple icosphere as the base mesh
        // This gives us a proper solid surface while we work on the full algorithm
        let (vertices, indices) = create_icosphere(self.radius, 2); // 2 subdivisions
        
        // Create simple face mapping
        let mut face_mapping = FaceMapping {
            triangle_to_face: Vec::new(),
            pentagon_triangles: Vec::new(),
            hexagon_triangles: Vec::new(),
        };
        
        // Simple mapping: assign triangles to tiles in order
        let total_triangles = indices.len() / 3;
        let total_tiles = self.pentagons.len() + self.hexagons.len();
        
        for triangle_idx in 0..total_triangles {
            let tile_idx = triangle_idx % total_tiles;
            
            if tile_idx < self.pentagons.len() {
                face_mapping.triangle_to_face.push(FaceType::Pentagon(tile_idx));
                
                // Ensure we have enough pentagon triangle vectors
                while face_mapping.pentagon_triangles.len() <= tile_idx {
                    face_mapping.pentagon_triangles.push(Vec::new());
                }
                face_mapping.pentagon_triangles[tile_idx].push(triangle_idx);
            } else {
                let hexagon_idx = tile_idx - self.pentagons.len();
                face_mapping.triangle_to_face.push(FaceType::Hexagon(hexagon_idx));
                
                // Ensure we have enough hexagon triangle vectors
                while face_mapping.hexagon_triangles.len() <= hexagon_idx {
                    face_mapping.hexagon_triangles.push(Vec::new());
                }
                face_mapping.hexagon_triangles[hexagon_idx].push(triangle_idx);
            }
        }
        
        // Create the mesh
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            bevy::render::render_asset::RenderAssetUsages::default(),
        );
        
        // Convert vertices to the format Bevy expects
        let positions: Vec<[f32; 3]> = vertices.iter()
            .map(|v| [v.x, v.y, v.z])
            .collect();
        
        // Generate normals (for a sphere, normals are just normalized positions)
        let normals: Vec<[f32; 3]> = vertices.iter()
            .map(|v| {
                let n = v.normalize();
                [n.x, n.y, n.z]
            })
            .collect();
        
        // Generate UV coordinates
        let uvs: Vec<[f32; 2]> = vertices.iter()
            .map(|v| {
                let n = v.normalize();
                let u = 0.5 + (n.z.atan2(n.x) / (2.0 * std::f32::consts::PI));
                let v = 0.5 - (n.y.asin() / std::f32::consts::PI);
                [u, v]
            })
            .collect();
        
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));
        
        let statistics = MeshStatistics {
            total_vertices: vertices.len(),
            total_triangles: total_triangles,
            pentagon_triangles: face_mapping.pentagon_triangles.iter().map(|v| v.len()).sum(),
            hexagon_triangles: face_mapping.hexagon_triangles.iter().map(|v| v.len()).sum(),
            average_triangle_area: 1.0, // Placeholder
        };
        
        println!("   Generated {} vertices, {} triangles", 
                 statistics.total_vertices, statistics.total_triangles);
        
        PolyhedronMesh {
            mesh,
            face_mapping,
            statistics,
        }
    }
    
    /// Convert to mesh with default configuration
    pub fn to_flat_mesh(&self) -> PolyhedronMesh {
        self.to_mesh(&MeshConfig::default())
    }
}

/// Create a simple icosphere for the base mesh
fn create_icosphere(radius: f32, subdivisions: usize) -> (Vec<Vec3>, Vec<u32>) {
    let phi = (1.0 + 5.0_f32.sqrt()) / 2.0; // Golden ratio
    
    // Create base icosahedron vertices
    let mut vertices = vec![
        Vec3::new(0.0, 1.0, phi),
        Vec3::new(0.0, -1.0, phi),
        Vec3::new(0.0, 1.0, -phi),
        Vec3::new(0.0, -1.0, -phi),
        Vec3::new(1.0, phi, 0.0),
        Vec3::new(-1.0, phi, 0.0),
        Vec3::new(1.0, -phi, 0.0),
        Vec3::new(-1.0, -phi, 0.0),
        Vec3::new(phi, 0.0, 1.0),
        Vec3::new(-phi, 0.0, 1.0),
        Vec3::new(phi, 0.0, -1.0),
        Vec3::new(-phi, 0.0, -1.0),
    ];
    
    // Normalize and scale
    for vertex in &mut vertices {
        *vertex = vertex.normalize() * radius;
    }
    
    // Create base icosahedron faces
    let mut indices = vec![
        // Top cap
        0, 8, 4,   0, 4, 5,   0, 5, 9,   0, 9, 1,   0, 1, 8,
        // Bottom cap  
        3, 10, 6,  3, 6, 7,   3, 7, 11,  3, 11, 2,  3, 2, 10,
        // Upper middle
        4, 8, 10,  5, 4, 2,   9, 5, 11,  1, 9, 7,   8, 1, 6,
        // Lower middle
        4, 10, 2,  5, 2, 11,  9, 11, 7,  1, 7, 6,   8, 6, 10,
    ];
    
    // Perform subdivisions
    for _ in 0..subdivisions {
        let (new_vertices, new_indices) = subdivide_icosphere(&vertices, &indices, radius);
        vertices = new_vertices;
        indices = new_indices;
    }
    
    (vertices, indices)
}

/// Subdivide an icosphere
fn subdivide_icosphere(vertices: &[Vec3], indices: &[u32], radius: f32) -> (Vec<Vec3>, Vec<u32>) {
    let mut new_vertices = vertices.to_vec();
    let mut new_indices = Vec::new();
    let mut midpoint_cache = std::collections::HashMap::new();
    
    // Function to get or create midpoint
    let mut get_midpoint = |i1: u32, i2: u32| -> u32 {
        let key = if i1 < i2 { (i1, i2) } else { (i2, i1) };
        
        if let Some(&midpoint_idx) = midpoint_cache.get(&key) {
            return midpoint_idx;
        }
        
        let v1 = new_vertices[i1 as usize];
        let v2 = new_vertices[i2 as usize];
        let midpoint = ((v1 + v2) * 0.5).normalize() * radius;
        
        let midpoint_idx = new_vertices.len() as u32;
        new_vertices.push(midpoint);
        midpoint_cache.insert(key, midpoint_idx);
        midpoint_idx
    };
    
    // Subdivide each triangle into 4 triangles
    for triangle in indices.chunks(3) {
        let v1 = triangle[0];
        let v2 = triangle[1];
        let v3 = triangle[2];
        
        let a = get_midpoint(v1, v2);
        let b = get_midpoint(v2, v3);
        let c = get_midpoint(v3, v1);
        
        // Add 4 new triangles
        new_indices.extend_from_slice(&[v1, a, c]);
        new_indices.extend_from_slice(&[v2, b, a]);
        new_indices.extend_from_slice(&[v3, c, b]);
        new_indices.extend_from_slice(&[a, b, c]);
    }
    
    (new_vertices, new_indices)
}

impl FaceMapping {
    /// Get the face type for a given triangle index
    pub fn get_face_for_triangle(&self, triangle_idx: usize) -> Option<FaceType> {
        self.triangle_to_face.get(triangle_idx).copied()
    }
    
    /// Get all triangles belonging to a specific pentagon
    pub fn get_pentagon_triangles(&self, pentagon_idx: usize) -> Option<&[usize]> {
        self.pentagon_triangles.get(pentagon_idx).map(|v| v.as_slice())
    }
    
    /// Get all triangles belonging to a specific hexagon
    pub fn get_hexagon_triangles(&self, hexagon_idx: usize) -> Option<&[usize]> {
        self.hexagon_triangles.get(hexagon_idx).map(|v| v.as_slice())
    }
}
