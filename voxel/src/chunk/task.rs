use bevy::{
    math::IVec3,
    tasks::{AsyncComputeTaskPool, Task, TaskPool, TaskPoolBuilder},
};
use std::{
    cell::{LazyCell, RefCell},
    sync::Arc,
};

use crate::{
    block_library::{self, BlockLibrary},
    chunk::{ChunkMap, ChunkPos},
};

use super::{Chunk, Mesher, VoxelQuad, VoxelQuadOffsets};

thread_local! {
    static MESHER: RefCell<Mesher> = RefCell::new(Mesher::new());
}

thread_local! {
    static SINGLETON: RefCell<Option<Arc<T>>> = RefCell::new(None);
}

struct MeshingTasks {
    tasks: Vec<(ChunkPos, Task<(VoxelQuadOffsets)>)>,
}

impl MeshingTasks {
    pub fn task(&mut self) {
        let pool = AsyncComputeTaskPool::get();

        let task = pool.spawn(mesh_task());
    }
}

async fn mesh_task(chunk_map: &ChunkMap<Chunk>, chunk_pos: ChunkPos, block_library: &BlockLibrary) {
    MESHER.with_borrow_mut(|mesher| {
        let Some(chunk) = chunk_map.get(&chunk_pos) else {
            return;
        };

        mesher.clear();
        let (quads, offsets) = mesher.mesh(&chunk, chunk_pos, block_library);
    })
}

#[derive(Debug)]
pub struct MeshData {
    pub quads: [Vec<VoxelQuad>; 6],
}

impl Background<MeshData> {
    fn mesh(&self, chunk_map: &ChunkMap, chunk_pos: ChunkPos) {
        let tx = self.tx.clone();
        let chunk_map = chunk_map.clone();

        self.pool.spawn(move || {
            MESHER.with_borrow_mut(|mesher| {
                let Some(chunk) = chunk_map.get(&chunk_pos) else {
                    return;
                };

                mesher.clear();
                mesher.fast_mesh(&chunk.voxels, &chunk.opaque_mask, &chunk.transparent_mask);

                let mesh_data = MeshData {
                    quads: mesher.mesh.clone(),
                };

                tx.send(mesh_data);
            })
        });
    }
}
