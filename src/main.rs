use bevy::color::palettes::tailwind;
use bevy::input::mouse::MouseMotion;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
};


/// Player movement speed factor.
const PLAYER_SPEED: f32 = 10.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(
            Startup,
            (
                spawn_view_model,
                spawn_world_model,
                spawn_lights,
                spawn_text,
                setup_wind_turbines,
            ),
        )
        .add_systems(
            Update, (
                move_player, 
                change_fov, 
                quit_on_esc_system,
                rotate_blades,
            )
        )
        .run();
}

fn quit_on_esc_system(
    _: Commands,
    kb_input: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    // Check if the Escape key is pressed
    if kb_input.just_pressed(KeyCode::Escape) {
        // Send the exit event to quit the game
        exit.send(AppExit::Success);
    }
}

#[derive(Debug, Component)]
struct Player;

#[derive(Debug, Component)]
struct WorldModelCamera;

/// Used implicitly by all entities without a `RenderLayers` component.
/// Our world model camera and all objects other than the player are on this layer.
/// The light source belongs to both layers.
const DEFAULT_RENDER_LAYER: usize = 0;

/// Used by the view model camera and the player's arm.
/// The light source belongs to both layers.
const VIEW_MODEL_RENDER_LAYER: usize = 1;

fn spawn_view_model(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let arm = meshes.add(Cuboid::new(0.1, 0.1, 0.5));
    let arm_material = materials.add(Color::from(tailwind::TEAL_200));

    commands
        .spawn((
            Player,
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 1.0, 0.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                WorldModelCamera,
                Camera3dBundle {
                    projection: PerspectiveProjection {
                        fov: 90.0_f32.to_radians(),
                        ..default()
                    }
                    .into(),
                    ..default()
                },
            ));

            // Spawn view model camera.
            parent.spawn((
                Camera3dBundle {
                    camera: Camera {
                        // Bump the order to render on top of the world model.
                        order: 1,
                        ..default()
                    },
                    projection: PerspectiveProjection {
                        fov: 70.0_f32.to_radians(),
                        ..default()
                    }
                    .into(),
                    ..default()
                },
                // Only render objects belonging to the view model.
                RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
            ));

            // Spawn the player's right arm.
            parent.spawn((
                MaterialMeshBundle {
                    mesh: arm,
                    material: arm_material,
                    transform: Transform::from_xyz(0.2, -0.1, -0.25),
                    ..default()
                },
                // Ensure the arm is only rendered by the view model camera.
                RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
                // The arm is free-floating, so shadows would look weird.
                NotShadowCaster,
            ));
        });
}

fn spawn_world_model(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let floor = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10.0)));
    let cube = meshes.add(Cuboid::new(2.0, 0.5, 1.0));
    let material = materials.add(Color::WHITE);

    // The world model camera will render the floor and the cubes spawned in this system.
    // Assigning no `RenderLayers` component defaults to layer 0.

    commands.spawn(MaterialMeshBundle {
        mesh: floor,
        material: material.clone(),
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        mesh: cube.clone(),
        material: material.clone(),
        transform: Transform::from_xyz(0.0, 0.25, -3.0),
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        mesh: cube,
        material,
        transform: Transform::from_xyz(0.75, 1.75, 0.0),
        ..default()
    });
}

fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                color: Color::from(tailwind::ROSE_300),
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(-2.0, 4.0, -0.75),
            ..default()
        },
        // The light source illuminates both the world model and the view model.
        RenderLayers::from_layers(&[DEFAULT_RENDER_LAYER, VIEW_MODEL_RENDER_LAYER]),
    ));
}

fn spawn_text(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(12.0),
                left: Val::Px(12.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                concat!(
                    "Move the camera with your mouse.\n",
                    "Press - or _ to decrease the FOV of the world model.\n",
                    "Press + or = to increase the FOV of the world model."
                ),
                TextStyle {
                    font_size: 25.0,
                    ..default()
                },
            ));
        });
}

