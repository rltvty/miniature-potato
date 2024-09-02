use bevy::prelude::*;
use bevy::render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub fn setup_wind_turbines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Spawn multiple wind turbines at different positions
    spawn_wind_turbine(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut images,
        Vec3::new(0.0, 0.0, 0.0),
        1.0,
    );
    spawn_wind_turbine(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut images,
        Vec3::new(3.0, 0.0, 10.0),
        1.2,
    );
    spawn_wind_turbine(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut images,
        Vec3::new(-3.0, 0.0, -10.0),
        0.8,
    );
}

fn spawn_wind_turbine(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    position: Vec3,
    rotation_speed: f32,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    // Tower (Cylinder)
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Cylinder {
            radius: 0.3,
            half_height: 4.0,
            ..Default::default()
        })),
        material: debug_material.clone(),
        transform: Transform::from_translation(position + Vec3::new(0.0, 4.0, 0.0)),
        ..Default::default()
    });

    // Nacelle (Cube)
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Cuboid {
            half_size: Vec3::new(0.5, 0.5, 1.0),
        })),
        material: debug_material.clone(),
        transform: Transform {
            translation: position + Vec3::new(0.0, 8.5, 0.0),
            ..Default::default()
        },
        ..Default::default()
    });

    // Blades (Cylinder)
    let blade_length = 4.0;
    let blade_thickness = 0.1;
    let blade_axis_position = position + Vec3::new(0.0, 8.5, 1.1);

    for i in 0..3 {
        let angle = (i as f32) * (2.0 * std::f32::consts::PI / 3.0);

        let blade_offset = Vec3::new(0.0, blade_length / 2.0, 0.0); // Offset the blade by half its length
        let rotated_offset = Quat::from_rotation_z(angle) * blade_offset; // Apply rotation to the offset

        let blade_transform = Transform {
            translation: blade_axis_position + rotated_offset,
            rotation: Quat::from_rotation_z(angle),
            ..Default::default()
        };

        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(Cylinder {
                    radius: blade_thickness,
                    half_height: blade_length / 2.0,
                    ..Default::default()
                })),
                material: debug_material.clone(),
                transform: blade_transform,
                ..Default::default()
            })
            .insert(Blade) // Insert Blade component
            .insert(RotationSpeed(rotation_speed)); // Assign rotation speed to the blade
    }
}

#[derive(Debug, Component)]
pub struct Blade;

#[derive(Component)]
pub struct RotationSpeed(f32);

pub fn rotate_blades(
    time: Res<Time>,
    mut query: Query<(&RotationSpeed, &mut Transform), With<Blade>>,
) {
    for (rotation_speed, mut transform) in query.iter_mut() {
        let delta_rotation = Quat::from_rotation_z(time.delta_seconds() * rotation_speed.0);

        // Calculate the pivot point (the end of the blade)
        let pivot = transform.translation - transform.rotation * Vec3::new(0.0, 2.0, 0.0);

        // Rotate around the pivot point
        transform.rotate_around(pivot, delta_rotation);
    }
}

/// Creates a colorful test pattern
pub fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
