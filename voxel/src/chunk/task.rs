use bevy::{
    render::renderer::{RenderDevice, RenderQueue},
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
};
use std::cell::RefCell;

use crate::{block_lib::BlockLibrary, chunk::mesher::VoxelQuad, render::alloc_buffer::AllocBuffer};

use super::{ChunkMap, ChunkMesh, ChunkPos, Mesher};

thread_local! {
    static MESHER: RefCell<Mesher> = RefCell::new(Mesher::new());
}

struct MeshingTasks {
    tasks: Vec<(ChunkPos, Task<Option<ChunkMesh>>)>,
}

impl MeshingTasks {
    pub fn spawn_task(
        &mut self,
        chunk_map: ChunkMap,
        chunk_pos: ChunkPos,

        alloc_buffer: AllocBuffer<VoxelQuad>,

        block_library: BlockLibrary,

        queue: RenderQueue,
        device: RenderDevice,
    ) {
        let pool = AsyncComputeTaskPool::get();

        let task = pool.spawn(async move {
            MESHER.with_borrow_mut(|mesher| {
                let Some(chunk) = chunk_map.get(&chunk_pos) else {
                    return None;
                };

                mesher.clear();
                let (quads, mut offsets) = mesher.mesh(&chunk, chunk_pos, &block_library);
                let allocation = alloc_buffer.lock().store(quads, &queue, &device);

                offsets.shift(allocation.offset());

                Some(ChunkMesh {
                    allocation,
                    offsets,
                })
            })
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
