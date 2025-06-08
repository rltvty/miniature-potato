//! A spherical world game with hexagonal tiling using icosphere generation

use bevy::prelude::*;
use miniature_potato::camera::{camera_controller, calculate_camera_transform, OrbitCamera};
use miniature_potato::ray_casting::{
    ray_casting_system, debug_hovered_triangle, visualize_hit_point, HoveredTriangle
};
use miniature_potato::tiles::{tile_selection_system, visualize_tiles};
use miniature_potato::world::setup_world;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<HoveredTriangle>()
        .add_systems(Startup, (setup_world, setup_camera))
        .add_systems(Update, (
            camera_controller,
            ray_casting_system,
            debug_hovered_triangle,
            visualize_hit_point,
            tile_selection_system,
            visualize_tiles,
        ))
        .run();
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
