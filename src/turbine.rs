use bevy::prelude::*;
use bevy::render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use player::Player;
use rand::Rng;

use crate::player;

#[derive(Resource)]
pub struct DropCooldown {
    timer: Timer,
}

impl Default for DropCooldown {
    fn default() -> Self {
        DropCooldown {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        }
    }
}

pub fn drop_wind_turbine(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cooldown: ResMut<DropCooldown>,
    mut query: Query<&Transform, With<Player>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(transform) = query.get_single_mut() else {
        return;
    };

    // Update the cooldown timer
    cooldown.timer.tick(time.delta());

    // Check if the key is pressed and if the timer has finished
    if keyboard.just_pressed(KeyCode::KeyT) && cooldown.timer.finished() {
        println!("Dropping Turbine at {}", transform.translation);

        let mut rng = rand::thread_rng();
        let random_float: f32 = rng.gen_range(0.5..1.0);

        spawn_wind_turbine(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut images,
            transform.translation
                + Vec3 {
                    x: 2.0,
                    y: -2.0,
                    z: 2.0,
                },
            random_float,
        );

        // Reset the timer after the turbine is dropped
        cooldown.timer.reset();
    }
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
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(Cylinder {
                radius: 3.0,
                half_height: 40.0,
                ..Default::default()
            })),
            material: debug_material.clone(),
            transform: Transform::from_translation(position + Vec3::new(0.0, 40.0, 0.0)),
            ..default()
        },
        // RigidBody::Dynamic,
        // Collider::cylinder(0.3, 8.0),
    ));

    // Nacelle (Cube)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid {
                half_size: Vec3::new(5.0, 5.0, 10.0),
            })),
            material: debug_material.clone(),
            transform: Transform {
                translation: position + Vec3::new(0.0, 85.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        },
        // RigidBody::Dynamic,
        // Collider::cuboid(1.0, 1.0, 2.0),
    ));

    // Blades (Cylinder)
    let blade_length = 40.0;
    let blade_thickness = 1.0;
    let blade_axis_position = position + Vec3::new(0.0, 85.0, 11.0);

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
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(Cylinder {
                        radius: blade_thickness,
                        half_height: blade_length / 2.0,
                        ..Default::default()
                    })),
                    material: debug_material.clone(),
                    transform: blade_transform,
                    ..Default::default()
                },
                // RigidBody::Dynamic,
                // Collider::cylinder(blade_thickness, blade_length),
            ))
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
        let pivot = transform.translation - transform.rotation * Vec3::new(0.0, 20.0, 0.0);

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
