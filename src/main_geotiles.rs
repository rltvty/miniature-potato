//! Main entry point using geotiles crate

use bevy::prelude::*;
use bevy::pbr::wireframe::{WireframePlugin, WireframeConfig};
use miniature_potato::camera::{camera_controller, calculate_camera_transform, OrbitCamera};
use miniature_potato::geotiles_bevy::{setup_hexasphere_world, tile_hover_system};

fn main() {
    println!("🚀 Starting miniature-potato with geotiles integration");
    
    App::new()
        .add_plugins((
            DefaultPlugins,
            WireframePlugin::default(),
        ))
        .add_systems(Startup, (setup_hexasphere_world, setup_camera))
        .add_systems(Update, (
            handle_escape_key,
            toggle_wireframe,
            camera_controller,
            tile_hover_system,
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

/// System to toggle wireframe rendering
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
    
    println!("📷 Camera initialized");
    println!("   Controls:");
    println!("     • Mouse drag: Rotate camera");
    println!("     • Mouse wheel: Zoom in/out");
    println!("     • WASD: Pan camera");
    println!("     • Space: Toggle wireframe");
    println!("     • Esc: Quit");
}