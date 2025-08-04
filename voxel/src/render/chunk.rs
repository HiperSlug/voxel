use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
};

use super::texture_array::{SharedTextureArrayMaterial, TextureArrayMaterial};

#[derive(Debug, Component)]
pub struct MeshedFlag;

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
	material: Res<SharedTextureArrayMaterial>,
) {
    for (entity, mut task) in query {
        if let Some(mesh) = block_on(poll_once(&mut task.0)) {
            commands
                .entity(entity)
                .remove::<ChunkMesherTask>()
                .insert(Mesh3d(meshes.add(mesh)))
                .entry::<MeshMaterial3d<TextureArrayMaterial>>()
                .or_insert(MeshMaterial3d(material.0.clone()));
        }
    }
}
