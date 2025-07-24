pub mod octree;

pub mod object;

pub mod voxel;

use octree::Octree;

pub use voxel::Voxel;

pub const CHUNK_DEPTH: u8 = 4;

pub type Chunk = Octree<Voxel, CHUNK_DEPTH>;
