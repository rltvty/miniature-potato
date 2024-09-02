use bevy::prelude::*;
use bevy::render::{render_asset::RenderAssetUsages, render_resource::PrimitiveTopology};
use rand::Rng;

pub struct PotatoPlugin;

impl Plugin for PotatoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let potato_mesh = generate_potato_mesh(64, 16, 0.05, 2.0);

    commands.spawn(PbrBundle {
        mesh: meshes.add(potato_mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.5, 0.3),
            ..Default::default()
        }),
        transform: Transform::from_xyz(4.0, 4.0, 4.0),
        ..Default::default()
    });
}

fn generate_potato_mesh(
    longitude_segments: usize,
    latitude_segments: usize,
    noise_factor: f32,
    elongation_factor: f32,
) -> Mesh {
    let mut rng = rand::thread_rng();
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=latitude_segments {
        let theta = i as f32 * std::f32::consts::PI / latitude_segments as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        // Scale noise based on latitude to reduce spikes at the poles
        let latitude_noise_scale = (1.0 - cos_theta.abs()).powf(0.5);

        for j in 0..=longitude_segments {
            let phi = j as f32 * 2.0 * std::f32::consts::PI / longitude_segments as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            // Apply elongation on the x-axis
            let x = cos_phi * sin_theta * elongation_factor;
            let y = cos_theta;
            let z = sin_phi * sin_theta;

            // Apply the latitude-scaled noise
            let radius = 1.0 + rng.gen_range(-noise_factor..noise_factor) * latitude_noise_scale;

            vertices.push([x * radius, y * radius, z * radius]);
        }
    }

    for i in 0..latitude_segments {
        for j in 0..longitude_segments {
            let first = i * (longitude_segments + 1) + j;
            let second = first + longitude_segments + 1;

            indices.push(first as u32);
            indices.push(second as u32);
            indices.push((first + 1) as u32);

            indices.push(second as u32);
            indices.push((second + 1) as u32);
            indices.push((first + 1) as u32);
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
