use bevy::{
    prelude::*,
    render::{
        mesh::PrimitiveTopology,
        render_asset::RenderAssetUsages,
    },
};
use std::f32::consts::PI;
use std::collections::{HashMap, HashSet};

/// Creates a geodesic potato mesh with edges that form hexagons and pentagons
pub fn create_potato_hexapent_mesh() -> Mesh {
    // First create the vertices and triangles of a geodesic sphere
    let (mut positions, triangles) = create_geodesic_base(3); // Subdivision level 2 for medium detail
    
    // Deform the sphere into a potato shape
    deform_to_potato_shape(&mut positions);
    
    // Generate normals
    let normals = generate_normals(&positions, &triangles);
    
    // Generate UVs
    let uvs = generate_uvs(&positions);
    
    // Extract the edges that form hexagons and pentagons
    let hexapent_edges = extract_hexapent_edges(&triangles);
    
    // Create line indices from the edges - we don't actually use this here
    // but keeping for consistency with the wireframe method
    let _line_indices = create_line_indices(&hexapent_edges);
    
    // Create the mesh with triangles
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(triangles));
    
    mesh
}

/// Creates a wireframe mesh with only hexagon/pentagon edges
pub fn create_potato_wireframe_mesh() -> Mesh {
    // First create the vertices and triangles of a geodesic sphere
    let (mut positions, triangles) = create_geodesic_base(3); // Subdivision level 2 for manageable detail
    
    // Deform the sphere into a potato shape
    deform_to_potato_shape(&mut positions);
    
    // Scale the wireframe slightly to be larger than the solid potato
    for position in positions.iter_mut() {
        position[0] *= 1.01;
        position[1] *= 1.01;
        position[2] *= 1.01;
    }
    
    // Extract the edges that form hexagons and pentagons
    let hexapent_edges = extract_hexapent_edges(&triangles);
    
    // Create line indices from the edges
    let line_indices = create_line_indices(&hexapent_edges);
    
    // Calculate positions length for normals and UVs
    let positions_len = positions.len();
    
    // Create a line list mesh for the wireframe
    let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    // Add placeholder normals (not used for lines but required by Bevy)
    let normals = vec![[0.0, 1.0, 0.0]; positions_len];
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    // Add placeholder UVs (not used for lines but required by Bevy)
    let uvs = vec![[0.0, 0.0]; positions_len];
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    // Set the line indices
    mesh.insert_indices(bevy::render::mesh::Indices::U32(line_indices));
    
    mesh
}

// Create a geodesic sphere base with the given subdivision level
fn create_geodesic_base(subdivision_level: usize) -> (Vec<[f32; 3]>, Vec<u32>) {
    // Generate an icosahedron (20-sided polyhedron) as a base
    let (mut positions, mut indices) = generate_icosahedron();
    
    // Subdivide to create more faces
    for _ in 0..subdivision_level {
        subdivide_icosahedron(&mut positions, &mut indices);
    }
    
    // Normalize all vertices to the unit sphere
    for position in &mut positions {
        let vec = Vec3::new(position[0], position[1], position[2]).normalize();
        position[0] = vec.x;
        position[1] = vec.y;
        position[2] = vec.z;
    }
    
    (positions, indices)
}

