use avian3d::prelude::*;
use bevy::color::palettes::css::*;
use bevy::color::palettes::tailwind;
use bevy::input::mouse::*;
use bevy::prelude::*;
use bevy_tnua::builtins::TnuaBuiltinDash;
use bevy_tnua::math::{float_consts, AdjustPrecision, AsF32, Float, Quaternion, Vector3};
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;

#[derive(Debug, Component)]
pub struct Player;

#[derive(Debug, Component)]
pub struct PlayerHead {
    pub forward: Vector3,
    pub pitch_angle: Float,
}

impl Default for PlayerHead {
    fn default() -> Self {
        Self {
            forward: Vector3::NEG_Z,
            pitch_angle: 0.0,
        }
    }
}

#[derive(Debug, Component)]
pub struct PlayerEyes;

/// Player movement speed factor.
const PLAYER_SPEED: f32 = 10.;

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
            (player_look, player_move, player_fov, head_height_adjust, update_gravity_system),
        );
    }
}

fn player_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let body = meshes.add(Capsule3d::new(10.0, 40.0));
    let body_material = materials.add(Color::from(tailwind::FUCHSIA_900));

    let arm = meshes.add(Capsule3d::new(2.5, 30.0));
    let arm_material = materials.add(Color::from(tailwind::GREEN_800));

    let head = meshes.add(Sphere::new(8.0));
    let head_material = materials.add(Color::from(tailwind::FUCHSIA_900));

    let glasses = meshes.add(Cuboid::new(14.0, 5.0, 14.0));
    let glasses_material = materials.add(Color::from(tailwind::BLUE_900));

    commands
        .spawn((
            Player,
            // The player character needs to be configured as a dynamic rigid body of the physics
            // engine.
            RigidBody::Dynamic,
            Collider::capsule(10.0, 40.0),
            // This bundle holds the main components.
            TnuaControllerBundle::default(),
            // A sensor shape is not strictly necessary, but without it we'll get weird results.
            TnuaAvian3dSensorShape(Collider::capsule(10.0, 40.0)),
            // TODO: remove this to allow the body to face forward,
            // but then need to fix head over-rotation
            // LockedAxes::ROTATION_LOCKED,
            SpatialBundle::from_transform(Transform::from_xyz(500.0, 1000.0, 500.0)),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    PlayerHead::default(),
                    MaterialMeshBundle {
                        mesh: head,
                        material: head_material,
                        transform: Transform::from_xyz(0.0, 50.0, 0.0),
                        ..default()
                    },
                ))
                .with_children(|sub_parent| {
                    sub_parent.spawn((MaterialMeshBundle {
                        mesh: glasses,
                        material: glasses_material,
                        transform: Transform::from_xyz(0.0, 3.0, -4.0),
                        ..default()
                    },));
                    sub_parent.spawn((
                        PlayerEyes,
                        Camera3dBundle {
                            projection: PerspectiveProjection {
                                fov: 90.0_f32.to_radians(),
                                ..default()
                            }
                            .into(),
                            transform: Transform::from_xyz(0.0, 3.0, -4.0),
                            ..default()
                        },
                    ));
                });

            parent.spawn((
                MaterialMeshBundle {
                    mesh: body,
                    material: body_material,
                    ..default()
                },
            ));

            parent.spawn((
                MaterialMeshBundle {
                    mesh: arm,
                    material: arm_material,
                    transform: Transform::from_xyz(0.0, 15.0, 0.0)
                        .with_rotation(Quat::from_rotation_z(90.0_f32.to_radians())),
                    ..default()
                },
            ));
        });
}

