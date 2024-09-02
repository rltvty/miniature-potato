use avian3d::prelude::*;
use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;

pub mod player;
pub mod terrain;
pub mod turbine;

use player::*;
use terrain::*;
use turbine::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            // Enables debug rendering
            PhysicsDebugPlugin::default(),
            PlayerPlugin,
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
            (spawn_lights, spawn_text, setup_wind_turbines, setup_terrain),
        )
        .add_systems(Update, (quit_on_esc_system, rotate_blades))
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
