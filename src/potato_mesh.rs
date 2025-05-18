use bevy::{
    prelude::*,
    render::{
        mesh::PrimitiveTopology,
        render_asset::RenderAssetUsages,
    },
};
use std::f32::consts::PI;

pub fn create_potato_mesh() -> Mesh {
    // Start with a basic ellipsoid shape, then apply noise and deformation
    // to create a more natural potato shape
    
    // Basic parameters
    let base_length = 2.0;
    let base_width = 1.1;
    let base_height = 1.0;
    let segments = 32;
    let rings = 16;
    
    // Noise parameters for the bumpy potato skin - reduced for a smoother potato
    let bump_scale = 0.03;  // Reduced from 0.08 
    let large_bump_scale = 0.06; // Reduced from 0.15
    let large_bump_frequency = 2.0;
    
    // We'll use a parametric approach to create a continuous surface
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Generate a base ellipsoid with custom proportions
    for ring_idx in 0..=rings {
        // v parameter goes from 0 to 1
        let v = ring_idx as f32 / rings as f32;
        
        // Convert to spherical coordinate phi (0 to PI)
        let phi = v * PI;
        
        // Add a slight asymmetry to the potato by shifting the phi angle
        // This makes one end more pointed than the other
        let modified_phi = if phi < PI/2.0 {
            phi * 0.8 // Compress one half slightly
        } else {
            PI - (PI - phi) * 1.2 // Stretch the other half
        };
        
        // Standard spherical coordinate calculations
        let sin_phi = modified_phi.sin();
        let cos_phi = modified_phi.cos();
        
        for segment_idx in 0..=segments {
            // u parameter goes from 0 to 1
            let u = segment_idx as f32 / segments as f32;
            
            // Convert to spherical coordinate theta (0 to 2*PI)
            let theta = u * 2.0 * PI;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            
            // Start with ellipsoid base shape, with different radii along each axis
            let mut x = base_width * sin_phi * cos_theta;
            let mut y = base_height * sin_phi * sin_theta;
            let mut z = base_length * cos_phi;
            
            // Apply a slight random deformation to the entire potato
            // using deterministic noise functions
            
            // Gradually reduce noise influence near the ends to avoid artifacts
            let pole_weight = (sin_phi).powf(0.7); // Higher power makes transition more gradual
            
            // Small bumps using several frequencies
            let small_bumps = 
                (sin_theta * 7.0 + cos_phi * 13.0).sin() * 0.3 +
                (cos_theta * 5.0 + sin_phi * 11.0).cos() * 0.4 +
                (sin_theta * 13.0 + cos_phi * 17.0).sin() * 0.3;
                
            // Add a few larger protrusions with lower frequency
            let large_bumps = 
                (sin_theta * large_bump_frequency + cos_phi * (large_bump_frequency + 0.7)).sin() * 0.6 +
                (cos_theta * (large_bump_frequency + 0.3) + sin_phi * large_bump_frequency).cos() * 0.4;
            
            // Combine the noise
            let small_noise = small_bumps * bump_scale * pole_weight;
            let large_noise = large_bumps * large_bump_scale * pole_weight;
            
            // Apply the noise to the base ellipsoid
            let noise_vector = Vec3::new(
                cos_theta * sin_phi, 
                sin_theta * sin_phi, 
                cos_phi
            ).normalize() * (small_noise + large_noise);
            
            x += noise_vector.x;
            y += noise_vector.y;
            z += noise_vector.z;
            
            // Apply a slight global asymmetry to make it more potato-like
            // Make one end slightly more pointed
            if z > 0.0 {
                z *= 1.1;
                y *= 0.95;
            }
            
            // Apply a very subtle flat spot on the "bottom" side
            if y < -0.2 {
                y *= 1.0 + (y * 0.2);
            }
            
            // Calculate the normal using central differences method 
            // for better accuracy with our deformed surface
            let epsilon = 0.001;
            
            // Sample neighboring points
            let u_plus = (u + epsilon).min(1.0);
            let u_minus = (u - epsilon).max(0.0);
            let v_plus = (v + epsilon).min(1.0);
            let v_minus = (v - epsilon).max(0.0);
            
            // Find positions of neighboring points
            let pos_u_plus = sample_point(u_plus, v, base_width, base_height, base_length, 
                                         bump_scale, large_bump_scale, large_bump_frequency);
            let pos_u_minus = sample_point(u_minus, v, base_width, base_height, base_length, 
                                         bump_scale, large_bump_scale, large_bump_frequency);
            let pos_v_plus = sample_point(u, v_plus, base_width, base_height, base_length, 
                                        bump_scale, large_bump_scale, large_bump_frequency);
            let pos_v_minus = sample_point(u, v_minus, base_width, base_height, base_length, 
                                         bump_scale, large_bump_scale, large_bump_frequency);
            
            // Calculate tangent vectors
            let tangent_u = Vec3::new(
                pos_u_plus.x - pos_u_minus.x,
                pos_u_plus.y - pos_u_minus.y,
                pos_u_plus.z - pos_u_minus.z,
            );
            
            let tangent_v = Vec3::new(
                pos_v_plus.x - pos_v_minus.x,
                pos_v_plus.y - pos_v_minus.y,
                pos_v_plus.z - pos_v_minus.z,
            );
            
            // Calculate normal as cross product of tangents
            let normal = tangent_u.cross(tangent_v).normalize();
            
            // Add vertex data
            positions.push([x, y, z]);
            normals.push([normal.x, normal.y, normal.z]);
            uvs.push([u, v]);
        }
    }

    // Create indices for triangles
    for ring in 0..rings {
        for segment in 0..segments {
            let current = ring * (segments + 1) + segment;
            let next_segment = current + 1;
            let next_ring = current + segments + 1;
            let next_ring_segment = next_ring + 1;

            // First triangle
            indices.push(current as u32);
            indices.push(next_segment as u32);
            indices.push(next_ring as u32);

            // Second triangle
            indices.push(next_segment as u32);
            indices.push(next_ring_segment as u32);
            indices.push(next_ring as u32);
        }
    }

    // ----- ADD END CAPS TO FIX HOLES -----
    
    // Add center vertices for the caps
    let top_center_idx = positions.len() as u32;
    let bottom_center_idx = top_center_idx + 1;
    
    // Get the position for the center points by averaging nearby points
    let mut top_center = Vec3::ZERO;
    let mut bottom_center = Vec3::ZERO;
    let mut top_normal = Vec3::ZERO;
    let mut bottom_normal = Vec3::ZERO;
    
    // Top center (the first ring)
    for segment in 0..=segments {
        let idx = segment;
        let pos = Vec3::new(positions[idx][0], positions[idx][1], positions[idx][2]);
        top_center += pos;
        
        let normal = Vec3::new(normals[idx][0], normals[idx][1], normals[idx][2]);
        top_normal += normal;
    }
    top_center /= (segments + 1) as f32;
    top_normal = top_normal.normalize();
    
    // Bottom center (the last ring)
    for segment in 0..=segments {
        let idx = rings * (segments + 1) + segment;
        let pos = Vec3::new(positions[idx][0], positions[idx][1], positions[idx][2]);
        bottom_center += pos;
        
        let normal = Vec3::new(normals[idx][0], normals[idx][1], normals[idx][2]);
        bottom_normal += normal;
    }
    bottom_center /= (segments + 1) as f32;
    bottom_normal = bottom_normal.normalize();
    
    // Add center vertices
    positions.push([top_center.x, top_center.y, top_center.z]);
    normals.push([top_normal.x, top_normal.y, top_normal.z]);
    uvs.push([0.5, 0.0]); // Center of the UV map
    
    positions.push([bottom_center.x, bottom_center.y, bottom_center.z]);
    normals.push([bottom_normal.x, bottom_normal.y, bottom_normal.z]);
    uvs.push([0.5, 1.0]); // Center of the UV map
    
    // Add cap triangles
    // Top cap (first ring)
    for segment in 0..segments {
        let current = segment;
        let next = segment + 1;
        
        indices.push(top_center_idx);
        indices.push(current as u32);
        indices.push(next as u32);
    }
    
    // Bottom cap (last ring)
    for segment in 0..segments {
        let current = rings * (segments + 1) + segment;
        let next = current + 1;
        
        indices.push(bottom_center_idx);
        indices.push(next as u32);
        indices.push(current as u32);
    }

    // Create the mesh
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    mesh
}

