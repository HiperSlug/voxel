use crate::data::chunk::Chunk;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, poll_once};

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
