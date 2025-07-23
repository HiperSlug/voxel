/// Voxel position in global space
///
/// # Components
/// - `VoxelLocalPos` the voxels position in space relative to a chunks origin
/// - `ChunkPos` the chunks position in space relative to the objects origin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalVoxelPos {
    pub chunk_pos: ChunkPos,
    pub local_pos: LocalVoxelPos,
}

/// A voxel position relative to a chunks origin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalVoxelPos(pub u8, pub u8, pub u8);

/// The position of a chunk relative to the origin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos(pub i64, pub i64, pub i64);