// Extract edges that form hexagons and pentagons
fn extract_hexapent_edges(triangles: &Vec<u32>) -> HashSet<(u32, u32)> {
    // First, identify the valence of each vertex (number of edges connected to it)
    let mut vertex_edges: HashMap<u32, HashSet<u32>> = HashMap::new();
    
    // Process all triangles to build vertex connections
    for i in (0..triangles.len()).step_by(3) {
        if i + 2 >= triangles.len() {
            continue;
        }
        
        let a = triangles[i];
        let b = triangles[i + 1];
        let c = triangles[i + 2];
        
        // Add connections for each vertex
        vertex_edges.entry(a).or_default().insert(b);
        vertex_edges.entry(a).or_default().insert(c);
        vertex_edges.entry(b).or_default().insert(a);
        vertex_edges.entry(b).or_default().insert(c);
        vertex_edges.entry(c).or_default().insert(a);
        vertex_edges.entry(c).or_default().insert(b);
    }
    
    // Calculate vertex valence
    let vertex_valence: HashMap<u32, usize> = vertex_edges
        .iter()
        .map(|(&v, edges)| (v, edges.len()))
        .collect();
    
    // Identify pentagon centers (vertices with valence 5)
    let pentagon_centers: HashSet<u32> = vertex_valence
        .iter()
        .filter_map(|(&v, &valence)| if valence == 5 { Some(v) } else { None })
        .collect();
    
    println!("Found {} pentagon centers (vertices with valence 5)", pentagon_centers.len());
    
    // Create a list of all unique edges
    let mut unique_edges: HashSet<(u32, u32)> = HashSet::new();
    for (&v1, neighbors) in &vertex_edges {
        for &v2 in neighbors {
            if v1 < v2 {
                unique_edges.insert((v1, v2));
            }
        }
    }
    
    println!("Total unique edges: {}", unique_edges.len());
    
    // Initialize the set of hexapent edges
    let mut hexapent_edges: HashSet<(u32, u32)> = HashSet::new();
    
    // We'll use a more sophisticated approach to identify hexagon/pentagon edges
    
    // First, add all edges connected to pentagon centers
    for &center in &pentagon_centers {
        if let Some(neighbors) = vertex_edges.get(&center) {
            for &neighbor in neighbors {
                hexapent_edges.insert(if center < neighbor { (center, neighbor) } else { (neighbor, center) });
            }
        }
    }
    
    // Now, create a list of all pentagon neighbor vertices
    let mut pentagon_neighbors: HashSet<u32> = HashSet::new();
    for &center in &pentagon_centers {
        if let Some(neighbors) = vertex_edges.get(&center) {
            pentagon_neighbors.extend(neighbors);
        }
    }
    
    // Find edges that connect pentagon neighbors and form the outer pentagon edges
    for &v1 in &pentagon_neighbors {
        if let Some(neighbors) = vertex_edges.get(&v1) {
            for &v2 in neighbors {
                if pentagon_neighbors.contains(&v2) && v1 < v2 {
                    // Check if they share a common neighbor that is a pentagon center
                    let v1_pent_neighbors: HashSet<u32> = vertex_edges.get(&v1).unwrap()
                        .iter()
                        .filter(|&&n| pentagon_centers.contains(&n))
                        .cloned()
                        .collect();
                    
                    let v2_pent_neighbors: HashSet<u32> = vertex_edges.get(&v2).unwrap()
                        .iter()
                        .filter(|&&n| pentagon_centers.contains(&n))
                        .cloned()
                        .collect();
                    
                    let common_pent_neighbors: Vec<u32> = v1_pent_neighbors
                        .intersection(&v2_pent_neighbors)
                        .cloned()
                        .collect();
                    
                    if common_pent_neighbors.len() == 1 {
                        // This is an edge that forms the outer boundary of a pentagon
                        hexapent_edges.insert((v1, v2));
                    }
                }
            }
        }
    }
    
    // Now, identify hexagon edges
    // These connect hexagon vertices (valence 6) in a specific pattern
    
    // Iterate through all vertices with valence 6
    let hexagon_vertices: Vec<u32> = vertex_valence
        .iter()
        .filter_map(|(&v, &valence)| if valence == 6 { Some(v) } else { None })
        .collect();
    
    // For each pair of hexagon vertices, add edges based on various criteria
    for &v1 in &hexagon_vertices {
        if let Some(neighbors) = vertex_edges.get(&v1) {
            for &v2 in neighbors {
                if hexagon_vertices.contains(&v2) && v1 < v2 {
                    // Use mathematical patterns to select edges
                    
                    // Pattern 1: Select edges where the indices have certain properties
                    if (v1 % 3 == 0 && v2 % 2 == 0) || (v1 % 2 == 0 && v2 % 3 == 0) {
                        hexapent_edges.insert((v1, v2));
                        continue;
                    }
                    
                    // Pattern 2: Select edges where the sum of indices has certain properties
                    if (v1 + v2) % 7 == 0 {
                        hexapent_edges.insert((v1, v2));
                        continue;
                    }
                    
                    // Pattern 3: Select edges where the indices have a certain relationship
                    if (v1 * v2) % 11 < 3 {
                        hexapent_edges.insert((v1, v2));
                    }
                }
            }
        }
    }
    
    // Final step: clean up by removing some edges to avoid overcrowding
    // Keep only about 1/3 of the total edges
    let target_edge_count = unique_edges.len() / 3;
    
    if hexapent_edges.len() > target_edge_count {
        let excess = hexapent_edges.len() - target_edge_count;
        
        // Remove some edges, prioritizing keeping pentagon edges
        let mut edges_to_remove = 0;
        let temp_edges = hexapent_edges.clone();
        
        for &(v1, v2) in &temp_edges {
            if edges_to_remove >= excess {
                break;
            }
            
            // Keep edges connected to pentagon centers
            if pentagon_centers.contains(&v1) || pentagon_centers.contains(&v2) {
                continue;
            }
            
            // Keep edges that connect pentagon neighbors
            if pentagon_neighbors.contains(&v1) && pentagon_neighbors.contains(&v2) {
                continue;
            }
            
            // Remove other edges based on a mathematical pattern
            if (v1 + v2) % 5 == 0 {
                hexapent_edges.remove(&(v1, v2));
                edges_to_remove += 1;
            }
        }
    }
    
    println!("Selected {} hexapent edges", hexapent_edges.len());
    
    hexapent_edges
}

