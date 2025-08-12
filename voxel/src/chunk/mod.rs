mod task;
mod mesher;
mod data;

use bevy::math::UVec3;
use ndshape::{ConstPow2Shape3u32, Shape};

use crate::voxel::VOXEL_LENGTH;

pub use data::Chunk;

const BITS: u32 = 6;

pub const CHUNK_LENGTH: usize = 1 << BITS;
pub const CHUNK_AREA: usize = CHUNK_LENGTH.pow(2);
pub const CHUNK_VOLUME: usize = CHUNK_LENGTH.pow(3);

pub const WORLD_CHUNK_LENGTH: f32 = CHUNK_LENGTH as f32 * VOXEL_LENGTH;

type ChunkShape = ConstPow2Shape3u32<BITS, BITS, BITS>;

pub const CHUNK_SHAPE: ChunkShape = ChunkShape {};

pub fn linearize(pos: UVec3) -> u32 {
	CHUNK_SHAPE.linearize(pos.into())
}

pub fn delinearize(index: u32) -> UVec3 {
	CHUNK_SHAPE.delinearize(index).into()
}