// A helper function to sample a point on our potato for normal calculation
fn sample_point(
    u: f32, 
    v: f32, 
    base_width: f32, 
    base_height: f32, 
    base_length: f32,
    bump_scale: f32,
    large_bump_scale: f32,
    large_bump_frequency: f32
) -> Vec3 {
    // Convert parameters to spherical coordinates
    let phi = v * PI;
    
    // Apply the same asymmetry as in the main function
    let modified_phi = if phi < PI/2.0 {
        phi * 0.8
    } else {
        PI - (PI - phi) * 1.2
    };
    
    let sin_phi = modified_phi.sin();
    let cos_phi = modified_phi.cos();
    let theta = u * 2.0 * PI;
    let sin_theta = theta.sin();
    let cos_theta = theta.cos();
    
    // Base ellipsoid
    let mut x = base_width * sin_phi * cos_theta;
    let mut y = base_height * sin_phi * sin_theta;
    let mut z = base_length * cos_phi;
    
    // Apply the same deformations as in the main function
    let pole_weight = (sin_phi).powf(0.7);
    
    let small_bumps = 
        (sin_theta * 7.0 + cos_phi * 13.0).sin() * 0.3 +
        (cos_theta * 5.0 + sin_phi * 11.0).cos() * 0.4 +
        (sin_theta * 13.0 + cos_phi * 17.0).sin() * 0.3;
        
    let large_bumps = 
        (sin_theta * large_bump_frequency + cos_phi * (large_bump_frequency + 0.7)).sin() * 0.6 +
        (cos_theta * (large_bump_frequency + 0.3) + sin_phi * large_bump_frequency).cos() * 0.4;
    
    let small_noise = small_bumps * bump_scale * pole_weight;
    let large_noise = large_bumps * large_bump_scale * pole_weight;
    
    let noise_vector = Vec3::new(
        cos_theta * sin_phi, 
        sin_theta * sin_phi, 
        cos_phi
    ).normalize() * (small_noise + large_noise);
    
    x += noise_vector.x;
    y += noise_vector.y;
    z += noise_vector.z;
    
    // Apply asymmetry
    if z > 0.0 {
        z *= 1.1;
        y *= 0.95;
    }
    
    // Apply flat spot
    if y < -0.2 {
        y *= 1.0 + (y * 0.2);
    }
    
    Vec3::new(x, y, z)
}

