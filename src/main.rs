use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::window::WindowRef;
use bevy::window::WindowResolution;
use bevy::color::palettes::tailwind;

pub mod glft_info;
pub mod player;
pub mod potato;
pub mod turbine;

use glft_info::GltfInfoPlugin;
use player::*;
use potato::PotatoPlugin;
use turbine::*;

fn main() {
    App::new()
        .insert_resource(DropCooldown::default())
        .add_plugins((
            DefaultPlugins,
            GltfInfoPlugin,
            PhysicsPlugins::default(),
            // Enables debug rendering
            PhysicsDebugPlugin::default(),
            PlayerPlugin,
            PotatoPlugin,
        ))
        // Overwrite default debug rendering configuration (optional)
        .insert_gizmo_config(
            PhysicsGizmos {
                //aabb_color: Some(Color::WHITE),
                //collider_color: Some(Color::srgb_from_array(tailwind::AMBER_900.to_f32_array_no_alpha()) ),
                contact_point_color: Some(Color::srgb_from_array(tailwind::CYAN_500.to_f32_array_no_alpha())),
                ..default()
            },
            GizmoConfig::default(),
        )
        .add_systems(
            Startup,
            (spawn_lights, spawn_text, spawn_world_window), //, setup_wind_turbines, setup_terrain),
        )
        .add_systems(
            Update,
            (quit_on_esc_system, rotate_blades, drop_wind_turbine, update_gizmo_config),
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

fn spawn_lights(mut commands: Commands) {
    // Spawn a light that mimics the sun
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.9), // Slightly warm light
            illuminance: 100_000.0,             // Bright intensity, adjust based on scene
            shadows_enabled: true,              // Enable shadows for more realistic sunlight
            ..default()
        },
        transform: Transform {
            rotation: Quat::from_euler(
                EulerRot::XYZ,
                -std::f32::consts::FRAC_PI_4,
                std::f32::consts::FRAC_PI_4,
                0.0,
            ),
            ..default()
        },
        ..default()
    });
}

fn spawn_world_window(mut commands: Commands) {
    let new_window = Window {
        resolution: WindowResolution::new(800.0, 600.0),
        title: "World Window".to_string(),
        ..default() // Use default for other parameters
    };

    let window_entity = commands.spawn((new_window,)).id();

    // Add the camera at a fixed point in space
    commands.spawn((Camera3dBundle {
        camera: Camera {
            // Bump the order to render on top of the world model.
            target: bevy::render::camera::RenderTarget::Window(WindowRef::Entity(window_entity)),
            //order: 1,
            ..default()
        },
        projection: PerspectiveProjection {
            fov: 30.0_f32.to_radians(),
            ..default()
        }
        .into(),
        transform: Transform::from_xyz(600.0, 2000.0, 600.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },));
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

fn update_gizmo_config(
    mut config_store: ResMut<GizmoConfigStore>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if keyboard.just_pressed(KeyCode::KeyY) {
        for (_, config, _) in config_store.iter_mut() {
            config.depth_bias = if config.depth_bias == 0. { -1. } else { 0. };
        }
    }
    if keyboard.just_pressed(KeyCode::KeyU) {
        for (_, config, _) in config_store.iter_mut() {
            // Toggle line_perspective
            config.line_perspective ^= true;
            // Increase the line width when line_perspective is on
            config.line_width *= if config.line_perspective { 5. } else { 1. / 5. };
        }
    }

    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    if keyboard.pressed(KeyCode::BracketRight) {
        config.line_width += 5. * time.delta_seconds();
        config.line_width = config.line_width.clamp(0., 50.);
    }
    if keyboard.pressed(KeyCode::BracketLeft) {
        config.line_width -= 5. * time.delta_seconds();
        config.line_width = config.line_width.clamp(0., 50.);
    }
    if keyboard.just_pressed(KeyCode::Backslash) {
        config.enabled ^= true;
    }
    if keyboard.just_pressed(KeyCode::KeyP) {
        config.line_style = match config.line_style {
            GizmoLineStyle::Solid => GizmoLineStyle::Dotted,
            _ => GizmoLineStyle::Solid,
        };
    }
    if keyboard.just_pressed(KeyCode::KeyO) {
        config.line_joints = match config.line_joints {
            GizmoLineJoint::Bevel => GizmoLineJoint::Miter,
            GizmoLineJoint::Miter => GizmoLineJoint::Round(4),
            GizmoLineJoint::Round(_) => GizmoLineJoint::None,
            GizmoLineJoint::None => GizmoLineJoint::Bevel,
        };
    }

    if keyboard.just_pressed(KeyCode::KeyI) {
        // AABB gizmos are normally only drawn on entities with a ShowAabbGizmo component
        // We can change this behaviour in the configuration of AabbGizmoGroup
        config_store.config_mut::<AabbGizmoConfigGroup>().1.draw_all ^= true;
    }
}