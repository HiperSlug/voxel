use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
};

use crate::{
    assets::{
        block_library::{BlockLibrary, SharedBlockLibrary},
        textures::{
            SharedTextureArrayMaterial, SharedTextureMap, TextureArrayMaterial, TextureMap,
        },
    },
    chunk::{ChunkData, ChunkFlag},
    data::{chunk::Chunk, voxel::Voxel},
};

#[derive(Debug, Component)]
pub struct MeshedFlag;

#[derive(Debug, Component)]
pub struct ChunkMesherTask(Task<Mesh>);

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
                .or_insert(MeshMaterial3d(material.clone()));
        }
    }
}

pub fn handle_chunk_meshing(
    mut commands: Commands,
    query: Query<(Entity, &ChunkData), (With<ChunkFlag>, Without<MeshedFlag>)>,
    block_lib: Res<SharedBlockLibrary>,
    texture_map: Res<SharedTextureMap>,
) {
    for (entity, chunk_data) in query {
        match &chunk_data.0 {
            Chunk::Uniform(_) => {
                commands.entity(entity).insert(MeshedFlag);
            }
            Chunk::Mixed(voxels) => {
                let voxels = voxels.load();
                let lib = block_lib.clone();
                let map = texture_map.clone();

                commands.entity(entity).insert(MeshedFlag);
                commands
                    .entity(entity)
                    .insert(ChunkMesherTask::new(move || mesh(&**voxels, &lib, &map)));
            }
        };
    }
}

pub fn mesh(voxels: &[Voxel], block_lib: &BlockLibrary, texture_map: &TextureMap) -> Mesh {
    todo!();
}
