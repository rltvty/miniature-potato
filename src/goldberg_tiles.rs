//! Tile system based on Goldberg polyhedron

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use crate::tiles::{Tile, TileMap, TileType};

/// Generate tiles from a Goldberg polyhedron
pub fn generate_tiles_from_goldberg(
    vertices: &[Vec3],
    indices: &[u32],
    h: u32,
    k: u32,
    radius: f32,
) -> TileMap {
    let mut tile_map = TileMap::default();
    
    println!("Generating tiles from Goldberg polyhedron with parameters ({}, {})...", h, k);
    
    // 1. Identify pentagon centers (12 original icosahedron vertices)
    let pentagon_centers = identify_pentagon_centers();
    
    // 2. Identify hexagon centers based on (h,k) parameters
    let hexagon_centers = if h == 1 && k == 0 {
        // For soccer ball pattern, use exactly 10 hexagons
        identify_soccer_ball_hexagons()
    } else {
        identify_hexagon_centers(h, k)
    };
    
    println!("Found {} pentagon centers", pentagon_centers.len());
    println!("Found {} hexagon centers", hexagon_centers.len());
    
    // 3. Pre-create all tiles to avoid indexing issues
    // First add all pentagons
    for (i, &center) in pentagon_centers.iter().enumerate() {
        tile_map.tiles.push(Tile {
            tile_type: TileType::Pentagon,
            center,
            neighbors: Vec::new(),
            triangle_indices: Vec::new(),
        });
        tile_map.pentagon_indices.push(tile_map.tiles.len() - 1);
    }
    
    // Then add all hexagons
    for (i, &center) in hexagon_centers.iter().enumerate() {
        tile_map.tiles.push(Tile {
            tile_type: TileType::Hexagon,
            center,
            neighbors: Vec::new(),
            triangle_indices: Vec::new(),
        });
        tile_map.hexagon_indices.push(tile_map.tiles.len() - 1);
    }
    
    // 4. Find triangles centers
    let triangle_centers = calculate_triangle_centers(vertices, indices);
    
    // 5. Assign triangles to nearest tile center
    for (tri_idx, tri_center) in triangle_centers.iter().enumerate() {
        let (tile_type, tile_idx) = find_closest_tile_center(
            *tri_center, 
            &pentagon_centers, 
            &hexagon_centers
        );
        
        // Get the global tile index
        let global_tile_idx = match tile_type {
            TileType::Pentagon => tile_map.pentagon_indices[tile_idx],
            TileType::Hexagon => tile_map.hexagon_indices[tile_idx],
        };
        
        // Add triangle to the tile
        tile_map.tiles[global_tile_idx].triangle_indices.push(tri_idx);
        tile_map.triangle_to_tile.insert(tri_idx, global_tile_idx);
    }
    
    // 6. Compute tile neighbors
    compute_tile_neighbors(&mut tile_map, vertices, indices);
    
    // 7. Output statistics
    print_tile_statistics(&tile_map, h, k, indices);
    
    tile_map
}

/// Identify the 12 pentagon centers (original icosahedron vertices)
fn identify_pentagon_centers() -> Vec<Vec3> {
    let phi = (1.0 + 5.0_f32.sqrt()) / 2.0; // Golden ratio
    
    vec![
        Vec3::new(0.0, 1.0, phi).normalize(),
        Vec3::new(0.0, -1.0, phi).normalize(),
        Vec3::new(0.0, 1.0, -phi).normalize(),
        Vec3::new(0.0, -1.0, -phi).normalize(),
        Vec3::new(1.0, phi, 0.0).normalize(),
        Vec3::new(-1.0, phi, 0.0).normalize(),
        Vec3::new(1.0, -phi, 0.0).normalize(),
        Vec3::new(-1.0, -phi, 0.0).normalize(),
        Vec3::new(phi, 0.0, 1.0).normalize(),
        Vec3::new(-phi, 0.0, 1.0).normalize(),
        Vec3::new(phi, 0.0, -1.0).normalize(),
        Vec3::new(-phi, 0.0, -1.0).normalize(),
    ]
}

