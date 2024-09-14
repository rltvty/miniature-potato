use bevy::prelude::*;

pub struct PotatoPlugin;

impl Plugin for PotatoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,  // Add the AssetServer to load glTF files
) {

    let gltf_scene = asset_server.load("potato-1.glb#Scene0");


    // Spawn the glTF model into the scene
    commands.spawn(SceneBundle {
        scene: gltf_scene,
        transform: Transform {
            translation: Vec3::ZERO,  // Place the object at the origin
            scale: Vec3::splat(5.0),  // Ensure the object is not too small or too large
            ..Default::default()
        },
        ..Default::default()
    });
}
