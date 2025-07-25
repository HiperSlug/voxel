use super::{octree::Octree, voxel::Voxel};

const CHUNK_DEPTH: u8 = 4;
pub const CHUNK_LENGTH: u8 = 1 << CHUNK_DEPTH;

pub struct Chunk {
    pub data: Octree<Voxel, CHUNK_DEPTH>,
}
