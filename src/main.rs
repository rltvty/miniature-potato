use bevy::{
    prelude::*,
    pbr::wireframe::{WireframePlugin, WireframeConfig},
    render::{
        render_resource::WgpuFeatures,
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};
mod geodesic_potato;
mod hexapent_potato;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                // WARN this is a native only feature. It will not work with webgl or webgpu
                features: WgpuFeatures::POLYGON_MODE_LINE,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(WireframePlugin::default())
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::BLACK.into(),
        })
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
    // Create two parts: solid potato and hexagon/pentagon wireframe
    
    // First, the solid potato
    commands.spawn((
        Mesh3d(meshes.add(geodesic_potato::create_geodesic_potato_mesh())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.7, 0.5), // Potato-like color
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(0.96)), // Slightly smaller
        Potato,
    ));

    // Then, a line mesh showing only the hexagon/pentagon edges
    commands.spawn((
        Mesh3d(meshes.add(hexapent_potato::create_potato_wireframe_mesh())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 0.0, 0.0), // Black for wireframe
            emissive: Color::srgb(0.0, 0.0, 0.0).into(),
            unlit: true, // Unaffected by lighting
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(1.02)), // Slightly larger
        Potato, // Mark as potato for rotation
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