use bevy::prelude::*;
mod potato_mesh;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_potato)
        .run();
}

#[derive(Component)]
struct Potato;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a potato-shaped mesh using our imported function
    // You can change which potato variant to use here:
    // let potato_mesh = potato_mesh::create_potato_mesh();  // Default potato
    // let potato_mesh = potato_mesh::create_red_potato_mesh(); // Smoother, rounder potato
    let potato_mesh = potato_mesh::create_red_potato_mesh(); // Using the red potato variant
    
    // Add the potato to the scene
    commands.spawn((
        Mesh3d(meshes.add(potato_mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.7, 0.5), // Potato-like color
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Potato,
    ));

    // Add a directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add an ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
        affects_lightmapped_meshes: false,
    });

    // Add a camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn rotate_potato(mut query: Query<&mut Transform, With<Potato>>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        // Rotate around multiple axes for an interesting effect
        transform.rotate_x(0.1 * time.delta_secs());
        transform.rotate_y(0.2 * time.delta_secs());
        transform.rotate_z(0.05 * time.delta_secs());
    }
}