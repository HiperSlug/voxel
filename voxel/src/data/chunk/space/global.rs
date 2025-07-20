use super::CHUNK_LENGTH;
use super::ChunkPos;
use super::VLocalPos;
use super::LocalCoord;

use utils::PrimWrapper;
use utils::WrapBoundInt;

/// Representation of a voxels position in global space.
#[derive(Debug, Clone, Copy, Hash)]
pub enum VGlobalPos {
	Raw(VRawGlobalPos),
	Comp(VCompGlobalPos),
}

impl VGlobalPos {
	/// Changes this variable to a VRawGlobalPos and returns a ref to its inner value
	pub fn as_raw(&mut self) -> &VRawGlobalPos {
		match self {
			Self::Raw(r) => r,
			Self::Comp(c) => {
				let as_raw: VRawGlobalPos = (*c).into();
				*self = Self::Raw(as_raw.clone());
				self.as_raw()
			}
		}
	}

	/// Returns a copy of this variable as a VRawGlobalPos
	pub fn to_raw(&self) -> VRawGlobalPos {
		match self {
			Self::Raw(r) => r.clone(),
			Self::Comp(c) => (*c).into(),
		}
	}

	/// Changes this variable to a VCompGlobalPos and returns a ref to its inner value
	pub fn as_comp(&mut self) -> &VCompGlobalPos {
		match self {
			Self::Comp(c) => c,
			Self::Raw(r) => {
				let as_comp: VCompGlobalPos = (*r).into();
				*self = Self::Comp(as_comp);
				self.as_comp()
			}
		}
	}

	/// Returns a copy of this variable as a VCompGlobalPos
	pub fn to_comp(&self) -> VCompGlobalPos {
		match self {
			Self::Comp(c) => c.clone(),
			Self::Raw(r) => (*r).into(),
		}
	}
}

/// Voxel position relative to the origin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct VRawGlobalPos {
	pub x: i128,
	pub y: i128,
	pub z: i128,
}

impl VRawGlobalPos {
	pub fn new(x: i128, y: i128, z: i128) -> Self {
		Self {
			x,
			y,
			z,
		}
	}
}

impl From<VCompGlobalPos> for VRawGlobalPos {
	fn from(value: VCompGlobalPos) -> Self {
		let x = value.chunk_pos.x as i128 * CHUNK_LENGTH as i128 + value.local_pos.x.inner() as i128;
		let y = value.chunk_pos.y as i128 * CHUNK_LENGTH as i128 + value.local_pos.y.inner() as i128;
		let z = value.chunk_pos.z as i128 * CHUNK_LENGTH as i128 + value.local_pos.z.inner() as i128;

		Self::new(x, y, z)
	}
}

/// Composite Voxel position made up of a chunk position and a local voxel position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct VCompGlobalPos {
	pub chunk_pos: ChunkPos,
	pub local_pos: VLocalPos,
}		

impl VCompGlobalPos {
	pub fn new(chunk_pos: ChunkPos, local_pos: VLocalPos) -> Self {
		Self {
			chunk_pos,
			local_pos,
		}
	}
}

impl From<VRawGlobalPos> for VCompGlobalPos {
	fn from(value: VRawGlobalPos) -> Self {
		let chunk_x = (value.x / CHUNK_LENGTH as i128) as i64;
		let chunk_y = (value.y / CHUNK_LENGTH as i128) as i64;
		let chunk_z = (value.z / CHUNK_LENGTH as i128) as i64;

		let chunk_pos = ChunkPos::new(chunk_x, chunk_y, chunk_z);

		// I am using this custom fn to preserve the sign for the bottom data because the wrapped_new data should
		// handle negative numbers differently than positive numbers
		fn i128_to_i8(num: i128) -> i8 {
			let mask = (1 << (i8::BITS - 1)) - 1;
			let sign = (num >> (i128::BITS- i8::BITS)) as i8 & !mask;
			let data = (num as i8) & mask;
			sign | data
		}

		let local_x = LocalCoord::wrapped_new(i128_to_i8(value.x)); 
		let local_y = LocalCoord::wrapped_new(i128_to_i8(value.y));
		let local_z = LocalCoord::wrapped_new(i128_to_i8(value.z));

		let local_pos = VLocalPos::new(local_x, local_y, local_z);

		Self::new(chunk_pos, local_pos)
	}
}