//! A spherical world game with true Goldberg polyhedron tiling

use bevy::prelude::*;
use bevy::pbr::wireframe::WireframePlugin;
use miniature_potato::camera::{camera_controller, calculate_camera_transform, OrbitCamera};
use miniature_potato::ray_casting::{
    HoveredTriangle, simple_tile_visualization, handle_tile_selection, print_hovered_tile_info
};
use miniature_potato::world::{setup_world, toggle_wireframe, toggle_borders, print_tile_info};

fn main() {
    println!("🚀 Starting True Goldberg Polyhedron World with Pentagon-Centric Construction");
    
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
            toggle_borders,
            print_tile_info,
            print_hovered_tile_info,
            camera_controller,
            simple_tile_visualization,
            handle_tile_selection,
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
    println!("     • Left click: Select/deselect tile");
    println!("     • Space: Toggle wireframe");
    println!("     • B: Toggle pentagon/hexagon borders");
    println!("     • I: Print tile system info");
    println!("     • H: Print hovered tile info");
    println!("     • Esc: Quit");
}
