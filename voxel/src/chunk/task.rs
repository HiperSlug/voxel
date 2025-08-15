use crossbeam::channel::{Receiver, Sender, unbounded};
use rayon::ThreadPool;
use std::{cell::RefCell, sync::Arc};

use super::{Chunk, ChunkMap, Mesher, VoxelQuad, ChunkPos};

#[derive(Debug)]
pub struct ChannelThreadPool<T> {
    pool: ThreadPool,
    pub tx: Sender<T>,
    pub rx: Receiver<T>,
}

impl<T> From<ThreadPool> for ChannelThreadPool<T> {
    fn from(pool: ThreadPool) -> Self {
        let (tx, rx) = unbounded();
        Self { pool, tx, rx }
    }
}

thread_local! {
    static MESHER: RefCell<Mesher> = RefCell::new(Mesher::new());
}

#[derive(Debug)]
pub struct MeshData {
    pub quads: [Vec<VoxelQuad>; 6],
}

impl ChannelThreadPool<MeshData> {
    fn mesh(&self, chunk_map: &Arc<ChunkMap>, chunk_pos: ChunkPos) {
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
                    quads: mesher.quads.clone(),
                };

                let _ = tx.send(mesh_data).is_err();
            })
        });
    }
}

impl ChannelThreadPool<Chunk> {
    fn generate(&self, chunk_pos: ChunkPos) {
        let tx = self.tx.clone();

        todo!();
    }
}
