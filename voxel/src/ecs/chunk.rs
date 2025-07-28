use crate::data::chunk::Chunk;
use crate::mesher;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, poll_once};
use rand::random;
use std::sync::{Arc, RwLock};

#[derive(Debug, Component)]
pub struct ChunkData(pub Arc<RwLock<Chunk>>);

#[derive(Debug, Component)]
pub struct ChunkConstructor(pub Task<Chunk>);

impl ChunkConstructor {
    pub fn new<F>(constructor: F) -> Self
    where
        F: Fn() -> Chunk + Send + 'static,
    {
        let thread_pool = AsyncComputeTaskPool::get();
        let task = thread_pool.spawn(async move { constructor() });
        Self(task)
    }
}

pub fn poll_chunk_constructors(
    mut commands: Commands,
    query: Query<(Entity, &mut ChunkConstructor)>,
) {
    for (entity, mut task) in query {
        if let Some(chunk) = block_on(poll_once(&mut task.0)) {
            let chunk = Arc::new(RwLock::new(chunk));
            commands
                .entity(entity)
                .remove::<ChunkConstructor>()
                .insert(ChunkData(chunk));
        }
    }
}

#[derive(Debug, Component)]
pub struct ChunkMesher(pub Task<Mesh>);

impl ChunkMesher {
    pub fn new(chunk: Arc<RwLock<Chunk>>) -> Self {
        let thread_pool = AsyncComputeTaskPool::get();
        let task = thread_pool.spawn(async move {
            let read_guard = chunk.read().expect("lock poisoned");
            mesher::mesh(&read_guard)
        });
        Self(task)
    }
}

pub fn poll_chunk_meshers(
    mut commands: Commands,
    query: Query<(Entity, &mut ChunkMesher)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut task) in query {
        if let Some(mesh) = block_on(poll_once(&mut task.0)) {
            commands.entity(entity).remove::<ChunkMesher>().insert((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(Color::srgb_u8(random(), random(), random()))), // This material is temporary
            ));
        }
    }
}
