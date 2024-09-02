use bevy::color::palettes::tailwind;
use bevy::input::mouse::*;
use bevy::pbr::NotShadowCaster;
use bevy::{color::palettes::css, prelude::*};
use bevy::render::view::RenderLayers;
use bevy::render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
    render_resource::PrimitiveTopology,
};


use noise::{NoiseFn, Perlin};
use avian3d::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;


/// Player movement speed factor.
const PLAYER_SPEED: f32 = 10.;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins, 
            PhysicsPlugins::default(),
            // Enables debug rendering
            PhysicsDebugPlugin::default(),
            // We need both Tnua's main controller plugin, and the plugin to connect to the physics
            // backend (in this case XBPD-3D)
            TnuaControllerPlugin::default(),
            TnuaAvian3dPlugin::default(),
        ))
        // Overwrite default debug rendering configuration (optional)
        .insert_gizmo_config(
            PhysicsGizmos {
                aabb_color: Some(Color::WHITE),
                ..default()
            },
            GizmoConfig::default(),
        )
        .add_systems(
            Startup,
            (
                spawn_view_model,
                spawn_world_model,
                spawn_lights,
                spawn_text,
                setup_wind_turbines,
                setup_terrain,
            ),
        )
        .add_systems(
            Update, (
                player_look,
                player_move, 
                player_fov,
                player_grow_shrink,
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
            PbrBundle {
                mesh: meshes.add(Capsule3d {
                    radius: 0.5,
                    half_length: 0.5,
                }),
                material: materials.add(Color::from(css::DARK_CYAN)),
                transform: Transform::from_xyz(0.0, 2.0, 0.0),
                ..Default::default()
            },
            // The player character needs to be configured as a dynamic rigid body of the physics
            // engine.
            RigidBody::Dynamic,
            Collider::capsule(0.5, 1.0),
            // This bundle holds the main components.
            TnuaControllerBundle::default(),
            // A sensor shape is not strictly necessary, but without it we'll get weird results.
            TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),
            // Tnua can fix the rotation, but the character will still get rotated before it can do so.
            // By locking the rotation we can prevent this.
            LockedAxes::ROTATION_LOCKED,
            RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
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


fn player_move(keyboard: Res<ButtonInput<KeyCode>>, mut controller: Query<(&mut TnuaController, &Transform), With<Player>>) {
    
    let Ok((mut controller, transform)) = controller.get_single_mut() else {
        return;
    };
    
    let mut direction = Vec3::ZERO;

    if keyboard.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        direction -= Vec3::Z;
    }
    if keyboard.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        direction += Vec3::Z;
    }
    if keyboard.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        direction -= Vec3::X;
    }
    if keyboard.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        direction += Vec3::X;
    }

    // NOTE: Restricting the player movement to XZ plane might be incorrect
    // after there are slopes the player must navigate up and down.

    // Get the player's forward direction vector (in the XZ plane)
    let forward = transform.rotation * Vec3::Z;
    let right = transform.rotation * Vec3::X;

    // Ignore the Y component (only consider X and Z)
    let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let right_xz = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

    // Calculate the movement in the XZ plane
    let direction = forward_xz * direction.z + right_xz * direction.x;

    // Feed the basis every frame. Even if the player doesn't move - just use `desired_velocity:
    // Vec3::ZERO`. `TnuaController` starts without a basis, which will make the character collider
    // just fall.
    controller.basis(TnuaBuiltinWalk {
        // The `desired_velocity` determines how the character will move.
        desired_velocity: direction.normalize_or_zero() * 10.0,
        // The `float_height` must be greater (even if by little) from the distance between the
        // character's center and the lowest point of its collider.
        float_height: 1.5,
        // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they have
        // sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn what they do.
        ..Default::default()
    });

    // Feed the jump action every frame as long as the player holds the jump button. If the player
    // stops holding the jump button, simply stop feeding the action.
    if keyboard.any_pressed([KeyCode::Backspace, KeyCode::Backspace]) {
        controller.action(TnuaBuiltinJump {
            // The height is the only mandatory field of the jump button.
            height: 4.0,
            // `TnuaBuiltinJump` also has customization fields with sensible defaults.
            ..Default::default()
        });
    }

}


fn spawn_world_model(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // let floor = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10.0)));
    let cube = meshes.add(Cuboid::new(2.0, 0.5, 1.0));
    let material = materials.add(Color::WHITE);

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
                color: Color::from(tailwind::YELLOW_300),
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(-2.0, 14.0, -0.75),
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
                    "Use the scroll-wheel to change the FOV\n",
                    "Use WASD to move. Use +/- to get taller/shorter."
                ),
                TextStyle {
                    font_size: 25.0,
                    ..default()
                },
            ));
        });
}


fn player_look(
    mut player: Query<&mut Transform, With<Player>>,
    mut mouse_motion: EventReader<MouseMotion>,
) {
    let Ok(mut player) = player.get_single_mut() else {
        return;
    };

    for motion in mouse_motion.read() {
        let yaw = -motion.delta.x * 0.003;
        let pitch = -motion.delta.y * 0.002;
        // Order of rotations is important, see <https://gamedev.stackexchange.com/a/136175/103059>
        player.rotate_y(yaw);
        player.rotate_local_x(pitch);
    }
}

fn player_fov(
    mut mouse_wheel: EventReader<MouseWheel>,
    mut world_model_projection: Query<&mut Projection, With<WorldModelCamera>>,
) {
    let mut projection = world_model_projection.single_mut();
    let Projection::Perspective(ref mut perspective) = projection.as_mut() else {
        unreachable!(
            "The `Projection` component was explicitly built with `Projection::Perspective`"
        );
    };

    for wheel in mouse_wheel.read() {
        if wheel.y  > 0.0 {
            perspective.fov -= 1.0_f32.to_radians();
            perspective.fov = perspective.fov.max(20.0_f32.to_radians());
        }
        else if wheel.y < 0.0 {
            perspective.fov += 1.0_f32.to_radians();
            perspective.fov = perspective.fov.min(160.0_f32.to_radians());
        }
    }
}

fn player_grow_shrink(
    mut transform: Query<&mut Transform, With<WorldModelCamera>>,
    time: Res<Time>,
    kb_input: Res<ButtonInput<KeyCode>>,
    
) {
    let Ok(mut transform) = transform.get_single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;

    if kb_input.pressed(KeyCode::Minus) {
        direction.y -= 1.;
    }

    if kb_input.pressed(KeyCode::Equal) {
        direction.y += 1.;
    }

    // Progressively update the player's position over time. Normalize the
    // direction vector to prevent it from exceeding a magnitude of 1 when
    // moving diagonally.
    let move_delta = direction.normalize_or_zero() * PLAYER_SPEED * time.delta_seconds();
    
    // Apply the movement to the player's translation
    transform.translation += move_delta;
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

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
    .with_duplicated_vertices()
    .with_computed_flat_normals()

}

fn setup_terrain(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Generate procedural terrain mesh
    let terrain_mesh = generate_procedural_terrain_mesh(100, 0.2);

    // Spawn terrain entity
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(terrain_mesh),
            material: materials.add(Color::from(tailwind::LIME_500)),
            transform: Transform::from_xyz(-50.0, 0.0, -50.0),
            ..Default::default()
        },
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
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
