pub use local::*;
pub use global::*;
pub use chunk::*;

/// Side length of standard chunks
pub const CHUNK_LENGTH: usize = 16;

/// Number of voxels in a standard chunk
/// 
/// Equal to CHUNK_LENGTH.pow(3)
pub const VOXELS_IN_CHUNK: usize = CHUNK_LENGTH.pow(3);

/// Global space position
pub mod global;

/// Local space position
pub mod local;

/// Chunk space position
pub mod chunk;