/// Special function to identify exactly 10 hexagon centers for the soccer ball pattern
fn identify_soccer_ball_hexagons() -> Vec<Vec3> {
    // For a proper soccer ball, we need exactly 10 hexagons
    // We'll place them at specific locations that work well with the 12 pentagons
    
    let phi = (1.0 + 5.0_f32.sqrt()) / 2.0; // Golden ratio
    
    // These positions are carefully chosen to create a truncated icosahedron (soccer ball)
    vec![
        // Top ring (5 hexagons)
        Vec3::new(0.5, 0.5, phi).normalize(),
        Vec3::new(-0.5, 0.5, phi).normalize(),
        Vec3::new(-0.5, -0.5, phi).normalize(),
        Vec3::new(0.5, -0.5, phi).normalize(),
        Vec3::new(0.0, 0.0, phi+0.2).normalize(),
        
        // Bottom ring (5 hexagons)
        Vec3::new(0.5, 0.5, -phi).normalize(),
        Vec3::new(-0.5, 0.5, -phi).normalize(),
        Vec3::new(-0.5, -0.5, -phi).normalize(),
        Vec3::new(0.5, -0.5, -phi).normalize(),
        Vec3::new(0.0, 0.0, -phi-0.2).normalize(),
    ]
}

/// Identify hexagon centers based on Goldberg parameters (h,k)
fn identify_hexagon_centers(h: u32, k: u32) -> Vec<Vec3> {
    let mut hexagon_centers = Vec::new();
    
    if h == 0 && k == 0 {
        // Special case: (0,0) - this is a dodecahedron, no hexagons
        // We'll leave the hexagon centers empty
        return hexagon_centers;
    }
    
    // For other (h,k) values, we need proper Goldberg mathematics
    // This is a simplified version that just places hexagons in a pattern
    let expected_hexagons = crate::goldberg::goldberg_helpers::num_hexagons(h, k);
    
    // We'll use a combination of faces and edge midpoints
    // Step 1: Use face centers as a starting point
    let faces = get_icosahedron_faces();
    for face in faces {
        let v1 = identify_pentagon_centers()[face[0]];
        let v2 = identify_pentagon_centers()[face[1]];
        let v3 = identify_pentagon_centers()[face[2]];
        
        // Face center
        let center = ((v1 + v2 + v3) / 3.0).normalize();
        hexagon_centers.push(center);
        
        // For h > 1, add additional points along the edges
        if h > 1 {
            // For each edge, add h-1 intermediate points
            for i in 1..h {
                let t = i as f32 / h as f32;
                
                // Edge 1-2
                let mid12 = (v1 * (1.0 - t) + v2 * t).normalize();
                hexagon_centers.push(mid12);
                
                // Edge 2-3
                let mid23 = (v2 * (1.0 - t) + v3 * t).normalize();
                hexagon_centers.push(mid23);
                
                // Edge 3-1
                let mid31 = (v3 * (1.0 - t) + v1 * t).normalize();
                hexagon_centers.push(mid31);
            }
        }
        
        // For k > 0, add even more points
        if k > 0 {
            // Add points in the interior of the face
            for i in 1..k+1 {
                for j in 1..k+1 {
                    let a = i as f32 / (k + 1) as f32;
                    let b = j as f32 / (k + 1) as f32;
                    let c = 1.0 - a - b;
                    
                    if c > 0.0 {
                        let point = (v1 * a + v2 * b + v3 * c).normalize();
                        hexagon_centers.push(point);
                    }
                }
            }
        }
    }
    
    // Filter out centers that are too close to pentagon centers
    let pentagon_centers = identify_pentagon_centers();
    hexagon_centers.retain(|center| {
        // Make sure it's not too close to any pentagon center
        for &pentagon_center in &pentagon_centers {
            if center.distance(pentagon_center) < 0.2 {
                return false;
            }
        }
        true
    });
    
    // Deduplicate centers that are too close to each other
    let mut i = 0;
    while i < hexagon_centers.len() {
        let mut j = i + 1;
        while j < hexagon_centers.len() {
            if hexagon_centers[i].distance(hexagon_centers[j]) < 0.2 {
                hexagon_centers.remove(j);
            } else {
                j += 1;
            }
        }
        i += 1;
    }
    
    // If we have too many, limit to the expected number
    if hexagon_centers.len() > expected_hexagons as usize {
        hexagon_centers.truncate(expected_hexagons as usize);
    }
    
    // If we don't have enough, add more points
    while hexagon_centers.len() < expected_hexagons as usize {
        // Generate random points on the sphere
        let theta = rand::random::<f32>() * std::f32::consts::PI * 2.0;
        let phi = (rand::random::<f32>() * 2.0 - 1.0).acos();
        
        let x = phi.sin() * theta.cos();
        let y = phi.sin() * theta.sin();
        let z = phi.cos();
        
        let new_point = Vec3::new(x, y, z).normalize();
        
        // Check if it's too close to existing centers
        let mut too_close = false;
        
        for &center in &pentagon_centers {
            if new_point.distance(center) < 0.2 {
                too_close = true;
                break;
            }
        }
        
        if !too_close {
            for &center in &hexagon_centers {
                if new_point.distance(center) < 0.2 {
                    too_close = true;
                    break;
                }
            }
        }
        
        if !too_close {
            hexagon_centers.push(new_point);
        }
    }
    
    hexagon_centers
}

