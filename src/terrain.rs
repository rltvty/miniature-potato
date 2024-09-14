use bevy::color::palettes::tailwind;
use bevy::render::{render_asset::RenderAssetUsages, render_resource::PrimitiveTopology};
use bevy::{color::palettes::css, prelude::*};

use avian3d::prelude::*;
use noise::{NoiseFn, Perlin};

fn generate_procedural_terrain_mesh(size: usize, scale: f64) -> Mesh {
    let perlin = Perlin::new(42);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices
    for z in 0..size {
        for x in 0..size {
            let height = perlin.get([x as f64 * scale, z as f64 * scale]) as f32;
            vertices.push([x as f32, height, z as f32]);
        }
    }

    // Generate indices
    for z in 0..(size - 1) {
        for x in 0..(size - 1) {
            let i = z * size + x;

            // First triangle of the quad
            indices.push(i as u32);
            indices.push((i + 1) as u32);
            indices.push((i + size) as u32);

            // Second triangle of the quad
            indices.push((i + 1) as u32);
            indices.push((i + size + 1) as u32);
            indices.push((i + size) as u32);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
    .with_duplicated_vertices()
    .with_computed_flat_normals()
}

pub fn setup_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Generate procedural terrain mesh
    let terrain_mesh = generate_procedural_terrain_mesh(100, 0.2);

    // Spawn terrain entity
    commands.spawn((
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
        //Collider::convex_decomposition_from_mesh(&terrain_mesh).unwrap(),
        PbrBundle {
            mesh: meshes.add(terrain_mesh),
            material: materials.add(Color::from(tailwind::LIME_500)),
            transform: Transform::from_xyz(-50.0, 0.0, -50.0),
            ..Default::default()
        },
    ));

    // Spawn a little platform for the player to jump on.
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(4.0, 1.0, 4.0)),
            material: materials.add(Color::from(css::GRAY)),
            transform: Transform::from_xyz(-6.0, 2.0, 0.0),
            ..Default::default()
        },
        RigidBody::Static,
        Collider::cuboid(4.0, 1.0, 4.0),
    ));
}
