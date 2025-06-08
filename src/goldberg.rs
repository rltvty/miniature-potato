//! Goldberg polyhedron generation for spherical hex/pentagon tiling
//! 
//! This module implements a Goldberg polyhedron, which is a convex polyhedron
//! made up of exactly 12 pentagonal faces and the rest hexagonal faces.
//! It's a generalization of the truncated icosahedron (soccer ball).

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use std::collections::{HashMap, HashSet};

/// Goldberg polyhedron parameters
pub struct GoldbergPolyhedron {
    pub radius: f32,
    pub h: u32, // First Goldberg parameter
    pub k: u32, // Second Goldberg parameter
}

impl GoldbergPolyhedron {
    /// Create a new Goldberg polyhedron with parameters (h, k)
    pub fn new(radius: f32, h: u32, k: u32) -> Self {
        Self { radius, h, k }
    }

    /// Generate the Goldberg polyhedron mesh
    pub fn generate(&self) -> Mesh {
        let (vertices, indices) = self.generate_vertices_and_indices();
        
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

    /// Generate mesh with raw data for tile generation
    pub fn generate_with_data(&self) -> (Mesh, Vec<Vec3>, Vec<u32>) {
        let (vertices, indices) = self.generate_vertices_and_indices();
        
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
        mesh.insert_indices(Indices::U32(indices.clone()));

        (mesh, vertices, indices)
    }

    /// Generate vertices and indices for the Goldberg polyhedron
    fn generate_vertices_and_indices(&self) -> (Vec<Vec3>, Vec<u32>) {
        // Start with base icosahedron
        let base_vertices = self.generate_icosahedron_vertices();
        let base_indices = self.generate_icosahedron_indices();
        
        // Create subdivided mesh based on (h,k)
        let (mut vertices, indices) = self.subdivide_goldberg(&base_vertices, &base_indices);
        
        // Project all vertices to sphere surface
        for vertex in vertices.iter_mut() {
            *vertex = vertex.normalize() * self.radius;
        }
        
        (vertices, indices)
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

    /// Subdivide the icosahedron according to Goldberg parameters (h,k)
    fn subdivide_goldberg(&self, vertices: &[Vec3], indices: &[u32]) -> (Vec<Vec3>, Vec<u32>) {
        let mut new_vertices = vertices.to_vec();
        let mut new_indices = Vec::new();
        let mut vertex_map = HashMap::new();
        
        // For each triangle face of the icosahedron
        for triangle in indices.chunks(3) {
            let v1_idx = triangle[0] as usize;
            let v2_idx = triangle[1] as usize;
            let v3_idx = triangle[2] as usize;
            
            let v1 = vertices[v1_idx];
            let v2 = vertices[v2_idx];
            let v3 = vertices[v3_idx];
            
            // Subdivide this triangular face according to (h,k) parameters
            let (face_vertices, face_indices) = self.subdivide_face(v1, v2, v3, v1_idx, v2_idx, v3_idx, &mut new_vertices, &mut vertex_map);
            
            // Add the new indices to our overall list
            new_indices.extend_from_slice(&face_indices);
        }
        
        (new_vertices, new_indices)
    }

    /// Subdivide a single triangular face according to Goldberg parameters
    fn subdivide_face(
        &self,
        v1: Vec3,
        v2: Vec3,
        v3: Vec3,
        v1_idx: usize,
        v2_idx: usize,
        v3_idx: usize,
        vertices: &mut Vec<Vec3>,
        vertex_map: &mut HashMap<(i32, i32, i32), usize>,
    ) -> (Vec<Vec3>, Vec<u32>) {
        let h = self.h as i32;
        let k = self.k as i32;
        
        // Basis vectors for the triangular grid
        let edge1 = v2 - v1;
        let edge2 = v3 - v1;
        
        let mut face_indices = Vec::new();
        
        // Create a grid of points within the triangle
        // The grid has size determined by parameters (h,k)
        // For each point in the grid:
        for i in 0..=h+k {
            for j in 0..=h+k-i {
                // Calculate barycentric coordinates
                let a = i as f32 / (h+k) as f32;
                let b = j as f32 / (h+k) as f32;
                let c = 1.0 - a - b;
                
                // Skip if outside the triangle
                if a < 0.0 || b < 0.0 || c < 0.0 {
                    continue;
                }
                
                // Create the new vertex
                let new_vertex = v1 + edge1 * a + edge2 * b;
                
                // Use the original vertex if we're at a corner
                let vertex_idx = if (i, j) == (0, 0) {
                    v1_idx
                } else if (i, j) == (h+k, 0) {
                    v2_idx
                } else if (i, j) == (0, h+k) {
                    v3_idx
                } else {
                    // Generate a unique key for this vertex to avoid duplicates
                    let key = (i, j, v1_idx as i32 * 1000 + v2_idx as i32 * 100 + v3_idx as i32);
                    
                    if let Some(&idx) = vertex_map.get(&key) {
                        idx
                    } else {
                        let idx = vertices.len();
                        vertices.push(new_vertex);
                        vertex_map.insert(key, idx);
                        idx
                    }
                };
                
                // Create triangles by connecting with neighbors
                if i < h+k && j < h+k-i {
                    // Connect with the vertex to the right and below
                    let right_idx = if (i+1, j) == (h+k, 0) {
                        v2_idx
                    } else {
                        let key = (i+1, j, v1_idx as i32 * 1000 + v2_idx as i32 * 100 + v3_idx as i32);
                        vertex_map.get(&key).copied().unwrap_or_else(|| {
                            let new_vertex = v1 + edge1 * ((i+1) as f32 / (h+k) as f32) + edge2 * (j as f32 / (h+k) as f32);
                            let idx = vertices.len();
                            vertices.push(new_vertex);
                            vertex_map.insert(key, idx);
                            idx
                        })
                    };
                    
                    let below_idx = if (i, j+1) == (0, h+k) {
                        v3_idx
                    } else {
                        let key = (i, j+1, v1_idx as i32 * 1000 + v2_idx as i32 * 100 + v3_idx as i32);
                        vertex_map.get(&key).copied().unwrap_or_else(|| {
                            let new_vertex = v1 + edge1 * (i as f32 / (h+k) as f32) + edge2 * ((j+1) as f32 / (h+k) as f32);
                            let idx = vertices.len();
                            vertices.push(new_vertex);
                            vertex_map.insert(key, idx);
                            idx
                        })
                    };
                    
                    // Check if we're at the boundary of the triangle
                    if i + j + 2 > h + k {
                        // Only add one triangle (no diagonal point)
                        face_indices.extend_from_slice(&[vertex_idx as u32, right_idx as u32, below_idx as u32]);
                    } else {
                        // We have room for the diagonal point
                        let diag_idx = {
                            let key = (i+1, j+1, v1_idx as i32 * 1000 + v2_idx as i32 * 100 + v3_idx as i32);
                            vertex_map.get(&key).copied().unwrap_or_else(|| {
                                let new_vertex = v1 + edge1 * ((i+1) as f32 / (h+k) as f32) + edge2 * ((j+1) as f32 / (h+k) as f32);
                                let idx = vertices.len();
                                vertices.push(new_vertex);
                                vertex_map.insert(key, idx);
                                idx
                            })
                        };
                        
                        // Add two triangles to form a quad
                        face_indices.extend_from_slice(&[vertex_idx as u32, right_idx as u32, below_idx as u32]);
                        face_indices.extend_from_slice(&[right_idx as u32, diag_idx as u32, below_idx as u32]);
                    }
                }
            }
        }
        
        (vertices.to_vec(), face_indices)
    }
}

/// Helper functions for Goldberg polyhedron
pub mod goldberg_helpers {
    /// Compute the subdivision count `n` from Goldberg indices (h, k)
    pub fn subdivision_count(h: u32, k: u32) -> f64 {
        let h = h as f64;
        let k = k as f64;
        (h * h + h * k + k * k).sqrt()
    }

    /// Compute the number of hexagons in a Goldberg polyhedron
    pub fn num_hexagons(h: u32, k: u32) -> u32 {
        10 * (h * h + h * k + k * k)
    }

    /// Always returns 12 (the number of pentagons)
    pub fn num_pentagons() -> u32 {
        12
    }

    /// Total number of faces (hexagons + pentagons)
    pub fn total_faces(h: u32, k: u32) -> u32 {
        num_hexagons(h, k) + num_pentagons()
    }

    /// Compute the triangle edge length on the sphere from radius and (h, k)
    pub fn triangle_edge_length(r: f64, h: u32, k: u32) -> f64 {
        let n = subdivision_count(h, k);
        let base_icosahedron_edge_length = 1.05146; // Approximate edge length for unit icosahedron
        r * base_icosahedron_edge_length / n
    }
}
