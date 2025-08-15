use bevy::prelude::*;

use crate::voxel::VOXEL_LENGTH;

use super::BITS;

pub const PADDED_CHUNK_LENGTH: usize = 1 << BITS;
pub const PADDED_CHUNK_AREA: usize = PADDED_CHUNK_LENGTH.pow(2);
pub const PADDED_CHUNK_VOLUME: usize = PADDED_CHUNK_LENGTH.pow(3);

pub const CHUNK_LENGTH: usize = PADDED_CHUNK_LENGTH - 2;
pub const CHUNK_AREA: usize = CHUNK_LENGTH.pow(2);
pub const CHUNK_VOLUME: usize = CHUNK_LENGTH.pow(3);

pub const WORLD_CHUNK_LENGTH: f32 = CHUNK_LENGTH as f32 * VOXEL_LENGTH;

#[derive(Debug, Deref, DerefMut, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ChunkPos(pub IVec3);

impl From<IVec3> for ChunkPos {
    fn from(value: IVec3) -> Self {
        Self(value)
    }
}

impl ChunkPos {
	#[inline]
	pub fn as_world(&self) -> Vec3 {
		self.as_vec3() * WORLD_CHUNK_LENGTH
	}

	#[inline]
	pub fn from_world(world: Vec3) -> Self {
		(world / WORLD_CHUNK_LENGTH).floor().as_ivec3().into()
	}
}
