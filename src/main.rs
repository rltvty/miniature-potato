//! A spherical world game using geotiles for geodesic polyhedron tiling

use bevy::color::palettes::tailwind::*;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::picking::pointer::PointerInteraction;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::WindowPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use iyes_perf_ui::prelude::PerfUiAllEntries;
use iyes_perf_ui::PerfUiPlugin;
use miniature_potato::geotiles_bevy::{
    handle_tile_selection, setup_hexasphere_world, tile_gizmos_system, toggle_normals,
    HexasphereResource, WorldParent,
};
use miniature_potato::character::{setup_character, handle_character_movement, update_dead_zone_state, debug_gizmos_system, toggle_debug_gizmos, CharacterResource, DebugGizmosResource, Character, find_neighbor_in_direction, vec3_from_point};
use serde::Deserialize;
use std::env;
use std::fs;

/// Resource to track screenshot timing
#[derive(Resource)]
struct ScreenshotTimer {
    timer: Timer,
    should_screenshot: bool,
    exit_timer: Option<Timer>,
}

/// Event for automated testing commands
#[derive(Event, Clone)]
pub struct AutomatedTestEvent {
    pub command_type: TestCommandType,
    pub description: String,
}

#[derive(Clone)]
pub enum TestCommandType {
    ToggleGizmos,
    MoveRight,
    MoveLeft,
    MoveUp,
    MoveDown,
}

#[derive(Deserialize)]
struct TestConfig {
    commands: Vec<TestCommandConfig>,
}

#[derive(Deserialize)]
struct TestCommandConfig {
    command: String,
    description: String,
    wait_time: f32,
}

/// Resource for automated testing
#[derive(Resource)]
struct AutomatedTester {
    commands: Vec<TestCommand>,
    current_command: usize,
    command_timer: Timer,
    screenshot_timer: Timer,
    waiting_for_screenshot: bool,
}

#[derive(Clone)]
struct TestCommand {
    command_type: TestCommandType,
    description: String,
    wait_time: f32, // Time to wait before taking screenshot
}

impl AutomatedTester {
    fn new() -> Self {
        let commands = Self::load_commands_from_file();

        Self {
            commands,
            current_command: 0,
            command_timer: Timer::from_seconds(1.0, TimerMode::Once), // Initial delay
            screenshot_timer: Timer::from_seconds(0.5, TimerMode::Once),
            waiting_for_screenshot: false,
        }
    }
    
    fn load_commands_from_file() -> Vec<TestCommand> {
        let config_path = "test_commands_simple.yaml";
        
        match fs::read_to_string(config_path) {
            Ok(content) => {
                match serde_yaml::from_str::<TestConfig>(&content) {
                    Ok(config) => {
                        println!("📄 Loaded {} commands from {}", config.commands.len(), config_path);
                        config.commands.into_iter().map(|cmd| {
                            let command_type = match cmd.command.as_str() {
                                "toggle_gizmos" => TestCommandType::ToggleGizmos,
                                "move_right" => TestCommandType::MoveRight,
                                "move_left" => TestCommandType::MoveLeft,
                                "move_up" => TestCommandType::MoveUp,
                                "move_down" => TestCommandType::MoveDown,
                                // Support old commands for backward compatibility
                                "move_forward" => TestCommandType::MoveUp,
                                "move_backward" => TestCommandType::MoveDown,
                                _ => {
                                    println!("⚠️ Unknown command type: {}, defaulting to move_right", cmd.command);
                                    TestCommandType::MoveRight
                                }
                            };
                            
                            TestCommand {
                                command_type,
                                description: cmd.description,
                                wait_time: cmd.wait_time,
                            }
                        }).collect()
                    }
                    Err(e) => {
                        println!("⚠️ Failed to parse {}: {}", config_path, e);
                        Self::fallback_commands()
                    }
                }
            }
            Err(e) => {
                println!("⚠️ Failed to read {}: {}", config_path, e);
                Self::fallback_commands()
            }
        }
    }
    
    fn fallback_commands() -> Vec<TestCommand> {
        println!("📄 Using fallback test commands");
        vec![
            TestCommand {
                command_type: TestCommandType::ToggleGizmos,
                description: "Enable debug gizmos".to_string(),
                wait_time: 0.5,
            },
            TestCommand {
                command_type: TestCommandType::MoveUp,
                description: "Move up 1".to_string(),
                wait_time: 1.0,
            },
            TestCommand {
                command_type: TestCommandType::MoveUp,
                description: "Move up 2".to_string(),
                wait_time: 1.0,
            },
            TestCommand {
                command_type: TestCommandType::MoveUp,
                description: "Move up 3".to_string(),
                wait_time: 1.0,
            },
        ]
    }
}

static TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const TEXT_SIZE: f32 = 15.0;

fn main() {
    println!("🚀 Starting miniature-potato with geotiles geodesic polyhedron");

    // Check for flags
    let args: Vec<String> = env::args().collect();
    let screenshot_mode = args.contains(&"--screenshot".to_string());
    let test_mode = args.contains(&"--test".to_string());

    if screenshot_mode {
        println!("📸 Screenshot mode enabled - will take screenshot after 2 seconds and exit");
    } else if test_mode {
        println!("🤖 Automated test mode enabled - will run movement commands and take screenshots");
    }

    let mut app = App::new();

    // Configure plugins with smaller window for screenshot/test mode
    if screenshot_mode || test_mode {
        app.add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "miniature-potato (screenshot)".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }),));

        if screenshot_mode {
            app.insert_resource(ScreenshotTimer {
                timer: Timer::from_seconds(2.0, TimerMode::Once),
                should_screenshot: true,
                exit_timer: None,
            })
            .add_systems(Update, (screenshot_system,));
        } else if test_mode {
            app.add_event::<AutomatedTestEvent>()
            .insert_resource(AutomatedTester::new())
            .add_systems(Update, (
                automated_test_system, 
                handle_automated_test_events,
            ));
        }
    } else {
        app.add_plugins((DefaultPlugins,));

        app.insert_resource(WireframeConfig {
            global: false, // Disable wireframe by default
            default_color: Color::WHITE,
        });
    }

    let plugin_group = (
        WireframePlugin::default(),
        MeshPickingPlugin,
        PanOrbitCameraPlugin,
    );
    
    // Only add performance diagnostics in normal mode (not test mode)
    if !test_mode {
        app.add_plugins((
            plugin_group,
            // we want Bevy to measure these values for us:
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            bevy::diagnostic::EntityCountDiagnosticsPlugin,
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
            bevy::render::diagnostic::RenderDiagnosticsPlugin,
            // to be shown in this plugin:
            PerfUiPlugin,
        ));
    } else {
        app.add_plugins(plugin_group);
    }

    // Initialize character resource (needed in all modes)
    app.insert_resource(CharacterResource { 
        entity: None,
    });
    
    // Initialize debug gizmos resource
    app.insert_resource(DebugGizmosResource::default());

    app.add_systems(
        Startup,
        (
            setup_hexasphere_world,
            setup_character.after(setup_hexasphere_world),
            move |commands: Commands| setup_ui(commands, test_mode),
            setup_camera,
            setup_lighting,
        ),
    );

    app.add_systems(
        Update,
        (
            draw_mesh_intersections,
            handle_escape_key,
            toggle_wireframe,
            toggle_normals,
            print_tile_info,
            handle_tile_selection,
            handle_character_movement,
            tile_gizmos_system,
            debug_gizmos_system.after(update_dead_zone_state),
            toggle_debug_gizmos,
            update_hovered_tile_ui,
            update_selected_tile_ui,
            update_sphere_info_ui,
        ),
    );

    app.run();
}

/// System to handle escape key press for quitting the app
fn handle_escape_key(keyboard_input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
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
        println!(
            "Wireframe mode: {}",
            if wireframe_config.global { "ON" } else { "OFF" }
        );
    }
}

