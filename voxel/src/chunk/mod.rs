mod data;
mod mesher;
mod task;

use ndshape::ConstPow2Shape3u32;

use crate::voxel::VOXEL_LENGTH;

pub use data::Chunk;

const BITS: u32 = 6;

pub type ChunkShape = ConstPow2Shape3u32<BITS, BITS, BITS>;
pub const CHUNK_SHAPE: ChunkShape = ChunkShape {};

pub const X_SHIFT: u32 = ChunkShape::SHIFTS[0];
pub const Y_SHIFT: u32 = ChunkShape::SHIFTS[1];
pub const Z_SHIFT: u32 = ChunkShape::SHIFTS[2];

pub const X_STRIDE: u32 = 1 << X_SHIFT;
pub const Y_STRIDE: u32 = 1 << Y_SHIFT;
pub const Z_STRIDE: u32 = 1 << Z_SHIFT;

pub const PADDED_CHUNK_LENGTH: u32 = 1 << BITS;
pub const PADDED_CHUNK_AREA: u32 = PADDED_CHUNK_LENGTH.pow(2);
pub const PADDED_CHUNK_VOLUME: u32 = PADDED_CHUNK_LENGTH.pow(3);

pub const CHUNK_LENGTH: u32 = PADDED_CHUNK_LENGTH - 2;
pub const CHUNK_AREA: u32 = CHUNK_LENGTH.pow(2);
pub const CHUNK_VOLUME: u32 = CHUNK_LENGTH.pow(3);

pub const WORLD_CHUNK_LENGTH: f32 = CHUNK_LENGTH as f32 * VOXEL_LENGTH;
