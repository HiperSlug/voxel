use std::sync::{Arc, LazyLock};

use bevy::{
    platform::collections::HashMap,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
};

use crate::{
    chunk::{ChunkData, ChunkFlag},
    data::{
        chunk::{self, Chunk},
        voxel::Voxel,
    },
    render::texture_array::{SharedTextureMap, TextureMap},
};

use super::texture_array::{SharedTextureArrayMaterial, TextureArrayMaterial};

use block_mesh::{GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG, SignedAxis, greedy_quads};

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
                .or_insert(MeshMaterial3d(material.0.clone()));
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
    let mut buffer = GreedyQuadsBuffer::new(chunk::PADDED_VOLUME_IN_VOXELS);
    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

    greedy_quads(
        voxels,
        &chunk::Shape {},
        [0; 3],
        [chunk::PADDED_LENGTH_IN_VOXELS - 1; 3],
        &faces,
        &mut buffer,
        block_lib,
    );

    for (unoriented_group, face) in buffer.quads.groups.into_iter().zip(&faces) {
        for quad in unoriented_group {
            let index = chunk::linearize(quad.minimum.into());
            let voxel = voxels[index];
            let block = &block_lib.blocks[voxel.index()];

            use SignedAxis::*;
            let texture_name = match face.signed_axis() {
                PosX => &block.textures.pos_x,
                NegX => &block.textures.neg_x,
                PosY => &block.textures.pos_y,
                NegY => &block.textures.neg_y,
                PosZ => &block.textures.pos_z,
                NegZ => &block.textures.neg_z,
            };

            let texture_index = texture_map.get(texture_name).unwrap().clone();
        }
    }
}
