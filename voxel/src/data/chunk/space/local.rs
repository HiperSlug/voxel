use super::CHUNK_LENGTH;

pub use coord::*;
pub use position::*;

/// Bounded 0..CHUNK_LENGTH coordinate structure for local chunk coordinates
pub mod coord {
	use utils::bound_int::WrapBoundIntExt;
	use utils::WrapBoundInt;
	use utils::PrimWrapper;
	use utils::BoundInt;
	use utils::prim_wrapper_default_ops;

	use super::CHUNK_LENGTH;

	/// Wrapper structure that binds a u8 between 0..CHUNK_LENGTH
	/// 
	/// # Panic
	/// Operations that create a LocalCoord that leaves the bounds panic
	#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
	pub struct LocalCoord(u8);

	impl PrimWrapper for LocalCoord {
		type Inner = u8;

		fn new(inner: Self::Inner) -> Self {
			Self(inner)
		}

		fn inner(&self) -> Self::Inner { self.0 }
	}

	prim_wrapper_default_ops!(LocalCoord, u8);

	impl BoundInt for LocalCoord {
		const MAX: Self::Inner = CHUNK_LENGTH as Self::Inner;
		const MIN: Self::Inner = 0;
	}

	impl WrapBoundInt for LocalCoord {
		type WrapInput = i8;

		fn wrapped_new(input: Self::WrapInput) -> Self {
			Self::bounded_new(Self::wrap_value(input))
		}
	}
}

/// Local space position
pub mod position {
	use super::LocalCoord;

	/// Representation of a voxels position in local chunk space relative to the -x, -y, -z corner of the chunk
	/// 
	/// Chunk space is constrained from 0..CHUNK_LENGTH
	/// 
	/// Internally reprented by bound LocalCoords
	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
	pub struct VLocalPos {
		pub x: LocalCoord,
		pub y: LocalCoord,
		pub z: LocalCoord,
	}

	impl VLocalPos {
		pub fn new(x: LocalCoord, y: LocalCoord, z: LocalCoord) -> Self {
			Self {
				x,
				y,
				z,
			}
		}
	}
}