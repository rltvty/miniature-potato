use bevy::{
    prelude::*,
    render::{
        mesh::PrimitiveTopology,
        render_asset::RenderAssetUsages,
    },
};
use std::f32::consts::PI;

pub fn create_geodesic_potato_mesh() -> Mesh {
    // Create a geodesic dome (icosahedron-based) and deform it into a potato shape
    
    // Generate an icosahedron (20-sided polyhedron) as a base
    let (mut positions, mut indices) = generate_icosahedron();
    
    // Subdivide to create more faces (geodesic sphere)
    // Higher subdivision levels create more detailed meshes
    // Level 1 = 80 faces, Level 2 = 320 faces, Level 3 = 1280 faces
    let subdivision_level = 2; // Medium detail level
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
    
    // Deform the sphere into a potato shape
    deform_to_potato_shape(&mut positions);
    
    // Generate normals
    let normals = generate_normals(&positions, &indices);
    
    // Generate UVs using spherical mapping
    let uvs = generate_uvs(&positions);
    
    // Create the mesh
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    
    mesh
}

// Generate a base icosahedron (20-sided polyhedron)
fn generate_icosahedron() -> (Vec<[f32; 3]>, Vec<u32>) {
    // Create the 12 vertices of the icosahedron
    let t = (1.0 + 5.0_f32.sqrt()) / 2.0;
    
    let mut positions: Vec<[f32; 3]> = vec![
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
    for p in &mut positions {
        let length = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        p[0] /= length;
        p[1] /= length;
        p[2] /= length;
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
    
    (positions, indices)
}

// Subdivide an icosahedron to create a geodesic sphere
fn subdivide_icosahedron(positions: &mut Vec<[f32; 3]>, indices: &mut Vec<u32>) {
    let original_indices = indices.clone();
    indices.clear();
    
    // Keep track of midpoints to avoid duplicates
    let mut midpoint_cache: std::collections::HashMap<(u32, u32), u32> = std::collections::HashMap::new();
    
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
    cache: &mut std::collections::HashMap<(u32, u32), u32>
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
    let mut uvs = Vec::new();
    
    for position in positions {
        let dir = Vec3::new(position[0], position[1], position[2]).normalize();
        
        // Spherical mapping
        let u = (dir.x.atan2(dir.z) / (2.0 * PI)) + 0.5; // 0 to 1 around y-axis
        let v = (dir.y.asin() / PI) + 0.5; // 0 to 1 from pole to pole
        
        uvs.push([u, v]);
    }
    
    uvs
}