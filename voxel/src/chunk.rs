use crate::data::chunk::Chunk;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, poll_once};
use rand::random;

#[derive(Debug, Component)]
pub struct ChunkFlag;

#[derive(Debug, Component)]
pub struct ChunkData(pub Chunk);

#[derive(Debug, Component)]
pub struct ChunkConstructorTask(pub Task<Chunk>);

impl ChunkConstructorTask {
    pub fn new<C>(constructor: C) -> Self
    where
        C: Fn() -> Chunk + Send + 'static,
    {
        let thread_pool = AsyncComputeTaskPool::get();
        let task = thread_pool.spawn(async move { constructor() });
        Self(task)
    }
}

pub fn poll_chunk_constructors(
    mut commands: Commands,
    query: Query<(Entity, &mut ChunkConstructorTask)>,
) {
    for (entity, mut task) in query {
        if let Some(chunk_data) = block_on(poll_once(&mut task.0)) {
            commands
                .entity(entity)
                .remove::<ChunkConstructorTask>()
                .insert(ChunkData(chunk_data));
        }
    }
}

#[derive(Debug, Component)]
pub struct ChunkMesherTask(pub Task<Mesh>);

impl ChunkMesherTask {
    pub fn new<M>(mesher: M) -> Self
    where
        M: Fn() -> Mesh + Send + 'static,
    {
        let thread_pool = AsyncComputeTaskPool::get();
        let task = thread_pool.spawn(async move { mesher() });
        Self(task)
    }
}

pub fn poll_chunk_meshers(
    mut commands: Commands,
    query: Query<(Entity, &mut ChunkMesherTask)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut task) in query {
        if let Some(mesh) = block_on(poll_once(&mut task.0)) {
            commands.entity(entity).remove::<ChunkMesherTask>().insert((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(Color::srgb_u8(random(), random(), random()))), // This material is temporary
            ));
        }
    }
}
