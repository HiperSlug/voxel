/// The chunk position of a chunk relative to the origin.
/// 
/// This is similar to a IVec3 however it uses i64 instead of i32.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChunkPos {
	pub x: i64,
	pub y: i64,
	pub z: i64,
}

impl ChunkPos {
	pub fn new(x: i64, y: i64, z: i64) -> Self {
		Self {
			x,
			y,
			z,
		}
	}
}