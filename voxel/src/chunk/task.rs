use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, poll_once};
use dashmap::DashMap;
use parking_lot::Mutex;
use std::{cell::RefCell, sync::Arc};

use crate::{
    block_library::BlockLibrary,
    chunk::{ChunkMesh, ChunkPos},
    render::buffer_allocator::BufferAllocator,
};

use super::{Chunk, Mesher, VoxelQuad};

thread_local! {
    static MESHER: RefCell<Mesher> = RefCell::new(Mesher::new());
}

struct MeshingTasks {
    tasks: Vec<(ChunkPos, Task<Option<ChunkMesh>>)>,
}

impl MeshingTasks {
    pub fn spawn_task(
        &mut self,
        chunk_map: Arc<DashMap<ChunkPos, Chunk>>,
        chunk_pos: ChunkPos,
        buffer_allocator: Arc<Mutex<BufferAllocator<VoxelQuad>>>,
        block_library: Arc<BlockLibrary>,
    ) {
        let pool = AsyncComputeTaskPool::get();

        let task = pool.spawn(async move {
            mesh_task(&chunk_map, chunk_pos, &buffer_allocator, &block_library)
        });

        self.tasks.push((chunk_pos, task));
    }

    pub fn poll(&mut self, callback: impl Fn(ChunkPos, Option<ChunkMesh>)) {
        self.tasks.retain_mut(|(chunk_pos, task)| {
            if let Some(mesh_opt) = block_on(poll_once(task)) {
                callback(*chunk_pos, mesh_opt);
                false
            } else {
                true
            }
        });
    }
}

fn mesh_task(
    chunk_map: &DashMap<ChunkPos, Chunk>,
    chunk_pos: ChunkPos,
    buffer_allocator: &Mutex<BufferAllocator<VoxelQuad>>,
    block_library: &BlockLibrary,
) -> Option<ChunkMesh> {
    MESHER.with_borrow_mut(|mesher| {
        let Some(chunk) = chunk_map.get(&chunk_pos) else {
            return None;
        };

        mesher.clear();
        let (quads, mut offsets) = mesher.mesh(&chunk, chunk_pos, block_library);
        let buffer_allocation = buffer_allocator.lock().store(quads);

        offsets.shift(buffer_allocation.offset());

        Some(ChunkMesh {
            buffer_allocation,
            offsets,
        })
    })
}
