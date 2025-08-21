// use bevy::prelude::*;

// use std::{
//     collections::HashSet,
//     sync::{Arc, RwLock},
// };

// use crate::{
//     chunk::{Chunk, ChunkMap, ChunkPos},
//     voxel_viewer::VoxelViewer,
// };

// #[derive(Debug, Component, Default)]
// pub struct VoxelVolume {
//     chunk_map: Arc<ChunkMap>,
// }

// pub fn visible_chunks(
//     mut commands: Commands,
//     viewers: Query<(&VoxelViewer, &Transform)>,
//     volumes: Query<(&VoxelVolume, &Transform)>,
// ) {
//     for (volume, volume_transform) in volumes {
//         let chunk_map = &volume.chunk_map;

//         let visible_chunks = viewers
//             .iter()
//             .flat_map(|(viewer, transform)| {
//                 let chunk_pos = (transform.translation - volume_transform.translation).into();
//                 viewer.visible_positions(chunk_pos)
//             })
//             .collect::<HashSet<_>>();

//         for chunk_pos in &visible_chunks {
//             if !chunk_map.contains_key(chunk_pos) {
//                 let chunk_pos = *chunk_pos;

//                 let child_entity = commands
//                     .spawn((
//                         Transform::from_translation(chunk_pos.as_world()),
//                         ChunkConstructorTask::new(move || generator::temp(chunk_pos)),
//                     ))
//                     .id();

//                 commands.entity(entity).add_child(child_entity);

//                 let a = RwLock::new(2);

//                 chunks.insert(chunk_pos, child_entity);
//             }
//         }

//         let unload: Vec<IVec3> = chunks
//             .keys()
//             .filter(|k| !visible_chunks.contains(k))
//             .copied()
//             .collect();

//         for chunk_pos in unload {
//             let entity = chunks.remove(&chunk_pos).unwrap();
//             commands.entity(entity).despawn();
//         }
//     }
// }
