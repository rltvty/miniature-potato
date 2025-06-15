//! A spherical world game using geotiles for geodesic polyhedron tiling

use bevy::prelude::*;
use bevy::pbr::wireframe::{WireframePlugin, WireframeConfig};
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::WindowPlugin;
use std::env;
use miniature_potato::camera::{camera_controller, rotate_sphere_system, OrbitCamera, SphereRotation};
use miniature_potato::geotiles_bevy::{
    setup_hexasphere_world, tile_hover_system, tile_gizmos_system, toggle_borders, toggle_normals,
    HexasphereResource
};

/// Resource to track screenshot timing
#[derive(Resource)]
struct ScreenshotTimer {
    timer: Timer,
    should_screenshot: bool,
    exit_timer: Option<Timer>,
}

fn main() {
    println!("🚀 Starting miniature-potato with geotiles geodesic polyhedron");
    
    // Check for screenshot flag
    let args: Vec<String> = env::args().collect();
    let screenshot_mode = args.contains(&"--screenshot".to_string());
    
    if screenshot_mode {
        println!("📸 Screenshot mode enabled - will take screenshot after 2 seconds and exit");
    }
    
    let mut app = App::new();
    
    // Configure plugins with smaller window for screenshot mode
    if screenshot_mode {
        app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "miniature-potato (screenshot)".to_string(),
                    resolution: (800.0, 600.0).into(),
                    ..default()
                }),
                ..default()
            }),
            WireframePlugin::default(),
        ));
    } else {
        app.add_plugins((
            DefaultPlugins,
            WireframePlugin::default(),
        ));
    }
    
    app
        .insert_resource(SphereRotation::new())
        .add_systems(Startup, (setup_hexasphere_world, setup_camera, setup_ui));
    
    if screenshot_mode {
        app.insert_resource(ScreenshotTimer {
            timer: Timer::from_seconds(2.0, TimerMode::Once),
            should_screenshot: true,
            exit_timer: None,
        })
        .add_systems(Update, (
            handle_escape_key,
            toggle_wireframe,
            toggle_borders,
            toggle_normals,
            print_tile_info,
            handle_tile_selection,
            camera_controller,
            rotate_sphere_system,
            tile_hover_system,
            tile_gizmos_system,
            update_hovered_tile_ui,
            update_selected_tile_ui,
            update_sphere_info_ui,
            screenshot_system,
        ));
    } else {
        app.insert_resource(WireframeConfig { 
            global: false, // Disable wireframe by default
            default_color: Color::WHITE,
        })
        .add_systems(Update, (
            handle_escape_key,
            toggle_wireframe,
            toggle_borders,
            toggle_normals,
            print_tile_info,
            handle_tile_selection,
            camera_controller,
            rotate_sphere_system,
            tile_hover_system,
            tile_gizmos_system,
            update_hovered_tile_ui,
            update_selected_tile_ui,
            update_sphere_info_ui,
        ));
    }
    
    app.run();
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

/// System to print tile information when 'I' is pressed
fn print_tile_info(
    keyboard: Res<ButtonInput<KeyCode>>,
    hexasphere_res: Option<Res<HexasphereResource>>,
    camera_query: Query<&Transform, With<OrbitCamera>>,
) {
    if keyboard.just_pressed(KeyCode::KeyI) {
        if let Some(hexasphere) = hexasphere_res {
            println!("\n📊 === HEXASPHERE INFO ===");
            println!("Total tiles: {}", hexasphere.hexasphere.tiles.len());
            
            let pentagon_count = hexasphere.hexasphere.tiles.iter()
                .filter(|t| t.boundary.len() == 5)
                .count();
            let hexagon_count = hexasphere.hexasphere.tiles.iter()
                .filter(|t| t.boundary.len() == 6)
                .count();
            
            println!("Pentagons: {}", pentagon_count);
            println!("Hexagons: {}", hexagon_count);
            println!("Sphere radius: {}", hexasphere.hexasphere.radius);
            println!("Uniform hexagon radius: {:.3}", hexasphere.uniform_radius);
            
            // Check camera position relative to sphere
            if let Ok(camera_transform) = camera_query.single() {
                let distance_from_center = camera_transform.translation.length();
                println!("\n📷 Camera Info:");
                println!("Position: ({:.2}, {:.2}, {:.2})", 
                    camera_transform.translation.x,
                    camera_transform.translation.y, 
                    camera_transform.translation.z);
                println!("Distance from center: {:.2}", distance_from_center);
                println!("Inside sphere: {}", distance_from_center < hexasphere.hexasphere.radius as f32);
            }
            
            // Calculate statistics
            let stats = hexasphere.hexasphere.calculate_hexagon_stats();
            println!("\n📐 Hexagon Statistics:");
            println!("Average radius: {:.3}", stats.average_hexagon_radius);
            println!("Min radius: {:.3}", stats.min_hexagon_radius);
            println!("Max radius: {:.3}", stats.max_hexagon_radius);
            println!("Size variation: {:.1}%", 
                100.0 * (stats.max_hexagon_radius - stats.min_hexagon_radius) / stats.average_hexagon_radius);
            println!("Standard deviation: {:.3}", stats.radius_std_deviation);
        } else {
            println!("Hexasphere not yet initialized");
        }
    }
}