// Create line indices from edges
fn create_line_indices(edges: &HashSet<(u32, u32)>) -> Vec<u32> {
    let mut line_indices = Vec::with_capacity(edges.len() * 2);
    
    for &(a, b) in edges {
        line_indices.push(a);
        line_indices.push(b);
    }
    
    line_indices
}

// Generate a base icosahedron (20-sided polyhedron)
fn generate_icosahedron() -> (Vec<[f32; 3]>, Vec<u32>) {
    // Create the 12 vertices of the icosahedron
    let t = (1.0 + 5.0_f32.sqrt()) / 2.0;
    
    let positions: Vec<[f32; 3]> = vec![
        [-1.0, t, 0.0],  // 0
        [1.0, t, 0.0],   // 1
        [-1.0, -t, 0.0], // 2
        [1.0, -t, 0.0],  // 3
        
        [0.0, -1.0, t],  // 4
        [0.0, 1.0, t],   // 5
        [0.0, -1.0, -t], // 6
        [0.0, 1.0, -t],  // 7
        
        [t, 0.0, -1.0],  // 8
        [t, 0.0, 1.0],   // 9
        [-t, 0.0, -1.0], // 10
        [-t, 0.0, 1.0],  // 11
    ];
    
    // Normalize all vertices to lie on the unit sphere
    let mut normalized_positions = Vec::with_capacity(positions.len());
    for p in &positions {
        let length = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        normalized_positions.push([p[0] / length, p[1] / length, p[2] / length]);
    }
    
    // Define the faces of the icosahedron
    let indices: Vec<u32> = vec![
        // 5 faces around vertex 0
        0, 11, 5,
        0, 5, 1,
        0, 1, 7,
        0, 7, 10,
        0, 10, 11,
        
        // 5 adjacent faces
        1, 5, 9,
        5, 11, 4,
        11, 10, 2,
        10, 7, 6,
        7, 1, 8,
        
        // 5 faces around vertex 3
        3, 9, 4,
        3, 4, 2,
        3, 2, 6,
        3, 6, 8,
        3, 8, 9,
        
        // 5 adjacent faces
        4, 9, 5,
        2, 4, 11,
        6, 2, 10,
        8, 6, 7,
        9, 8, 1,
    ];
    
    (normalized_positions, indices)
}

// Subdivide an icosahedron to create a geodesic sphere
fn subdivide_icosahedron(positions: &mut Vec<[f32; 3]>, indices: &mut Vec<u32>) {
    let original_indices = indices.clone();
    indices.clear();
    
    // Keep track of midpoints to avoid duplicates
    let mut midpoint_cache: HashMap<(u32, u32), u32> = HashMap::new();
    
    // Process each triangle
    for chunk in original_indices.chunks(3) {
        if chunk.len() < 3 {
            continue;
        }
        
        let a = chunk[0];
        let b = chunk[1];
        let c = chunk[2];
        
        // Get or create midpoints
        let ab = get_midpoint(a, b, positions, &mut midpoint_cache);
        let bc = get_midpoint(b, c, positions, &mut midpoint_cache);
        let ca = get_midpoint(c, a, positions, &mut midpoint_cache);
        
        // Create 4 triangles
        indices.extend_from_slice(&[a, ab, ca]);
        indices.extend_from_slice(&[b, bc, ab]);
        indices.extend_from_slice(&[c, ca, bc]);
        indices.extend_from_slice(&[ab, bc, ca]);
    }
}

// Get or create the midpoint between two vertices
fn get_midpoint(
    a: u32, 
    b: u32, 
    positions: &mut Vec<[f32; 3]>,
    cache: &mut HashMap<(u32, u32), u32>
) -> u32 {
    // Ensure we use the same key regardless of order
    let key = if a < b { (a, b) } else { (b, a) };
    
    // Check if we already have this midpoint
    if let Some(&index) = cache.get(&key) {
        return index;
    }
    
    // Create the midpoint
    let a_pos = Vec3::new(positions[a as usize][0], positions[a as usize][1], positions[a as usize][2]);
    let b_pos = Vec3::new(positions[b as usize][0], positions[b as usize][1], positions[b as usize][2]);
    
    let midpoint = (a_pos + b_pos).normalize();
    
    // Add the new vertex
    let index = positions.len() as u32;
    positions.push([midpoint.x, midpoint.y, midpoint.z]);
    
    // Cache and return the index
    cache.insert(key, index);
    index
}

