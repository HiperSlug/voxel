/// Voxel position in global space
///
/// # Components
/// - `VoxelLocalPos` the voxels position in space relative to a chunks origin
/// - `ChunkPos` the chunks position in space relative to the objects origin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VoxelGlobalPos {
	pub chunk_pos: ChunkPos,
	pub local_pos: VoxelLocalPos,
}

/// A voxel position relative to a chunks origin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VoxelLocalPos(pub u8, pub u8, pub u8);



/// The position of a chunk relative to the origin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos(pub i64, pub i64, pub i64);