/// System to update the hovered tile UI text
fn update_hovered_tile_ui(
    hexasphere_res: Option<Res<HexasphereResource>>,
    mut hovered_text_query: Query<&mut Text, With<HoveredTileText>>,
) {
    if let Ok(mut text) = hovered_text_query.single_mut() {
        if let Some(hexasphere) = hexasphere_res {
            if let Some(hovered_index) = hexasphere.hovered_tile {
                if let Some(tile) = hexasphere.hexasphere.tiles.get(hovered_index) {
                    let tile_type = if tile.boundary.len() == 5 { "Pentagon" } else { "Hexagon" };
                    let center = &tile.center_point;
                    let lat_lon = tile.get_lat_lon(hexasphere.hexasphere.radius);
                    
                    **text = format!(
                        "Hovered: {} #{}\nType: {}\nCenter: ({:.2}, {:.2}, {:.2})\nLat/Lon: {:.1}°, {:.1}°\nNeighbors: {}",
                        tile_type,
                        hovered_index,
                        tile_type,
                        center.x, center.y, center.z,
                        lat_lon.lat, lat_lon.lon,
                        tile.neighbors.len()
                    );
                } else {
                    **text = "Hovered: Invalid tile".to_string();
                }
            } else {
                **text = "Hovered: None".to_string();
            }
        } else {
            **text = "Hovered: Loading...".to_string();
        }
    }
}

/// System to update the selected tile UI text
fn update_selected_tile_ui(
    hexasphere_res: Option<Res<HexasphereResource>>,
    mut selected_text_query: Query<&mut Text, With<SelectedTileText>>,
) {
    if let Ok(mut text) = selected_text_query.single_mut() {
        if let Some(hexasphere) = hexasphere_res {
            if let Some(selected_index) = hexasphere.selected_tile {
                if let Some(tile) = hexasphere.hexasphere.tiles.get(selected_index) {
                    let tile_type = if tile.boundary.len() == 5 { "Pentagon" } else { "Hexagon" };
                    **text = format!("Selected: {} #{}", tile_type, selected_index);
                } else {
                    **text = "Selected: Invalid tile".to_string();
                }
            } else {
                **text = "Selected: None".to_string();
            }
        } else {
            **text = "Selected: Loading...".to_string();
        }
    }
}

/// System to update the sphere info UI text
fn update_sphere_info_ui(
    hexasphere_res: Option<Res<HexasphereResource>>,
    mut sphere_info_query: Query<&mut Text, With<SphereInfoText>>,
) {
    if let Ok(mut text) = sphere_info_query.single_mut() {
        if let Some(hexasphere) = hexasphere_res {
            let pentagon_count = hexasphere.hexasphere.tiles.iter()
                .filter(|t| t.boundary.len() == 5)
                .count();
            let hexagon_count = hexasphere.hexasphere.tiles.iter()
                .filter(|t| t.boundary.len() == 6)
                .count();
            
            let stats = hexasphere.hexasphere.calculate_hexagon_stats();
            
            **text = format!(
                "Sphere Info:\nTotal tiles: {}\nPentagons: {}\nHexagons: {}\nRadius: {:.1}\nAvg hex radius: {:.3}\nSize variation: {:.1}%",
                hexasphere.hexasphere.tiles.len(),
                pentagon_count,
                hexagon_count,
                hexasphere.hexasphere.radius,
                stats.average_hexagon_radius,
                100.0 * (stats.max_hexagon_radius - stats.min_hexagon_radius) / stats.average_hexagon_radius
            );
        } else {
            **text = "Sphere Info:\nLoading...".to_string();
        }
    }
}

