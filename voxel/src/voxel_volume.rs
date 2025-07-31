use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

pub use voxel_viewer::*;

use crate::{
    chunk::{ChunkConstructorTask, ChunkFlag},
    data::chunk,
    generator,
};

mod voxel_viewer {
    use bevy::prelude::*;

    #[derive(Debug, Component)]
    pub struct VoxelViewer {
        pub view_distance: u8,
    }

    impl VoxelViewer {
        pub fn new(view_distance: u8) -> Self {
            Self { view_distance }
        }

        pub fn visible_positions(&self, chunk_pos: IVec3) -> impl Iterator<Item = IVec3> {
            sphere_iter(chunk_pos, self.view_distance)
        }
    }

    fn sphere_iter(position: IVec3, radius: u8) -> impl Iterator<Item = IVec3> {
        let radius = radius as i32;
        let radius_sq = radius.pow(2);

        (-radius..=radius).flat_map(move |x| {
            (-radius..=radius).flat_map(move |y| {
                (-radius..=radius).filter_map(move |z| {
                    let offset = IVec3::new(x, y, z);
                    if offset.length_squared() <= radius_sq {
                        Some(offset + position)
                    } else {
                        None
                    }
                })
            })
        })
    }
}

pub fn global_pos_to_chunk_pos(global: Vec3) -> IVec3 {
    (global / chunk::LENGTH).floor().as_ivec3()
}

pub fn chunk_pos_to_global_pos(chunk_pos: IVec3) -> Vec3 {
    chunk_pos.as_vec3() * chunk::LENGTH
}

#[derive(Debug, Component, Default)]
pub struct VoxelVolume {
    chunks: HashMap<IVec3, Entity>,
}

pub fn update_visible_chunks(
    mut commands: Commands,
    viewers: Query<(&VoxelViewer, &Transform)>,
    volumes: Query<(Entity, &mut VoxelVolume, &Transform)>,
) {
    for (entity, mut volume, volume_transform) in volumes {
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