/// System to print tile information when 'I' is pressed
fn print_tile_info(
    keyboard: Res<ButtonInput<KeyCode>>,
    hexasphere_res: Option<Res<HexasphereResource>>,
) {
    if keyboard.just_pressed(KeyCode::KeyI) {
        if let Some(hexasphere) = hexasphere_res {
            println!("\n📊 === HEXASPHERE INFO ===");
            println!("Total tiles: {}", hexasphere.hexasphere.tiles.len());

            let pentagon_count = hexasphere
                .hexasphere
                .tiles
                .iter()
                .filter(|t| t.boundary.len() == 5)
                .count();
            let hexagon_count = hexasphere
                .hexasphere
                .tiles
                .iter()
                .filter(|t| t.boundary.len() == 6)
                .count();

            println!("Pentagons: {}", pentagon_count);
            println!("Hexagons: {}", hexagon_count);
            println!("Sphere radius: {}", hexasphere.hexasphere.radius);
            println!("Uniform hexagon radius: {:.3}", hexasphere.uniform_radius);

            // Calculate statistics
            let stats = hexasphere.hexasphere.calculate_hexagon_stats();
            println!("\n📐 Hexagon Statistics:");
            println!("Average radius: {:.3}", stats.average_hexagon_radius);
            println!("Min radius: {:.3}", stats.min_hexagon_radius);
            println!("Max radius: {:.3}", stats.max_hexagon_radius);
            println!(
                "Size variation: {:.1}%",
                100.0 * (stats.max_hexagon_radius - stats.min_hexagon_radius)
                    / stats.average_hexagon_radius
            );
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
                    let tile_type = if tile.boundary.len() == 5 {
                        "Pentagon"
                    } else {
                        "Hexagon"
                    };
                    let center = &tile.center_point;
                    let lat_lon = tile.get_lat_lon(hexasphere.hexasphere.radius);

                    **text = format!(
                        "Type: {}\nCenter: ({:.2}, {:.2}, {:.2})\nLat/Lon: {:.1}°, {:.1}°\nNeighbors: {}\nHovered: {}",
                        tile_type,
                        center.x, center.y, center.z,
                        lat_lon.lat, lat_lon.lon,
                        tile.neighbors.len(),
                        hovered_index,
                    );
                } else {
                    **text =
                        "Type:\nCenter:\nLat/Lon:\nNeighbors:\nHovered: Invalid tile".to_string();
                }
            } else {
                **text = "Type:\nCenter:\nLat/Lon:\nNeighbors:\nHovered: None".to_string();
            }
        } else {
            **text = "Type:\nCenter:\nLat/Lon:\nNeighbors:\nHovered: Loading...".to_string();
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
            let selected_count = hexasphere.selected_tiles.len();
            if selected_count == 0 {
                **text = "Selected: None".to_string();
            } else {
                **text = format!("Selected: {} tiles", selected_count);
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
            let pentagon_count = hexasphere
                .hexasphere
                .tiles
                .iter()
                .filter(|t| t.boundary.len() == 5)
                .count();
            let hexagon_count = hexasphere
                .hexasphere
                .tiles
                .iter()
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

/// Component to mark the hovered tile info text
#[derive(Component)]
struct HoveredTileText;

/// Component to mark the selected tile info text
#[derive(Component)]
struct SelectedTileText;

/// Component to mark the sphere info text
#[derive(Component)]
struct SphereInfoText;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 15.0, 0.0)),
        PanOrbitCamera {
            // Fixed camera position looking at the sphere
            // Sphere rotation will handle keeping character centered
            yaw: Some(0.0),
            target_yaw: 0.0,
            // Keep pitch locked to maintain view angle
            pitch_upper_limit: Some(0.0),
            pitch_lower_limit: Some(0.0),
            // Disable manual orbit controls since sphere handles rotation
            orbit_sensitivity: 0.0,
            ..default()
        },
    ));
}

fn setup_lighting(mut commands: Commands) {
    // Add lighting - brighter setup for better visibility
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0, // Increased brightness
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // Add additional directional light from another angle
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 0.5, 0.5, 0.0)),
    ));

    // this light appears to do the highlighting
    commands.insert_resource(AmbientLight {
        color: Color::srgb(1.0, 1.0, 1.0),
        brightness: 0.3, // Increased ambient light for better visibility
        ..default()
    });
}

