use bevy::prelude::*;
use bevy::gltf::{Gltf, GltfMesh};
use avian3d::prelude::*;
use glft_info::GltfInfoComponent;
use crate::glft_info;
use bevy::render::mesh::Mesh;

pub struct PotatoPlugin;

impl Plugin for PotatoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(Update, process_mesh_load);
    }
}

#[derive(Component)]
struct Potato {
    gltf_handle: Handle<Gltf>,
    initalized: bool,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let gltf_handle = asset_server.load("potato-1.glb");

    commands.spawn_empty().insert(GltfInfoComponent { handle: gltf_handle.clone() });

    // Instead of trying to load the mesh directly, we defer this to a system that checks for the loaded asset
    commands.spawn_empty().insert(Potato { gltf_handle, initalized: false });
}



// System that waits for the mesh to be loaded
fn process_mesh_load(
    mut commands: Commands,
    mut query: Query<&mut Potato>,
    gltf_assets: Res<Assets<Gltf>>,  // Loaded GLTF assets
    gltf_mesh_assets: Res<Assets<GltfMesh>>,  // Access to GltfMeshes
    mesh_assets: Res<Assets<Mesh>>,  // Access to loaded Meshes
) {
    for mut potato in query.iter_mut() {
        if potato.initalized {
            continue;
        }
        let handle = &potato.gltf_handle; 
        if let Some(gltf) = gltf_assets.get(handle) {
            let mut mesh: Option<&Mesh> = Option::None;
            let mut scene_handle: Option<&Handle<Scene>> = Option::None;

            if gltf.meshes.len() == 1 {
                let gltf_mesh_handle = &gltf.meshes[0];
                if let Some(gltf_mesh) = gltf_mesh_assets.get(gltf_mesh_handle) {
                    if gltf_mesh.primitives.len() == 1 {
                        if let Some(mesh_found) = mesh_assets.get(&gltf_mesh.primitives[0].mesh) {
                            mesh = Some(mesh_found);
                            println!("Found Mesh for potato");
                        }
                    }
                }
            }

            if gltf.scenes.len() == 1 {
                scene_handle = Some(&gltf.scenes[0]);
                println!("Found Scene Handle for potato");
            }

            if scene_handle.is_some() && mesh.is_some() {
                commands.spawn((
                    SceneBundle {
                        scene: scene_handle.unwrap().clone(),
                        transform: Transform {
                            //translation: Vec3::new(10.0, 0.0, 10.1),
                            scale: Vec3::splat(1000.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    RigidBody::Static,
                    Collider::trimesh_from_mesh(mesh.unwrap()).unwrap(),
                ));
                potato.initalized = true;
                println!("Potato initalized.");
            }
        }
    }
}