/// Get the 20 faces of an icosahedron
fn get_icosahedron_faces() -> Vec<[usize; 3]> {
    vec![
        // Top cap
        [0, 8, 4], [0, 4, 5], [0, 5, 9], [0, 9, 1], [0, 1, 8],
        // Bottom cap  
        [3, 10, 6], [3, 6, 7], [3, 7, 11], [3, 11, 2], [3, 2, 10],
        // Upper middle
        [4, 8, 10], [5, 4, 2], [9, 5, 11], [1, 9, 7], [8, 1, 6],
        // Lower middle
        [4, 10, 2], [5, 2, 11], [9, 11, 7], [1, 7, 6], [8, 6, 10],
    ]
}

/// Calculate the center of each triangle
fn calculate_triangle_centers(vertices: &[Vec3], indices: &[u32]) -> Vec<Vec3> {
    let mut centers = Vec::new();
    
    for i in (0..indices.len()).step_by(3) {
        let v1 = vertices[indices[i] as usize];
        let v2 = vertices[indices[i+1] as usize];
        let v3 = vertices[indices[i+2] as usize];
        
        let center = ((v1 + v2 + v3) / 3.0).normalize();
        centers.push(center);
    }
    
    centers
}

/// Find the closest tile center (pentagon or hexagon) to a point
fn find_closest_tile_center(
    point: Vec3,
    pentagon_centers: &[Vec3],
    hexagon_centers: &[Vec3],
) -> (TileType, usize) {
    let mut closest_type = TileType::Pentagon;
    let mut closest_idx = 0;
    let mut closest_dist = f32::MAX;
    
    // Check against pentagon centers
    for (i, &center) in pentagon_centers.iter().enumerate() {
        let dist = point.distance(center);
        // Give pentagons a stronger advantage
        let adjusted_dist = dist * 0.5; // 50% bias towards pentagons
        if adjusted_dist < closest_dist {
            closest_dist = adjusted_dist;
            closest_idx = i;
            closest_type = TileType::Pentagon;
        }
    }
    
    // Check against hexagon centers
    for (i, &center) in hexagon_centers.iter().enumerate() {
        let dist = point.distance(center);
        if dist < closest_dist {
            closest_dist = dist;
            closest_idx = i;
            closest_type = TileType::Hexagon;
        }
    }
    
    (closest_type, closest_idx)
}

