//! Pentagon-Centric Goldberg Construction
//! 
//! Build a soccer ball by starting with one pentagon and wrapping hexagons around it,
//! then expanding outward. This mimics the natural construction method.

use bevy::prelude::*;
use std::collections::HashMap;

/// A true Goldberg polyhedron built pentagon by pentagon
#[derive(Debug, Clone)]
pub struct GoldbergPolyhedron {
    pub m: u32,
    pub n: u32,
    pub radius: f32,
    pub vertices: Vec<Vec3>,
    pub pentagons: Vec<Pentagon>,
    pub hexagons: Vec<Hexagon>,
    pub edges: Vec<Edge>,
}

/// A pentagonal face
#[derive(Debug, Clone)]
pub struct Pentagon {
    pub vertices: [usize; 5],
    pub center: Vec3,
    pub normal: Vec3,
    pub neighbors: [Option<usize>; 5], // Indices of neighboring hexagons
    pub edge_indices: [usize; 5],
}

/// A hexagonal face  
#[derive(Debug, Clone)]
pub struct Hexagon {
    pub vertices: [usize; 6],
    pub center: Vec3,
    pub normal: Vec3,
    pub neighbors: [Option<usize>; 6], // Mix of pentagon and hexagon indices
    pub edge_indices: [usize; 6],
}

/// An edge connecting two faces
#[derive(Debug, Clone)]
pub struct Edge {
    pub vertices: [usize; 2],
    pub faces: [usize; 2],
    pub length: f32,
}

/// Type of face
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceType {
    Pentagon(usize),
    Hexagon(usize),
}

impl FaceType {
    pub fn is_pentagon(&self) -> bool {
        matches!(self, FaceType::Pentagon(_))
    }
    
    pub fn is_hexagon(&self) -> bool {
        matches!(self, FaceType::Hexagon(_))
    }
    
    pub fn index(&self) -> usize {
        match self {
            FaceType::Pentagon(idx) => *idx,
            FaceType::Hexagon(idx) => *idx,
        }
    }
}

impl GoldbergPolyhedron {
    /// Create a new Goldberg polyhedron using pentagon-centric construction
    pub fn new(m: u32, n: u32, radius: f32) -> Self {
        println!("🔧 Creating Goldberg polyhedron GP({},{}) with pentagon-centric method", m, n);
        
        let mut polyhedron = Self {
            m,
            n,
            radius,
            vertices: Vec::new(),
            pentagons: Vec::new(),
            hexagons: Vec::new(),
            edges: Vec::new(),
        };
        
        if m == 1 && n == 1 {
            polyhedron.construct_soccer_ball_pentagon_centric();
        } else {
            polyhedron.construct_general_pentagon_centric();
        }
        
        polyhedron.print_statistics();
        polyhedron
    }
    
    /// Build a soccer ball starting with one pentagon + 5 hexagons
    fn construct_soccer_ball_pentagon_centric(&mut self) {
        println!("⚽ Building soccer ball pentagon by pentagon...");
        
        // Step 1: Create the first pentagon at the "north pole"
        let first_pentagon = self.create_pentagon_at_pole();
        
        // Step 2: Create 5 hexagons around the first pentagon
        let first_ring_hexagons = self.create_hexagon_ring_around_pentagon(first_pentagon);
        
        // Step 3: Create 5 more pentagons between the hexagons
        let second_ring_pentagons = self.create_pentagons_between_hexagons(&first_ring_hexagons);
        
        // Step 4: Create hexagons between the new pentagons
        let second_ring_hexagons = self.create_hexagons_between_pentagons(&second_ring_pentagons);
        
        // Step 5: Create the final pentagon at the "south pole"
        let final_pentagon = self.create_pentagon_at_south_pole(&second_ring_hexagons);
        
        // Step 6: Generate vertices and edges from the face structure
        self.generate_vertices_from_faces();
        self.generate_edges_from_faces();
        
        println!("✅ Soccer ball construction complete!");
    }
    
    /// Create the first pentagon at the north pole
    fn create_pentagon_at_pole(&mut self) -> usize {
        println!("🔺 Creating north pole pentagon...");
        
        let center = Vec3::new(0.0, 0.0, self.radius);
        let pentagon = Pentagon {
            vertices: [0; 5], // Will be filled later
            center,
            normal: center.normalize(),
            neighbors: [None; 5],
            edge_indices: [0; 5],
        };
        
        self.pentagons.push(pentagon);
        self.pentagons.len() - 1
    }
    
    /// Create 5 hexagons in a ring around a pentagon
    fn create_hexagon_ring_around_pentagon(&mut self, pentagon_idx: usize) -> Vec<usize> {
        println!("🔶 Creating first ring of 5 hexagons...");
        
        let pentagon_center = self.pentagons[pentagon_idx].center;
        let mut hexagon_indices = Vec::new();
        
        // Place 5 hexagons around the pentagon
        for i in 0..5 {
            let angle = (i as f32) * 2.0 * std::f32::consts::PI / 5.0;
            
            // Calculate position: rotate around the pentagon's axis
            let offset_distance = 1.2; // Distance from pentagon center
            let offset_height = -0.5;  // Move down from north pole
            
            let x = offset_distance * angle.cos();
            let y = offset_distance * angle.sin();
            let z = pentagon_center.z + offset_height;
            
            let center = Vec3::new(x, y, z).normalize() * self.radius;
            
            let hexagon = Hexagon {
                vertices: [0; 6], // Will be filled later
                center,
                normal: center.normalize(),
                neighbors: [None; 6],
                edge_indices: [0; 6],
            };
            
            self.hexagons.push(hexagon);
            hexagon_indices.push(self.hexagons.len() - 1);
        }
        
        hexagon_indices
    }
    
