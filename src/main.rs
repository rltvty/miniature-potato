//! A spherical world game with hexagonal tiling using icosphere generation

use bevy::prelude::*;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use miniature_potato::camera::{camera_controller, calculate_camera_transform, OrbitCamera};
use miniature_potato::ray_casting::{
    ray_casting_system, debug_hovered_triangle, visualize_hit_point, HoveredTriangle
};
use miniature_potato::tiles::{tile_selection_system, visualize_tiles};
use miniature_potato::world::setup_world;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            WireframePlugin::default(),
        ))
        .init_resource::<HoveredTriangle>()
        .add_systems(Startup, (setup_world, setup_camera))
        .add_systems(Update, (
            handle_escape_key,
            toggle_wireframe,
            camera_controller,
            ray_casting_system,
            debug_hovered_triangle,
            visualize_hit_point,
            tile_selection_system,
            visualize_tiles,
        ))
        .run();
}

/// System to handle escape key press for quitting the app
fn handle_escape_key(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        println!("Escape key pressed - exiting application");
        exit.write(AppExit::Success);
    }
}

/// System to toggle wireframe mode with spacebar
fn toggle_wireframe(
    mut wireframe_config: ResMut<WireframeConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        wireframe_config.global = !wireframe_config.global;
        println!("Wireframe mode: {}", if wireframe_config.global { "ON" } else { "OFF" });
    }
}

/// Setup the camera with orbit controls
fn setup_camera(mut commands: Commands) {
    let orbit_camera = OrbitCamera::default();
    let transform = calculate_camera_transform(&orbit_camera);
    
    commands.spawn((
        Camera3d::default(),
        transform,
        orbit_camera,
    ));
}