fn move_player(
    mut player: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
    mut mouse_motion: EventReader<MouseMotion>,
    kb_input: Res<ButtonInput<KeyCode>>,
    
) {
    let Ok(mut player) = player.get_single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;

    if kb_input.pressed(KeyCode::KeyW) {
        direction.z -= 1.;
    }

    if kb_input.pressed(KeyCode::KeyS) {
        direction.z += 1.;
    }

    if kb_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.;
    }

    if kb_input.pressed(KeyCode::KeyD) {
        direction.x += 1.;
    }

    // Progressively update the player's position over time. Normalize the
    // direction vector to prevent it from exceeding a magnitude of 1 when
    // moving diagonally.
    let move_delta = direction.normalize_or_zero() * PLAYER_SPEED * time.delta_seconds();

    // Get the player's forward direction vector (in the XZ plane)
    let forward = player.rotation * Vec3::Z;
    let right = player.rotation * Vec3::X;

    // Ignore the Y component (only consider X and Z)
    let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let right_xz = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

    // Calculate the movement in the XZ plane
    let move_delta = forward_xz * move_delta.z + right_xz * move_delta.x;
    
    // Apply the movement to the player's translation
    player.translation += move_delta;

    for motion in mouse_motion.read() {
        let yaw = -motion.delta.x * 0.003;
        let pitch = -motion.delta.y * 0.002;
        // Order of rotations is important, see <https://gamedev.stackexchange.com/a/136175/103059>
        player.rotate_y(yaw);
        player.rotate_local_x(pitch);
    }
}

fn change_fov(
    input: Res<ButtonInput<KeyCode>>,
    mut world_model_projection: Query<&mut Projection, With<WorldModelCamera>>,
) {
    let mut projection = world_model_projection.single_mut();
    let Projection::Perspective(ref mut perspective) = projection.as_mut() else {
        unreachable!(
            "The `Projection` component was explicitly built with `Projection::Perspective`"
        );
    };

    if input.pressed(KeyCode::Equal) {
        perspective.fov -= 1.0_f32.to_radians();
        perspective.fov = perspective.fov.max(20.0_f32.to_radians());
    }
    if input.pressed(KeyCode::Minus) {
        perspective.fov += 1.0_f32.to_radians();
        perspective.fov = perspective.fov.min(160.0_f32.to_radians());
    }
}

fn setup_wind_turbines(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut images: ResMut<Assets<Image>>) {
    // Spawn multiple wind turbines at different positions
    spawn_wind_turbine(&mut commands, &mut meshes, &mut materials, &mut images, Vec3::new(0.0, 0.0, 0.0), 1.0);
    spawn_wind_turbine(&mut commands, &mut meshes, &mut materials, &mut images, Vec3::new(3.0, 0.0, 10.0), 1.2);
    spawn_wind_turbine(&mut commands, &mut meshes, &mut materials, &mut images, Vec3::new(-3.0, 0.0, -10.0), 0.8);
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
        mesh: meshes.add(Mesh::from(Cylinder { radius: 0.3, half_height: 4.0, ..Default::default() })),
        material: debug_material.clone(),
        transform: Transform::from_translation(position + Vec3::new(0.0, 4.0, 0.0)),
        ..Default::default()
    });

    // Nacelle (Cube)
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Cuboid { half_size: Vec3::new(0.5, 0.5, 1.0) })),
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

        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(Cylinder {
                radius: blade_thickness,
                half_height: blade_length / 2.0,
                ..Default::default()
            })),
            material: debug_material.clone(),
            transform: blade_transform,
            ..Default::default()
        })
        .insert(Blade)  // Insert Blade component
        .insert(RotationSpeed(rotation_speed)); // Assign rotation speed to the blade
    }
}

#[derive(Debug, Component)]
struct Blade;

#[derive(Component)]
struct RotationSpeed(f32);

fn rotate_blades(time: Res<Time>, mut query: Query<(&RotationSpeed, &mut Transform), With<Blade>>) {
    for (rotation_speed, mut transform) in query.iter_mut() {
        let delta_rotation = Quat::from_rotation_z(time.delta_seconds() * rotation_speed.0);

        // Calculate the pivot point (the end of the blade)
        let pivot = transform.translation - transform.rotation * Vec3::new(0.0, 2.0, 0.0);
        
        // Rotate around the pivot point
        transform.rotate_around(pivot, delta_rotation);
    }
}


/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
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