/// System to handle tile selection with mouse clicks
fn handle_tile_selection(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut hexasphere_res: ResMut<HexasphereResource>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(hovered_index) = hexasphere_res.hovered_tile {
            // Toggle selection
            if hexasphere_res.selected_tile == Some(hovered_index) {
                // Deselect
                hexasphere_res.selected_tile = None;
                println!("🎯 Deselected tile");
            } else {
                // Select
                hexasphere_res.selected_tile = Some(hovered_index);
                if let Some(tile) = hexasphere_res.hexasphere.tiles.get(hovered_index) {
                    println!("🎯 Selected {} tile {}", 
                        if tile.boundary.len() == 5 { "pentagon" } else { "hexagon" },
                        hovered_index
                    );
                }
            }
        }
    }
}

/// Setup the camera with controls
fn setup_camera(mut commands: Commands) {
    let orbit_camera = OrbitCamera::default();
    
    // Initialize camera looking at origin, but after this it moves freely
    let transform = Transform::from_translation(orbit_camera.position)
        .looking_at(Vec3::ZERO, Vec3::Y);
    
    commands.spawn((
        Camera3d::default(),
        transform,
        orbit_camera,
    ));
    
    println!("📷 Camera initialized");
}

/// Component to mark the hovered tile info text
#[derive(Component)]
struct HoveredTileText;

/// Component to mark the selected tile info text
#[derive(Component)]
struct SelectedTileText;

/// Component to mark the sphere info text
#[derive(Component)]
struct SphereInfoText;

/// Setup the UI with on-screen controls help and tile info displays
fn setup_ui(mut commands: Commands) {
    // Controls help in top-left
    commands.spawn((
        Text::new(
            "Controls:\n\
            • Right mouse drag: Rotate sphere\n\
            • Mouse wheel: Zoom camera forward/back\n\
            • WASD: Move camera up/down/left/right\n\
            • Left click: Select/deselect tile\n\
            • Space: Toggle wireframe mode\n\
            • N: Toggle normal visualization\n\
            • I: Print hexasphere info\n\
            • Esc: Quit"
        ),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
    
    // Sphere info in top-right
    commands.spawn((
        Text::new("Sphere Info:\nLoading..."),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.7, 0.9, 1.0)), // Light blue color for sphere info
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            right: Val::Px(12.0),
            ..default()
        },
        SphereInfoText,
    ));
    
    // Hovered tile info in bottom-left
    commands.spawn((
        Text::new("Hovered: None"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        HoveredTileText,
    ));
    
    // Selected tile info in bottom-right
    commands.spawn((
        Text::new("Selected: None"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.3)), // Yellow color for selected
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            right: Val::Px(12.0),
            ..default()
        },
        SelectedTileText,
    ));
    
    println!("   Run with --screenshot flag to auto-capture screenshot and exit");
}

/// System to handle automatic screenshot capture and exit
fn screenshot_system(
    time: Res<Time>,
    mut timer_res: ResMut<ScreenshotTimer>,
    mut commands: Commands,
    windows: Query<Entity, With<Window>>,
    mut exit: EventWriter<AppExit>,
) {
    // Take screenshot after initial delay
    if timer_res.should_screenshot {
        timer_res.timer.tick(time.delta());
        
        if timer_res.timer.just_finished() {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            // Use absolute path to repo root
            let repo_root = "/Users/rltvty/src/github.com/rltvty/miniature-potato";
            let filename = format!("{}/screenshot_{}.png", repo_root, timestamp);
            
            println!("📸 Taking screenshot: {}", filename);
            
            // Try to get the window entity first
            if let Ok(window_entity) = windows.single() {
                println!("Found window entity: {:?}", window_entity);
                
                // Take screenshot using the new Bevy 0.16.1 API with specific window
                use bevy::render::camera::RenderTarget;
                use bevy::window::WindowRef;
                commands
                    .spawn(Screenshot(RenderTarget::Window(WindowRef::Entity(window_entity))))
                    .observe(save_to_disk(filename.clone()));
            } else {
                println!("No window entity found, using primary_window()");
                
                // Fallback to primary window
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(filename.clone()));
            }
            
            timer_res.should_screenshot = false;
            // Start exit timer to allow screenshot to save
            timer_res.exit_timer = Some(Timer::from_seconds(1.0, TimerMode::Once));
            
            println!("✅ Screenshot command sent as {}, will exit in 1 second...", filename);
        }
    }
    
    // Handle exit timer
    if let Some(ref mut exit_timer) = timer_res.exit_timer {
        exit_timer.tick(time.delta());
        if exit_timer.just_finished() {
            println!("⏰ Exit timer finished, shutting down...");
            exit.write(AppExit::Success);
        }
    }
}