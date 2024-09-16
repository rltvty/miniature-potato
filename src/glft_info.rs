use bevy::gltf::{Gltf, GltfMesh};
use bevy::prelude::*;
use std::collections::HashSet;

// Custom component to store the GLTF handle for each entity
#[derive(Component)]
pub struct GltfInfoComponent {
    pub handle: Handle<Gltf>,
}

// Resource to keep track of the GLTF handles we have already printed debug info for
#[derive(Resource, Default)]
pub struct PrintedHandles {
    pub handles: HashSet<Handle<Gltf>>, // A set to track printed handles
}

pub struct GltfInfoPlugin;

impl Plugin for GltfInfoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PrintedHandles::default()) // Insert the resource to track printed handles
            .add_systems(Update, print_gltf_info);
    }
}

fn print_gltf_info(
    mut printed_handles: ResMut<PrintedHandles>, // Resource to track printed handles
    query: Query<(&GltfInfoComponent, Entity)>,  // Query for entities with GltfInfoComponent
    gltf_assets: Res<Assets<Gltf>>,              // Loaded GLTF assets
    gltf_mesh_assets: Res<Assets<GltfMesh>>,     // Access to GltfMeshes
    mesh_assets: Res<Assets<Mesh>>,              // Access to loaded Meshes
) {
    for (gltf_info_component, entity) in query.iter() {
        let handle = &gltf_info_component.handle;

        // Check if we've already printed the debug info for this handle
        if !printed_handles.handles.contains(handle) {
            if let Some(gltf) = gltf_assets.get(handle) {
                println!("GLTF file for entity {:?} loaded:", entity);
                println!(" - Scenes: {:?}", gltf.scenes);
                println!(" - Meshes: {:?}", gltf.meshes);
                println!(" - Materials: {:?}", gltf.materials);

                // Loop through the GltfMesh handles and print more detailed mesh info
                for gltf_mesh_handle in &gltf.meshes {
                    if let Some(gltf_mesh) = gltf_mesh_assets.get(gltf_mesh_handle) {
                        // GltfMesh contains `primitives`, which are the actual meshes.
                        for primitive in &gltf_mesh.primitives {
                            if let Some(mesh) = mesh_assets.get(&primitive.mesh) {
                                println!(
                                    "Primitive mesh loaded with vertex count: {}",
                                    mesh.count_vertices()
                                );
                            } else {
                                println!("Primitive mesh not yet loaded.");
                            }
                        }
                    } else {
                        println!("GLTF mesh not yet loaded.");
                    }
                }

                // After printing the debug info, add the handle to the set
                printed_handles.handles.insert(handle.clone());
            } else {
                println!("GLTF file for entity {:?} is still loading.", entity);
            }
        }
    }
}