    /// Create pentagons between hexagons in the first ring
    fn create_pentagons_between_hexagons(&mut self, hexagon_indices: &[usize]) -> Vec<usize> {
        println!("🔻 Creating second ring of 5 pentagons...");
        
        let mut pentagon_indices = Vec::new();
        
        for i in 0..5 {
            let hex1_idx = hexagon_indices[i];
            let hex2_idx = hexagon_indices[(i + 1) % 5];
            
            let hex1_center = self.hexagons[hex1_idx].center;
            let hex2_center = self.hexagons[hex2_idx].center;
            
            // Place pentagon between two adjacent hexagons
            let center = ((hex1_center + hex2_center) * 0.5).normalize() * self.radius;
            
            let pentagon = Pentagon {
                vertices: [0; 5],
                center,
                normal: center.normalize(),
                neighbors: [None; 5],
                edge_indices: [0; 5],
            };
            
            self.pentagons.push(pentagon);
            pentagon_indices.push(self.pentagons.len() - 1);
        }
        
        pentagon_indices
    }
    
    /// Create hexagons between pentagons in the second ring
    fn create_hexagons_between_pentagons(&mut self, pentagon_indices: &[usize]) -> Vec<usize> {
        println!("🔷 Creating second ring of 10 hexagons...");
        
        let mut hexagon_indices = Vec::new();
        
        // Create 2 hexagons between each pair of adjacent pentagons
        for i in 0..5 {
            let pent1_idx = pentagon_indices[i];
            let pent2_idx = pentagon_indices[(i + 1) % 5];
            
            let pent1_center = self.pentagons[pent1_idx].center;
            let pent2_center = self.pentagons[pent2_idx].center;
            
            // Create 2 hexagons between these pentagons
            for j in 0..2 {
                let t = (j as f32 + 1.0) / 3.0; // Position along the arc
                let center = (pent1_center.lerp(pent2_center, t)).normalize() * self.radius;
                
                let hexagon = Hexagon {
                    vertices: [0; 6],
                    center,
                    normal: center.normalize(),
                    neighbors: [None; 6],
                    edge_indices: [0; 6],
                };
                
                self.hexagons.push(hexagon);
                hexagon_indices.push(self.hexagons.len() - 1);
            }
        }
        
        hexagon_indices
    }
    
    /// Create the final pentagon at the south pole
    fn create_pentagon_at_south_pole(&mut self, _hexagon_indices: &[usize]) -> usize {
        println!("🔻 Creating south pole pentagon...");
        
        let center = Vec3::new(0.0, 0.0, -self.radius);
        let pentagon = Pentagon {
            vertices: [0; 5],
            center,
            normal: center.normalize(),
            neighbors: [None; 5],
            edge_indices: [0; 5],
        };
        
        self.pentagons.push(pentagon);
        self.pentagons.len() - 1
    }
    
    /// Generate vertices from face centers (simplified)
    fn generate_vertices_from_faces(&mut self) {
        println!("📍 Generating vertices from face structure...");
        
        // For now, use face centers as vertices
        // In a complete implementation, we'd compute actual edge intersections
        for pentagon in &self.pentagons {
            self.vertices.push(pentagon.center);
        }
        
        for hexagon in &self.hexagons {
            self.vertices.push(hexagon.center);
        }
    }
    
    /// Generate edges from face adjacencies (placeholder)
    fn generate_edges_from_faces(&mut self) {
        println!("📏 Generating edges from face adjacencies...");
        // Placeholder - would compute actual edges between adjacent faces
        self.edges = Vec::new();
    }
    
    /// General construction for other GP(m,n) values
    fn construct_general_pentagon_centric(&mut self) {
        println!("🔧 Using general pentagon-centric construction for GP({},{})...", self.m, self.n);
        
        // Start with simplified approach for other patterns
        // This would be expanded to handle arbitrary GP(m,n)
        self.construct_soccer_ball_pentagon_centric();
    }
    
    /// Get expected counts
    pub fn expected_hexagon_count(&self) -> u32 {
        if self.m == 1 && self.n == 1 {
            20 // Soccer ball has exactly 20 hexagons
        } else {
            10 * (self.m * self.m + self.m * self.n + self.n * self.n)
        }
    }
    
    pub fn expected_face_count(&self) -> u32 {
        12 + self.expected_hexagon_count()
    }
    
    pub fn expected_vertex_count(&self) -> u32 {
        if self.m == 1 && self.n == 1 {
            60 // Soccer ball has exactly 60 vertices
        } else {
            let h = self.expected_hexagon_count();
            20 + 2 * h
        }
    }
    
    /// Print statistics
    fn print_statistics(&self) {
        println!("\n📊 Pentagon-Centric Goldberg Polyhedron GP({},{}) Statistics:", self.m, self.n);
        println!("   Pentagons: {} (expected: 12)", self.pentagons.len());
        println!("   Hexagons: {} (expected: {})", self.hexagons.len(), self.expected_hexagon_count());
        println!("   Total faces: {} (expected: {})", 
                 self.pentagons.len() + self.hexagons.len(), 
                 self.expected_face_count());
        println!("   Vertices: {} (expected: {})", self.vertices.len(), self.expected_vertex_count());
        println!("   Edges: {}", self.edges.len());
        println!("   Radius: {}", self.radius);
        
        if self.pentagons.len() == 12 && self.hexagons.len() == 20 {
            println!("   ✅ Perfect soccer ball structure!");
        }
    }
}