/// Setup the UI with on-screen controls help and tile info displays
fn setup_ui(mut commands: Commands, test_mode: bool) {
    // Don't show performance UI in test mode
    if !test_mode {
        commands.spawn(PerfUiAllEntries::default());
    }

    // Controls help in top-left (simplified in test mode)
    let controls_text = if test_mode {
        "🤖 Automated Testing Mode\n\
        Running movement commands...\n\
        Check console for progress"
    } else {
        "Controls:\n\
        * Mouse wheel: Zoom camera\n\
        * Left click: Select/deselect tile\n\
        * WASD: Move character\n\
        * Space: Toggle wireframe mode\n\
        * N: Toggle normal visualization\n\
        * G: Toggle debug axes gizmos\n\
        * I: Print hexasphere info\n\
        * Esc: Quit"
    };
    
    commands.spawn((
        Text::new(controls_text),
        TextFont {
            font_size: TEXT_SIZE,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));

    // Sphere info in bottom-right
    commands.spawn((
        Text::new("Sphere Info:\nLoading..."),
        TextFont {
            font_size: TEXT_SIZE,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            right: Val::Px(12.0),
            ..default()
        },
        SphereInfoText,
    ));

    // Hovered tile info in bottom-left
    commands.spawn((
        Text::new("Hovered: None"),
        TextFont {
            font_size: TEXT_SIZE,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        HoveredTileText,
    ));

    // Selected tile info in bottom-center
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            width: Val::Percent(100.0),
            height: Val::Auto,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            // Selected tile info (now centered horizontally)
            parent.spawn((
                Text::new("Selected: None"),
                TextFont {
                    font_size: TEXT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node::default(), // No positioning needed, flexbox handles it
                SelectedTileText,
            ));
        });

    println!("   Run with --screenshot flag to auto-capture screenshot and exit");
    println!("   Run with --test flag to run automated movement testing with screenshots");
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

            // Use current directory for screenshot
            let filename = format!("screenshot_{}.png", timestamp);

            println!("📸 Taking screenshot: {}", filename);

            // Try to get the window entity first
            if let Ok(window_entity) = windows.single() {
                println!("Found window entity: {:?}", window_entity);

                // Take screenshot using the new Bevy 0.16.1 API with specific window
                use bevy::render::camera::RenderTarget;
                use bevy::window::WindowRef;
                commands
                    .spawn(Screenshot(RenderTarget::Window(WindowRef::Entity(
                        window_entity,
                    ))))
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

            println!(
                "✅ Screenshot command sent as {}, will exit in 1 second...",
                filename
            );
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

/// System to run automated movement tests and capture screenshots
fn automated_test_system(
    time: Res<Time>,
    mut tester: ResMut<AutomatedTester>,
    mut commands: Commands,
    mut test_events: EventWriter<AutomatedTestEvent>,
    windows: Query<Entity, With<Window>>,
    mut exit: EventWriter<AppExit>,
) {
    // Check if we've completed all commands
    if tester.current_command >= tester.commands.len() {
        // Wait a bit before exiting
        if !tester.waiting_for_screenshot {
            println!("🎉 All automated tests completed! Exiting in 2 seconds...");
            tester.waiting_for_screenshot = true;
            tester.screenshot_timer = Timer::from_seconds(2.0, TimerMode::Once);
        }
        
        tester.screenshot_timer.tick(time.delta());
        if tester.screenshot_timer.just_finished() {
            exit.write(AppExit::Success);
        }
        return;
    }

    if tester.waiting_for_screenshot {
        // We're waiting to take a screenshot after the last command
        tester.screenshot_timer.tick(time.delta());
        
        if tester.screenshot_timer.just_finished() {
            // Take screenshot
            let command = &tester.commands[tester.current_command];
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            let filename = format!("test_{:02}_{}.png", tester.current_command, timestamp);
            
            println!("📸 Taking test screenshot: {} ({})", filename, command.description);
            
            if let Ok(window_entity) = windows.single() {
                use bevy::render::camera::RenderTarget;
                use bevy::window::WindowRef;
                commands
                    .spawn(Screenshot(RenderTarget::Window(WindowRef::Entity(window_entity))))
                    .observe(save_to_disk(filename));
            }
            
            // Move to next command
            tester.current_command += 1;
            tester.waiting_for_screenshot = false;
            tester.command_timer = Timer::from_seconds(1.0, TimerMode::Once); // Wait before next command
        }
    } else {
        // We're waiting to execute the next command
        tester.command_timer.tick(time.delta());
        
        if tester.command_timer.just_finished() {
            if tester.current_command < tester.commands.len() {
                let command = &tester.commands[tester.current_command].clone();
                
                println!("🤖 Executing test command {}: {}", tester.current_command + 1, command.description);
                
                // Send test event
                test_events.write(AutomatedTestEvent {
                    command_type: command.command_type.clone(),
                    description: command.description.clone(),
                });
                
                // Set up screenshot timer
                tester.waiting_for_screenshot = true;
                tester.screenshot_timer = Timer::from_seconds(command.wait_time, TimerMode::Once);
            }
        }
    }
}

/// System to handle automated test events and trigger actions
fn handle_automated_test_events(
    mut test_events: EventReader<AutomatedTestEvent>,
    mut debug_gizmos: ResMut<DebugGizmosResource>,
    hexasphere_res: Option<Res<HexasphereResource>>,
    world_parent_query: Query<&Transform, (With<WorldParent>, Without<Character>)>,
    mut character_query: Query<(&mut Character, &mut Transform)>,
) {
    for event in test_events.read() {
        println!("🔧 Processing automated test event: {}", event.description);
        
        match event.command_type {
            TestCommandType::ToggleGizmos => {
                debug_gizmos.show_axes = !debug_gizmos.show_axes;
                debug_gizmos.show_dead_zone = debug_gizmos.show_axes; // Same state as axes
                println!("Debug gizmos: {} (axes: {}, dead zone: {})", 
                         if debug_gizmos.show_axes { "ON" } else { "OFF" },
                         debug_gizmos.show_axes,
                         debug_gizmos.show_dead_zone);
            },
            TestCommandType::MoveRight => {
                simulate_movement(KeyCode::KeyD, &hexasphere_res, &world_parent_query, &mut character_query);
            },
            TestCommandType::MoveLeft => {
                simulate_movement(KeyCode::KeyA, &hexasphere_res, &world_parent_query, &mut character_query);
            },
            TestCommandType::MoveUp => {
                simulate_movement(KeyCode::KeyW, &hexasphere_res, &world_parent_query, &mut character_query);
            },
            TestCommandType::MoveDown => {
                simulate_movement(KeyCode::KeyS, &hexasphere_res, &world_parent_query, &mut character_query);
            },
        }
    }
}

/// Helper function to simulate movement without keyboard input
fn simulate_movement(
    key: KeyCode,
    hexasphere_res: &Option<Res<HexasphereResource>>,
    world_parent_query: &Query<&Transform, (With<WorldParent>, Without<Character>)>,
    character_query: &mut Query<(&mut Character, &mut Transform)>,
) {
    // Functions already imported at module level
    
    if let Some(hexasphere) = hexasphere_res {
        if let (Ok((mut character, mut transform)), Ok(world_transform)) = 
            (character_query.single_mut(), world_parent_query.single()) {
            
            let current_tile_index = character.current_tile;
            
            // Get current tile
            if let Some(current_tile) = hexasphere.hexasphere.tiles.get(current_tile_index) {
                // Get tile center and transform it by world rotation to get actual world position
                let local_center = vec3_from_point(&current_tile.center_point);
                let current_center = world_transform.transform_point(local_center);
                
                // Define movement directions in camera space (camera looks down -Z)
                let camera_relative_direction = match key {
                    KeyCode::KeyW => Some(Vec3::Y),  // Up (toward top of screen/sphere)
                    KeyCode::KeyS => Some(-Vec3::Y), // Down (toward bottom of screen/sphere)
                    KeyCode::KeyA => Some(-Vec3::X), // Left
                    KeyCode::KeyD => Some(Vec3::X),  // Right
                    _ => None,
                };
                
                if let Some(camera_direction) = camera_relative_direction {
                    // Use camera direction directly in world space - don't apply world rotation
                    // This keeps WASD movement consistent relative to camera view regardless of world rotation
                    let world_direction = camera_direction;
                    
                    // Project the desired direction onto the sphere's tangent plane at current position
                    let normal = current_center.normalize(); // Surface normal at current position
                    let tangent_direction = (world_direction - normal * world_direction.dot(normal)).normalize();
                    
                    // Find the neighbor that best matches this direction
                    if let Some(target_tile_index) = find_neighbor_in_direction(current_tile, tangent_direction, &hexasphere.hexasphere, world_transform) {
                        if let Some(target_tile) = hexasphere.hexasphere.tiles.get(target_tile_index) {
                            // Transform target tile center to world space
                            let local_target_center = vec3_from_point(&target_tile.center_point);
                            let target_center = world_transform.transform_point(local_target_center);
                            let new_position = target_center + target_center.normalize() * character.hover_height;
                            
                            // Convert world position back to local space relative to world parent
                            // Create inverse transform manually
                            let inv_rotation = world_transform.rotation.inverse();
                            let inv_translation = inv_rotation * (-world_transform.translation);
                            let local_position = inv_rotation * new_position + inv_translation;
                            
                            // Update character position and current tile
                            transform.translation = local_position;
                            character.current_tile = target_tile_index;
                            
                            let tile_type = if target_tile.boundary.len() == 5 { "pentagon" } else { "hexagon" };
                            println!("🚶 Character moved to {} tile #{}", tile_type, target_tile_index);
                        }
                    } else {
                        println!("🚫 No valid neighbor found in that direction");
                    }
                }
            }
        }
    }
}

/// A system that draws hit indicators for every pointer.
fn draw_mesh_intersections(pointers: Query<&PointerInteraction>, mut gizmos: Gizmos) {
    for (point, normal) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        gizmos.sphere(point, 0.05, RED_500);
        gizmos.arrow(point, point + normal.normalize() * 0.5, PINK_100);
    }
}
