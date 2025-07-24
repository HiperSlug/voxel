pub mod octree;

pub mod object;

pub mod voxel;

use octree::Octree;
use voxel::Voxel;

pub const CHUNK_DEPTH: u8 = 4;
pub const CHUNK_LEN: u8 = 1 << CHUNK_DEPTH;

pub type Chunk = Octree<Voxel, CHUNK_DEPTH>;
