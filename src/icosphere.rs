//! Icosphere generation for creating spherical meshes with even triangle distribution

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

/// Icosphere mesh generator
pub struct Icosphere {
    pub radius: f32,
    pub subdivisions: usize,
}

impl Icosphere {
    pub fn new(radius: f32, subdivisions: usize) -> Self {
        Self { radius, subdivisions }
    }

    /// Generate the icosphere mesh
    pub fn generate(&self) -> Mesh {
        let mut vertices = self.generate_icosahedron_vertices();
        let mut indices = self.generate_icosahedron_indices();

        // Subdivide the mesh
        for _ in 0..self.subdivisions {
            let (new_vertices, new_indices) = self.subdivide(&vertices, &indices);
            vertices = new_vertices;
            indices = new_indices;
        }

        // Project all vertices to sphere surface
        for vertex in vertices.iter_mut() {
            *vertex = vertex.normalize() * self.radius;
        }

        // Create the mesh
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            bevy::render::render_asset::RenderAssetUsages::default(),
        );

        // Convert Vec3 to [f32; 3] for positions
        let positions: Vec<[f32; 3]> = vertices.iter().map(|v| [v.x, v.y, v.z]).collect();
        
        // Generate normals (for a sphere, normals are just normalized positions)
        let normals: Vec<[f32; 3]> = vertices.iter().map(|v| {
            let n = v.normalize();
            [n.x, n.y, n.z]
        }).collect();

        // Generate UV coordinates (simple spherical mapping)
        let uvs: Vec<[f32; 2]> = vertices.iter().map(|v| {
            let n = v.normalize();
            let u = 0.5 + (n.z.atan2(n.x) / (2.0 * std::f32::consts::PI));
            let v = 0.5 - (n.y.asin() / std::f32::consts::PI);
            [u, v]
        }).collect();

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));

        mesh
    }

    /// Generate the base icosahedron vertices
    fn generate_icosahedron_vertices(&self) -> Vec<Vec3> {
        let phi = (1.0 + 5.0_f32.sqrt()) / 2.0; // Golden ratio
        
        vec![
            // Top vertex
            Vec3::new(0.0, 1.0, phi),
            Vec3::new(0.0, -1.0, phi),
            Vec3::new(0.0, 1.0, -phi),
            Vec3::new(0.0, -1.0, -phi),
            
            // Middle ring
            Vec3::new(1.0, phi, 0.0),
            Vec3::new(-1.0, phi, 0.0),
            Vec3::new(1.0, -phi, 0.0),
            Vec3::new(-1.0, -phi, 0.0),
            
            // Bottom ring
            Vec3::new(phi, 0.0, 1.0),
            Vec3::new(-phi, 0.0, 1.0),
            Vec3::new(phi, 0.0, -1.0),
            Vec3::new(-phi, 0.0, -1.0),
        ]
    }

    /// Generate the base icosahedron triangle indices
    fn generate_icosahedron_indices(&self) -> Vec<u32> {
        vec![
            // Top cap
            0, 8, 4,   0, 4, 5,   0, 5, 9,   0, 9, 1,   0, 1, 8,
            // Bottom cap  
            3, 10, 6,  3, 6, 7,   3, 7, 11,  3, 11, 2,  3, 2, 10,
            // Upper middle
            4, 8, 10,  5, 4, 2,   9, 5, 11,  1, 9, 7,   8, 1, 6,
            // Lower middle
            4, 10, 2,  5, 2, 11,  9, 11, 7,  1, 7, 6,   8, 6, 10,
        ]
    }

    /// Subdivide triangles by adding midpoint vertices
    fn subdivide(&self, vertices: &[Vec3], indices: &[u32]) -> (Vec<Vec3>, Vec<u32>) {
        let mut new_vertices = vertices.to_vec();
        let mut new_indices = Vec::new();
        let mut midpoint_cache = std::collections::HashMap::new();

        // Process each triangle
        for triangle in indices.chunks(3) {
            let v1 = triangle[0] as usize;
            let v2 = triangle[1] as usize;
            let v3 = triangle[2] as usize;

            // Get midpoint indices (or create new vertices)
            let mid12 = self.get_midpoint_index(v1, v2, &mut new_vertices, &mut midpoint_cache);
            let mid23 = self.get_midpoint_index(v2, v3, &mut new_vertices, &mut midpoint_cache);
            let mid31 = self.get_midpoint_index(v3, v1, &mut new_vertices, &mut midpoint_cache);

            // Create 4 new triangles
            new_indices.extend_from_slice(&[v1 as u32, mid12, mid31]);
            new_indices.extend_from_slice(&[v2 as u32, mid23, mid12]);
            new_indices.extend_from_slice(&[v3 as u32, mid31, mid23]);
            new_indices.extend_from_slice(&[mid12, mid23, mid31]);
        }

        (new_vertices, new_indices)
    }

    /// Get or create midpoint vertex between two vertices
    fn get_midpoint_index(
        &self,
        v1: usize,
        v2: usize,
        vertices: &mut Vec<Vec3>,
        cache: &mut std::collections::HashMap<(usize, usize), usize>,
    ) -> u32 {
        let key = if v1 < v2 { (v1, v2) } else { (v2, v1) };
        
        if let Some(&index) = cache.get(&key) {
            return index as u32;
        }

        let midpoint = (vertices[v1] + vertices[v2]) * 0.5;
        let index = vertices.len();
        vertices.push(midpoint);
        cache.insert(key, index);
        
        index as u32
    }
}
