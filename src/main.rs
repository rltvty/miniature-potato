use avian3d::prelude::*;
use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::window::WindowResolution;
use bevy::window::WindowRef;


pub mod glft_info;
pub mod player;
pub mod potato;
pub mod turbine;

use glft_info::GltfInfoPlugin;
use player::*;
use turbine::*;
use potato::PotatoPlugin;



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
                aabb_color: Some(Color::WHITE),
                ..default()
            },
            GizmoConfig::default(),
        )
        .add_systems(
            Startup,
            (spawn_lights, spawn_text, spawn_world_window) //, setup_wind_turbines, setup_terrain),
        )
        .add_systems(Update, (quit_on_esc_system, rotate_blades, drop_wind_turbine))
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

    // Spawn a light that mimics the sun
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.9), // Slightly warm light
            illuminance: 100_000.0, // Bright intensity, adjust based on scene
            shadows_enabled: true, // Enable shadows for more realistic sunlight
            ..default()
        },
        transform: Transform {
            rotation: Quat::from_euler(EulerRot::XYZ, -std::f32::consts::FRAC_PI_4, std::f32::consts::FRAC_PI_4, 0.0),
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
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                // Bump the order to render on top of the world model.
                target: bevy::render::camera::RenderTarget::Window(WindowRef::Entity(window_entity)),
                //order: 1,
                ..default()
            },
            projection: PerspectiveProjection {
                fov: 70.0_f32.to_radians(),
                ..default()
            }
            .into(),
            transform: Transform::from_xyz(700.0, 1200.0, 700.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
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