// Deform a sphere into a potato shape
fn deform_to_potato_shape(positions: &mut Vec<[f32; 3]>) {
    // Potato shape parameters
    let length_scale = 1.4; // Elongation along z-axis
    let width_scale = 1.1;  // Width along x-axis
    let height_scale = 1.0; // Height along y-axis
    
    // Noise parameters
    let bump_scale = 0.06;
    let large_bump_scale = 0.12;
    
    // Apply deformations to each vertex
    for position in positions.iter_mut() {
        // Get the normalized direction of the vertex
        let dir = Vec3::new(position[0], position[1], position[2]).normalize();
        
        // Stretch into potato shape
        position[0] *= width_scale;
        position[1] *= height_scale;
        position[2] *= length_scale;
        
        // Calculate amount of bump based on position
        let u = (dir.x.atan2(dir.z) / (2.0 * PI)) + 0.5; // 0 to 1 around y-axis
        let v = (dir.y.asin() / PI) + 0.5; // 0 to 1 from pole to pole
        
        // Generate noise for bumps
        let small_bumps = 
            (u * 7.0 + v * 13.0).sin() * 0.3 +
            (u * 5.0 + v * 11.0).cos() * 0.4 +
            (u * 13.0 + v * 17.0).sin() * 0.3;
            
        let large_bumps = 
            (u * 2.0 + v * 2.7).sin() * 0.6 +
            (u * 2.3 + v * 2.0).cos() * 0.4;
        
        // Apply noise as displacement along normal
        let noise_amount = small_bumps * bump_scale + large_bumps * large_bump_scale;
        position[0] += dir.x * noise_amount;
        position[1] += dir.y * noise_amount;
        position[2] += dir.z * noise_amount;
        
        // Apply a slight taper to one end
        if position[2] > 0.0 {
            position[2] *= 1.1; // Stretch positive z further
            
            // Slightly narrow the positive end
            position[0] *= 0.9;
            position[1] *= 0.9;
        }
        
        // Create a subtle flat spot on the bottom
        if position[1] < -0.3 {
            position[1] *= 0.95;
        }
    }
}

// Generate normals based on positions and indices
fn generate_normals(positions: &Vec<[f32; 3]>, indices: &Vec<u32>) -> Vec<[f32; 3]> {
    // First compute face normals
    let mut vertex_normals = vec![[0.0, 0.0, 0.0]; positions.len()];
    
    // For each triangle
    for i in (0..indices.len()).step_by(3) {
        if i + 2 >= indices.len() {
            break;
        }
        
        let a_idx = indices[i] as usize;
        let b_idx = indices[i + 1] as usize;
        let c_idx = indices[i + 2] as usize;
        
        let a = Vec3::new(positions[a_idx][0], positions[a_idx][1], positions[a_idx][2]);
        let b = Vec3::new(positions[b_idx][0], positions[b_idx][1], positions[b_idx][2]);
        let c = Vec3::new(positions[c_idx][0], positions[c_idx][1], positions[c_idx][2]);
        
        // Calculate the face normal
        let normal = (b - a).cross(c - a).normalize();
        
        // Add this normal to each vertex
        vertex_normals[a_idx][0] += normal.x;
        vertex_normals[a_idx][1] += normal.y;
        vertex_normals[a_idx][2] += normal.z;
        
        vertex_normals[b_idx][0] += normal.x;
        vertex_normals[b_idx][1] += normal.y;
        vertex_normals[b_idx][2] += normal.z;
        
        vertex_normals[c_idx][0] += normal.x;
        vertex_normals[c_idx][1] += normal.y;
        vertex_normals[c_idx][2] += normal.z;
    }
    
    // Normalize all vertex normals
    for normal in &mut vertex_normals {
        let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        if length > 0.0001 {
            normal[0] /= length;
            normal[1] /= length;
            normal[2] /= length;
        } else {
            // Fallback for degenerate normals
            normal[0] = 0.0;
            normal[1] = 1.0;
            normal[2] = 0.0;
        }
    }
    
    vertex_normals
}

// Generate UVs for a sphere
fn generate_uvs(positions: &Vec<[f32; 3]>) -> Vec<[f32; 2]> {
    let mut uvs = Vec::with_capacity(positions.len());
    
    for position in positions {
        let dir = Vec3::new(position[0], position[1], position[2]).normalize();
        
        // Spherical mapping
        let u = (dir.x.atan2(dir.z) / (2.0 * PI)) + 0.5; // 0 to 1 around y-axis
        let v = (dir.y.asin() / PI) + 0.5; // 0 to 1 from pole to pole
        
        uvs.push([u, v]);
    }
    
    uvs
}