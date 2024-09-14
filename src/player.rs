use avian3d::prelude::*;
use bevy::color::palettes::tailwind;
use bevy::input::mouse::*;
use bevy::pbr::NotShadowCaster;
use bevy::render::view::RenderLayers;
use bevy::{color::palettes::css, prelude::*};
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;

#[derive(Debug, Component)]
pub struct WorldModelCamera;

/// Player movement speed factor.
const PLAYER_SPEED: f32 = 10.;

/// Used implicitly by all entities without a `RenderLayers` component.
/// Our world model camera and all objects other than the player are on this layer.
/// The light source belongs to both layers.
pub const DEFAULT_RENDER_LAYER: usize = 0;

/// Used by the view model camera and the player's arm.
/// The light source belongs to both layers.
pub const VIEW_MODEL_RENDER_LAYER: usize = 1;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // We need both Tnua's main controller plugin,
            // and the plugin to connect to the physics backend
            TnuaControllerPlugin::default(),
            TnuaAvian3dPlugin::default(),
        ))
        .add_systems(Startup, (player_setup,))
        .add_systems(
            Update,
            (player_look, player_move, player_fov, player_grow_shrink),
        );
    }
}

#[derive(Debug, Component)]
pub struct Player;

fn player_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let arm = meshes.add(Cuboid::new(0.1, 0.1, 0.5));
    let arm_material = materials.add(Color::from(tailwind::TEAL_200));

    let body = meshes.add(Capsule3d::new(10.0, 40.0));
    let body_material = materials.add(Color::from(tailwind::FUCHSIA_900));

    commands
        .spawn((
            Player,
            PbrBundle {
                mesh: meshes.add(Capsule3d {
                    radius: 0.5,
                    half_length: 0.5,
                }),
                material: materials.add(Color::from(css::DARK_CYAN)),
                transform: Transform::from_xyz(0.0, 1000.0, 0.0),
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

            // Spawn the player's body.
            parent.spawn((
                MaterialMeshBundle {
                    mesh: body,
                    material: body_material,
                    transform: Transform::from_xyz(0.0, 2.0, 0.0),
                    ..default()
                },
                RenderLayers::layer(DEFAULT_RENDER_LAYER),
                NotShadowCaster,
            ));
        });
}

fn player_move(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut controller: Query<(&mut TnuaController, &Transform), With<Player>>,
) {
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
        if wheel.y > 0.0 {
            perspective.fov -= 1.0_f32.to_radians();
            perspective.fov = perspective.fov.max(20.0_f32.to_radians());
        } else if wheel.y < 0.0 {
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

pub fn add_player_actions(app: &mut App) -> &mut App {
    app.add_systems(
        Update,
        (player_look, player_fov, player_move, player_grow_shrink),
    )
}
