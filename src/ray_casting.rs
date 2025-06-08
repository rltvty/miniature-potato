//! Ray casting system for mouse picking on the icosphere - Bevy 0.16.1 compatible

use bevy::{
    picking::backend::ray::RayMap,
    prelude::*,
};

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

/// Ray casting system using Bevy 0.16.1 MeshRayCast
pub fn ray_casting_system(
    mut ray_cast: MeshRayCast,
    mut hovered: ResMut<HoveredTriangle>,
    ray_map: Res<RayMap>,
    icosphere_query: Query<(Entity, &IcosphereTriangles)>,
) {
    // Clear previous hover state
    hovered.triangle_index = None;
    hovered.hit_point = None;
    hovered.hit_normal = None;

    // Get the cursor ray from RayMap (automatically managed by Bevy)
    for (_camera_entity, ray) in ray_map.iter() {
        // Cast the ray and get hits
        let hits = ray_cast.cast_ray(*ray, &MeshRayCastSettings::default());
        
        if let Some((entity, hit)) = hits.first() {
            // Check if this hit is on our icosphere
            if let Ok((_entity, triangles)) = icosphere_query.get(*entity) {
                // Find which triangle was hit by finding the closest triangle center
                let hit_point = hit.point;
                let mut closest_distance = f32::INFINITY;
                let mut closest_triangle = None;

                for (triangle_idx, triangle) in triangles.indices.chunks(3).enumerate() {
                    let v0 = triangles.vertices[triangle[0] as usize];
                    let v1 = triangles.vertices[triangle[1] as usize];
                    let v2 = triangles.vertices[triangle[2] as usize];
                    
                    // Calculate triangle center
                    let triangle_center = (v0 + v1 + v2) / 3.0;
                    let distance = hit_point.distance(triangle_center);
                    
                    if distance < closest_distance {
                        closest_distance = distance;
                        closest_triangle = Some(triangle_idx);
                    }
                }

                hovered.triangle_index = closest_triangle;
                hovered.hit_point = Some(hit.point);
                hovered.hit_normal = Some(hit.normal);
            }
        }
    }
}

/// Debug system to print hovered triangle info
pub fn debug_hovered_triangle(
    hovered: Res<HoveredTriangle>,
    mut last_triangle: Local<Option<usize>>,
) {
    if hovered.triangle_index != *last_triangle {
        if let Some(triangle_idx) = hovered.triangle_index {
            println!("Hovering over triangle: {}", triangle_idx);
            if let Some(hit_point) = hovered.hit_point {
                println!("  Hit point: {:.2}", hit_point);
            }
            if let Some(hit_normal) = hovered.hit_normal {
                println!("  Hit normal: {:.2}", hit_normal);
            }
        } else {
            println!("No triangle hovered");
        }
        *last_triangle = hovered.triangle_index;
    }
}

/// System to draw a small sphere at the hit point for visual feedback
pub fn visualize_hit_point(
    mut gizmos: Gizmos,
    hovered: Res<HoveredTriangle>,
) {
    if let Some(hit_point) = hovered.hit_point {
        gizmos.sphere(hit_point, 0.05, Color::srgb(1.0, 0.0, 0.0));
    }
}