fn player_move(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query_player: Query<&mut TnuaController, With<Player>>,
    query_head: Query<&PlayerHead>,
    query_player_transform: Query<&Transform, With<Player>>,
    mut gizmos: Gizmos,
) {
    let Ok(mut controller) = query_player.get_single_mut() else {
        return;
    };

    let Ok(player_head) = query_head.get_single() else {
        return;
    };

    let Ok(player_transform) = query_player_transform.get_single() else {
        return;
    };

    // Initialize direction
    let mut direction = Vec3::ZERO;

    // Define player's local forward (Z-axis) and right (X-axis)
    let forward = player_transform.rotation * Vec3::Z;
    let right = player_transform.rotation * Vec3::X;

    // Adjust direction based on input relative to player's local space
    if keyboard.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        direction -= forward;
    }
    if keyboard.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        direction += forward;
    }
    if keyboard.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        direction -= right;
    }
    if keyboard.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        direction += right;
    }
    direction = direction.clamp_length_max(1.0);

    gizmos.arrow(Vec3::ZERO, Vec3::ONE * 1000.0, YELLOW);

    // Calculate the world-space position of the direction
    let world_start = player_transform.translation; // Player's position in world space
    let world_end = player_transform.translation + direction * 50.0; // Scale direction and apply it to the world start position

    // Draw the gizmo arrow
    gizmos.arrow(world_start, world_end, GREEN);

    // Draw gizmo 1: From the player's center straight down along the player's local Y-axis
    let player_position = player_transform.translation;
    let down_direction = player_transform.rotation * Vec3::NEG_Y * 100.0; // 100 units down
    let down_end = player_position + down_direction;
    gizmos.arrow(player_position, down_end, RED);

    let player_position = player_transform.translation;
    let down_direction = player_transform.rotation * Vec3::NEG_Y * 100.0; // 100 units down
    let down_end = player_position + down_direction;
    gizmos.arrow(player_position, down_end, RED);

    // Draw gizmo 2: From the player's center to the world origin (Vec3::ZERO)
    let world_origin = Vec3::ZERO;
    gizmos.line(player_position, world_origin, BLUE);


    let dash = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    // Feed the basis every frame. Even if the player doesn't move - just use `desired_velocity:
    // Vec3::ZERO`. `TnuaController` starts without a basis, which will make the character collider
    // just fall.
    controller.basis(TnuaBuiltinWalk {
        // The `desired_velocity` determines how the character will move.
        desired_velocity: direction.normalize_or_zero() * 10.0,
        //desired_forward: player_head.forward,
        // The `float_height` must be greater (even if by little) from the distance between the
        // character's center and the lowest point of its collider.
        float_height: 30.0,
        max_slope: float_consts::PI * 2.0,
        // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they have
        // sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn what they do.
        ..Default::default()
    });

    // Feed the jump action every frame as long as the player holds the jump button. If the player
    // stops holding the jump button, simply stop feeding the action.
    if keyboard.any_pressed([KeyCode::Space, KeyCode::Backspace]) {
        controller.action(TnuaBuiltinJump {
            // The height is the only mandatory field of the jump button.
            height: 4.0,
            // `TnuaBuiltinJump` also has customization fields with sensible defaults.
            ..Default::default()
        });
    }

    if dash {
        controller.action(TnuaBuiltinDash {
            // Dashing is also an action, but because it has directions we need to provide said
            // directions. `displacement` is a vector that determines where the jump will bring
            // us. Note that even after reaching the displacement, the character may still have
            // some leftover velocity (configurable with the other parameters of the action)
            //
            // The displacement is "frozen" when the action starts - user code does not have to
            // worry about storing the original direction.
            displacement: direction.normalize() * 10.0,
            // When set, the `desired_forward` of the dash action "overrides" the
            // `desired_forward` of the walk basis. Like the displacement, it gets "frozen" -
            // allowing to easily maintain a forward direction during the dash.
            // desired_forward: direction.normalize(),
            allow_in_air: true,
            ..Default::default()
        });        
    }
}

fn update_gravity_system(mut gravity: ResMut<Gravity>, query: Query<&Transform, With<Player>>) {
    let Ok(player_transform) = query.get_single() else {
        return;
    };

    // Calculate the direction from the player's position to the center of the planet (Vec3::ZERO)
    let direction_to_center = Vec3::ZERO - player_transform.translation;

    // Normalize the direction vector so it's unit length
    let gravity_direction = direction_to_center.normalize();

    // Set gravity to point towards the center with a magnitude of -9.8 (standard Earth gravity)
    gravity.0 = gravity_direction * 9.8;

    // // Adjust player's rotation to make them upright relative to gravity
    // // Create a rotation from the player's current "up" direction (Y axis) to the gravity direction (inverted)
    // let current_up = player_transform.rotation * Vec3::Y;
    // let rotation_to_upright = Quat::from_rotation_arc(current_up, -gravity_direction);
    // println!("current_up: {:?}", current_up);
    // println!("rotation_to_upright: {:?}", rotation_to_upright);
    //
    // // Apply the rotation to the player's transform
    // player_transform.rotation = rotation_to_upright * player_transform.rotation;

    // println!("Gravity: {:?}", gravity);
}

fn player_look(
    mut query: Query<(&mut Transform, &mut PlayerHead)>,
    mut mouse_motion: EventReader<MouseMotion>,
) {
    let Ok((mut transform, mut player_head)) = query.get_single_mut() else {
        return;
    };

    let total_delta: Vec2 = mouse_motion.read().map(|event| event.delta).sum();

    let yaw = Quaternion::from_rotation_y(-0.007 * total_delta.x.adjust_precision());
    player_head.forward = yaw.mul_vec3(player_head.forward);

    let pitch = 0.005 * total_delta.y.adjust_precision();
    player_head.pitch_angle =
        (player_head.pitch_angle + pitch).clamp(-float_consts::FRAC_PI_2, float_consts::FRAC_PI_2);

    // Normalize the forward vector
    let forward_normalized = player_head.forward.normalize();

    // Define the default forward direction (the direction the object is currently facing)
    let default_forward = Vec3::NEG_Z; // assuming -Z is the default forward direction

    // Create a quaternion to rotate the default forward vector to the target forward vector
    let forward_rotation = Quat::from_rotation_arc(default_forward, forward_normalized);

    // Create a quaternion for rotation around the local X-axis by the specified angle
    let x_rotation = Quat::from_rotation_x(player_head.pitch_angle);

    // Combine the two rotations by multiplying the quaternions
    transform.rotation = forward_rotation * x_rotation;
}

fn player_fov(
    mut mouse_wheel: EventReader<MouseWheel>,
    mut world_model_projection: Query<&mut Projection, With<PlayerEyes>>,
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

fn head_height_adjust(
    mut transform: Query<&mut Transform, With<PlayerHead>>,
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
    if kb_input.pressed(KeyCode::Digit9) {
        direction.x -= 1.;
    }
    if kb_input.pressed(KeyCode::Digit0) {
        direction.x += 1.;
    }
    if kb_input.pressed(KeyCode::Digit7) {
        direction.z -= 1.;
    }
    if kb_input.pressed(KeyCode::Digit8) {
        direction.z += 1.;
    }

    // Progressively update the player's position over time. Normalize the
    // direction vector to prevent it from exceeding a magnitude of 1 when
    // moving diagonally.
    transform.translation += direction.normalize_or_zero() * PLAYER_SPEED * time.delta_seconds();
}
