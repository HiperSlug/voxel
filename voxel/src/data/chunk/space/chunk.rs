/// The position of a chunk relative to the origin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChunkPos {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl ChunkPos {
    /// Creates a new `ChunkPos`
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }
}
