use crate::voxel::Voxel;

use super::{PADDED_CHUNK_AREA, PADDED_CHUNK_VOLUME};

#[derive(Debug)]
pub struct Chunk {
    pub voxels: [Voxel; PADDED_CHUNK_VOLUME],
    pub opaque_mask: [u64; PADDED_CHUNK_AREA],
    pub transparent_mask: [u64; PADDED_CHUNK_AREA],
}

impl Chunk {
    pub fn new(voxels: [Voxel; PADDED_CHUNK_VOLUME]) -> Self {
        // TODO: replace with actual code
        Self {
            voxels,
            opaque_mask: [0; PADDED_CHUNK_AREA],
            transparent_mask: [0; PADDED_CHUNK_AREA],
        }
    }
}
