use std::sync::Arc;
use tokio::sync::RwLock;

use crate::voxel::Voxel;

use super::PADDED_CHUNK_VOLUME;

#[derive(Debug)]
pub struct Chunk {
    pub voxels: Arc<RwLock<[Voxel; PADDED_CHUNK_VOLUME]>>,
}