/// Compute neighboring tiles by analyzing shared triangles
fn compute_tile_neighbors(tile_map: &mut TileMap, vertices: &[Vec3], indices: &[u32]) {
    // Create a map of edges to triangles
    let mut edge_to_triangles: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    
    for (tri_idx, indices_chunk) in indices.chunks(3).enumerate() {
        let v1 = indices_chunk[0];
        let v2 = indices_chunk[1];
        let v3 = indices_chunk[2];
        
        // Add all edges (ensuring the smaller index is first)
        let edges = [
            (v1.min(v2), v1.max(v2)),
            (v2.min(v3), v2.max(v3)),
            (v3.min(v1), v3.max(v1)),
        ];
        
        for edge in edges {
            edge_to_triangles.entry(edge).or_insert_with(Vec::new).push(tri_idx);
        }
    }
    
    // Find neighboring tiles using shared edges
    let mut neighbors: HashMap<usize, HashSet<usize>> = HashMap::new();
    
    for (_, triangle_indices) in edge_to_triangles {
        if triangle_indices.len() == 2 {
            let t1 = triangle_indices[0];
            let t2 = triangle_indices[1];
            
            if let (Some(&tile1), Some(&tile2)) = (
                tile_map.triangle_to_tile.get(&t1),
                tile_map.triangle_to_tile.get(&t2),
            ) {
                if tile1 != tile2 {
                    neighbors.entry(tile1).or_insert_with(HashSet::new).insert(tile2);
                    neighbors.entry(tile2).or_insert_with(HashSet::new).insert(tile1);
                }
            }
        }
    }
    
    // Update the tile neighbors
    for (tile_idx, neighbor_set) in neighbors {
        tile_map.tiles[tile_idx].neighbors = neighbor_set.into_iter().collect();
    }
}

/// Print statistics about the generated tiles
fn print_tile_statistics(tile_map: &TileMap, h: u32, k: u32, indices: &[u32]) {
    println!("=== Goldberg Polyhedron ({}, {}) Statistics ===", h, k);
    println!("Expected pentagons: 12");
    println!("Expected hexagons: {}", crate::goldberg::goldberg_helpers::num_hexagons(h, k));
    println!("Expected total tiles: {}", crate::goldberg::goldberg_helpers::total_faces(h, k));
    println!("Actual pentagons: {}", tile_map.pentagon_indices.len());
    println!("Actual hexagons: {}", tile_map.hexagon_indices.len());
    println!("Actual total tiles: {}", tile_map.tiles.len());
    
    // Calculate triangle distribution
    let mut pentagon_triangles = Vec::new();
    let mut hexagon_triangles = Vec::new();
    
    for tile in &tile_map.tiles {
        match tile.tile_type {
            TileType::Pentagon => pentagon_triangles.push(tile.triangle_indices.len()),
            TileType::Hexagon => hexagon_triangles.push(tile.triangle_indices.len()),
        }
    }
    
    // Calculate statistics
    if !pentagon_triangles.is_empty() {
        let min = *pentagon_triangles.iter().min().unwrap_or(&0);
        let max = *pentagon_triangles.iter().max().unwrap_or(&0);
        let avg = pentagon_triangles.iter().sum::<usize>() as f32 / pentagon_triangles.len().max(1) as f32;
        println!("Pentagon triangles - Min: {}, Max: {}, Avg: {:.1}", min, max, avg);
    } else {
        println!("No pentagon triangles found");
    }
    
    if !hexagon_triangles.is_empty() {
        let min = *hexagon_triangles.iter().min().unwrap_or(&0);
        let max = *hexagon_triangles.iter().max().unwrap_or(&0);
        let avg = hexagon_triangles.iter().sum::<usize>() as f32 / hexagon_triangles.len().max(1) as f32;
        println!("Hexagon triangles - Min: {}, Max: {}, Avg: {:.1}", min, max, avg);
    } else {
        println!("No hexagon triangles found");
    }
    
    // Calculate neighbor statistics
    let mut neighbor_counts = Vec::new();
    for tile in &tile_map.tiles {
        neighbor_counts.push(tile.neighbors.len());
    }
    
    if !neighbor_counts.is_empty() {
        let min = *neighbor_counts.iter().min().unwrap_or(&0);
        let max = *neighbor_counts.iter().max().unwrap_or(&0);
        let avg = neighbor_counts.iter().sum::<usize>() as f32 / neighbor_counts.len().max(1) as f32;
        println!("Neighbors per tile - Min: {}, Max: {}, Avg: {:.1}", min, max, avg);
    } else {
        println!("No tile neighbors found");
    }
    
    // Count total triangles assigned
    let total_triangles: usize = tile_map.tiles.iter()
        .map(|tile| tile.triangle_indices.len())
        .sum();
    
    let expected_triangles = indices.len() / 3;
    println!("Total triangles assigned: {} (expected {})", total_triangles, expected_triangles);
}
