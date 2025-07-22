use super::{ChunkPos, VLocalPos};

/// Voxels position in global space
///
/// # Components
/// - `VLocalPos` the position in space relative to a chunk
/// - `ChunkPos` the chunks position in space relative to the origin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct VGlobalPos {
    pub chunk_pos: ChunkPos,
    pub local_pos: VLocalPos,
}

// From absolute (i64, i64, i64)

// new

// other functions