pub fn create_russet_potato_mesh() -> Mesh {
    // More elongated, with heavier bumps
    // Basic parameters
    let base_length = 2.2;
    let base_width = 1.0;
    let base_height = 0.9;
    let segments = 32;
    let rings = 16;
    
    // Noise parameters for the bumpy russet potato skin
    let bump_scale = 0.08;  // More bumps for russet
    let large_bump_scale = 0.15; 
    let large_bump_frequency = 2.5;
    
    // Use the same generation code but with different parameters
    create_custom_potato_mesh(
        base_length, base_width, base_height,
        segments, rings,
        bump_scale, large_bump_scale, large_bump_frequency,
        true // Add end caps
    )
}

pub fn create_red_potato_mesh() -> Mesh {
    // Rounder, with smoother skin
    // Basic parameters
    let base_length = 1.6;
    let base_width = 1.3;
    let base_height = 1.25;
    let segments = 32;
    let rings = 16;
    
    // Noise parameters for the smoother red potato skin
    let bump_scale = 0.02;  // Much smoother than russet
    let large_bump_scale = 0.04; 
    let large_bump_frequency = 1.8;
    
    // Use the same generation code but with different parameters
    create_custom_potato_mesh(
        base_length, base_width, base_height,
        segments, rings,
        bump_scale, large_bump_scale, large_bump_frequency,
        true // Add end caps
    )
}

// More generic function to create different types of potatoes with custom parameters
fn create_custom_potato_mesh(
    base_length: f32,
    base_width: f32,
    base_height: f32,
    segments: usize,
    rings: usize,
    bump_scale: f32,
    large_bump_scale: f32,
    large_bump_frequency: f32,
    add_end_caps: bool
) -> Mesh {
    // This is just a placeholder - for now it returns the default potato
    // You could implement this function with the same logic as create_potato_mesh
    // but using the parameters passed in
    
    // This is just a placeholder  
    create_potato_mesh()  
}
