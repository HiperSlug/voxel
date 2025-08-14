use std::sync::Arc;
use bevy::prelude::*;
use tokio::sync::RwLock;

use crate::voxel::Voxel;

use super::PADDED_CHUNK_VOLUME;

#[derive(Debug, DerefMut, Deref)]
pub struct Chunk {
    pub voxels: Arc<RwLock<[Voxel; PADDED_CHUNK_VOLUME]>>,
}

impl Chunk {
    pub fn new(voxels: Arc<RwLock<[Voxel; PADDED_CHUNK_VOLUME]>>) -> Self {
        Self { voxels }
    }
}
