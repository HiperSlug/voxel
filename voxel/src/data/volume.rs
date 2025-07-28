use crate::data::brick::{self, Brick};
use crate::data::chunk::{self, Chunk};
use crate::data::voxel::Voxel;
use crate::mesher;

use bevy::prelude::*;
use bevy::tasks::{Task, poll_once, prelude::*};
use rand::random;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

#[derive(Component)]
pub struct ChunkData(Arc<RwLock<Chunk>>);

#[derive(Component)]
pub struct ChunkConstructor(Task<Chunk>);

pub fn generate_chunk(chunk_pos: IVec3) -> Task<Chunk> {
    let thread_pool = AsyncComputeTaskPool::get();
    thread_pool.spawn(async move { temp_gen(chunk_pos) })
}

pub fn poll_chunk_constructor(
    mut commands: Commands,
    query: Query<(Entity, &mut ChunkConstructor)>,
) {
    for (entity, mut task) in query {
        if let Some(generated_chunk) = block_on(poll_once(&mut task.0)) {
            let chunk = Arc::new(RwLock::new(generated_chunk));
            commands
                .entity(entity)
                .remove::<ChunkConstructor>()
                .insert(ChunkData(chunk.clone()))
                .insert(ChunkMesher(mesh_chunk(chunk)));
        }
    }
}

#[derive(Component)]
pub struct ChunkMesher(Task<Mesh>);

pub fn mesh_chunk(chunk: Arc<RwLock<Chunk>>) -> Task<Mesh> {
    let thread_pool = AsyncComputeTaskPool::get();
    thread_pool.spawn(async move {
        let read_guard = chunk.read().expect("lock poisoned");
        let mesh = mesher::mesh(&read_guard);
        mesh
    })
}

pub fn poll_chunk_mesher(
    mut commands: Commands,
    query: Query<(Entity, &mut ChunkMesher)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut task) in query {
        if let Some(mesh) = block_on(poll_once(&mut task.0)) {
            commands.entity(entity).remove::<ChunkMesher>().insert((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(Color::srgb_u8(random(), random(), random()))), // I have no idea how materials will later work.
            ));
        }
    }
}

use voxel_viewer::VoxelViewer;

pub mod voxel_viewer {
    use bevy::prelude::*;

    use super::global_pos_to_chunk_pos;

    #[derive(Component)]
    pub struct VoxelViewer {
        pub view_distance: u8,
    }

    impl VoxelViewer {
        pub fn visible(&self, pos: Vec3, vol_pos: Vec3) -> impl Iterator<Item = IVec3> {
            let viewer_chunk = global_pos_to_chunk_pos(pos - vol_pos);

            sphere_iter(viewer_chunk, self.view_distance)
        }
    }

    fn sphere_iter(position: IVec3, radius: u8) -> impl Iterator<Item = IVec3> {
        let radius = radius as i32;
        let radius_sq = radius.pow(2);

        (-radius..=radius).flat_map(move |x| {
            (-radius..=radius).flat_map(move |y| {
                (-radius..=radius).filter_map(move |z| {
                    let iter_position = IVec3::new(x, y, z);
                    if iter_position.length_squared() <= radius_sq {
                        Some(iter_position + position)
                    } else {
                        None
                    }
                })
            })
        })
    }
}

#[derive(Component, Default)]
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
            .flat_map(|(viewer, tr)| viewer.visible(tr.translation, volume_transform.translation))
            .collect::<HashSet<_>>();

        for chunk_pos in &visible_chunks {
            match chunks.get(&chunk_pos) {
                // hopefully this will be easy to integrate with LOD if I change how the visible function works
                Some(&entity) => {
                    let Ok(chunk_data) = chunk_data.get(entity) else {
                        continue;
                    };

                    if chunk_meshers.get(entity).is_err() && chunk_meshes.get(entity).is_err() {
                        commands
                            .entity(entity)
                            .insert(ChunkMesher(mesh_chunk(chunk_data.0.clone())));
                    }
                }
                None => {
                    let entity = commands
                        .spawn((
                            Transform::from_translation(chunk_pos_to_global_pos(*chunk_pos)),
                            ChunkConstructor(generate_chunk(*chunk_pos)),
                        ))
                        .id();

                    chunks.insert(*chunk_pos, entity);
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

pub fn global_pos_to_chunk_pos(global: Vec3) -> IVec3 {
    (global / chunk::LENGTH).floor().as_ivec3()
}

pub fn chunk_pos_to_global_pos(chunk_pos: IVec3) -> Vec3 {
    chunk_pos.as_vec3() * chunk::LENGTH
}

fn temp_gen(position: IVec3) -> Chunk {
    Chunk::from_fn_positions(|brick_pos| {
        let mut brick = Brick::from_fn_positions(|voxel_position| {
            let global_position = position.as_i64vec3() * chunk::LENGTH_IN_VOXELS as i64
                + brick_pos.as_i64vec3() * brick::LENGTH_IN_VOXELS as i64
                + voxel_position.as_i64vec3();
            // temporary
            if global_position.y < 0 {
                Voxel { id: 1 }
            } else {
                Voxel { id: 0 }
            }
        });
        brick.attempt_collapse();
        brick
    })
}
