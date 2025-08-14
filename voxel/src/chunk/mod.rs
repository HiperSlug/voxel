pub mod data;
pub mod mesher;
pub mod task;

use ndshape::ConstPow2Shape3u32;

use crate::voxel::VOXEL_LENGTH;

pub use data::Chunk;
pub use mesher::{Mesher, VoxelQuad};

const BITS: u32 = 6;

pub type ChunkShape = ConstPow2Shape3u32<BITS, BITS, BITS>;
pub const CHUNK_SHAPE: ChunkShape = ChunkShape {};

pub const X_SHIFT: usize = ChunkShape::SHIFTS[0] as usize;
pub const Y_SHIFT: usize = ChunkShape::SHIFTS[1] as usize;
pub const Z_SHIFT: usize = ChunkShape::SHIFTS[2] as usize;

pub const X_STRIDE: usize = 1 << X_SHIFT;
pub const Y_STRIDE: usize = 1 << Y_SHIFT;
pub const Z_STRIDE: usize = 1 << Z_SHIFT;

pub const PADDED_CHUNK_LENGTH: usize = 1 << BITS;
pub const PADDED_CHUNK_AREA: usize = PADDED_CHUNK_LENGTH.pow(2);
pub const PADDED_CHUNK_VOLUME: usize = PADDED_CHUNK_LENGTH.pow(3);

pub const CHUNK_LENGTH: usize = PADDED_CHUNK_LENGTH - 2;
pub const CHUNK_AREA: usize = CHUNK_LENGTH.pow(2);
pub const CHUNK_VOLUME: usize = CHUNK_LENGTH.pow(3);

pub const WORLD_CHUNK_LENGTH: f32 = CHUNK_LENGTH as f32 * VOXEL_LENGTH;
