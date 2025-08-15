use bevy::prelude::*;
use dashmap::DashMap;
use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

use crate::{
    chunk::{Chunk, ChunkMap, ChunkPos},
    voxel_viewer::VoxelViewer,
};

#[derive(Debug, Component, Default)]
pub struct VoxelVolume {
    chunks: Arc<ChunkMap>,
}

pub fn update_visible_chunks(
    mut commands: Commands,
    viewers: Query<(&VoxelViewer, &Transform)>,
    volumes: Query<(Entity, &VoxelVolume, &Transform)>,
) {
    for (entity, mut volume, volume_transform) in volumes {
        let chunks = &mut volume.chunks;

        let visible_chunks = viewers
            .iter()
            .flat_map(|(viewer, transform)| {
                let chunk_pos =
                    global_to_chunk(transform.translation - volume_transform.translation);
                viewer.visible_positions(chunk_pos)
            })
            .collect::<HashSet<_>>();

        for chunk_pos in &visible_chunks {
            if !chunks.contains_key(chunk_pos) {
                let chunk_pos = *chunk_pos;

                let child_entity = commands
                    .spawn((
                        ChunkFlag,
                        Transform::from_translation(chunk_pos_to_global_pos(chunk_pos)),
                        ChunkConstructorTask::new(move || generator::temp(chunk_pos)),
                    ))
                    .id();

                commands.entity(entity).add_child(child_entity);

                let a = RwLock::new(2);

                chunks.insert(chunk_pos, child_entity);
            }
        }

        let unload: Vec<IVec3> = chunks
            .keys()
            .filter(|k| !visible_chunks.contains(k))
            .copied()
            .collect();

        for chunk_pos in unload {
            let entity = chunks.remove(&chunk_pos).unwrap();
            commands.entity(entity).despawn();
        }
    }
}
