use std::collections::{HashMap, HashSet};

use crate::{
    ecs::{
        chunk::{ChunkConstructor, ChunkData, ChunkMesher},
        chunk_pos_to_global_pos, global_pos_to_chunk_pos,
        voxel_viewer::VoxelViewer,
    },
    generator,
};
use bevy::prelude::*;

#[derive(Debug, Component, Default)]
pub struct VoxelVolume {
    chunks: HashMap<IVec3, Entity>,
}

pub fn active_chunks(
    mut commands: Commands,
    viewers: Query<(&VoxelViewer, &Transform)>,
    volumes: Query<(&mut VoxelVolume, &Transform)>,

    chunk_data: Query<&ChunkData>,
    chunk_meshes: Query<&Mesh3d, With<ChunkData>>,
    chunk_meshers: Query<&ChunkMesher>,
) {
    for (mut volume, volume_transform) in volumes {
        let chunks = &mut volume.chunks;

        let visible_chunks = viewers
            .iter()
            .flat_map(|(viewer, transform)| {
                let chunk_pos =
                    global_pos_to_chunk_pos(transform.translation - volume_transform.translation);
                viewer.visible_positions(chunk_pos)
            })
            .collect::<HashSet<_>>();

        for chunk_pos in &visible_chunks {
            match chunks.get(chunk_pos) {
                Some(&entity) => {
                    let Ok(chunk_data) = chunk_data.get(entity) else {
                        continue;
                    };

                    if chunk_meshers.get(entity).is_err() && chunk_meshes.get(entity).is_err() {
                        commands
                            .entity(entity)
                            .insert(ChunkMesher::new(chunk_data.0.clone()));
                    }
                }
                None => {
                    let chunk_pos = *chunk_pos;
                    let entity = commands
                        .spawn((
                            Transform::from_translation(chunk_pos_to_global_pos(chunk_pos)),
                            ChunkConstructor::new(move || generator::flat(chunk_pos)),
                        ))
                        .id();

                    chunks.insert(chunk_pos, entity);
                }
            }
        }

        let unload: Vec<IVec3> = chunks
            .keys()
            .filter(|k| !visible_chunks.contains(k))
            .copied()
            .collect();

        for chunk_pos in unload {
            let entity = chunks.remove(&chunk_pos).unwrap();
            // stop any async tasks
            commands.entity(entity).despawn();
        }
    }
}
