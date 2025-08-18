pub mod data;
pub mod mesher;
pub mod space;
pub mod task;

pub use data::*;
pub use mesher::*;
pub use space::*;
pub use task::*;

use dashmap::DashMap;
use ndshape::ConstPow2Shape3u32;

const BITS: u32 = 6;

pub type ChunkShape = ConstPow2Shape3u32<BITS, BITS, BITS>;
pub const CHUNK_SHAPE: ChunkShape = ChunkShape {};

pub const X_SHIFT: usize = ChunkShape::SHIFTS[0] as usize;
pub const Y_SHIFT: usize = ChunkShape::SHIFTS[1] as usize;
pub const Z_SHIFT: usize = ChunkShape::SHIFTS[2] as usize;

pub const X_STRIDE: usize = 1 << X_SHIFT;
pub const Y_STRIDE: usize = 1 << Y_SHIFT;
pub const Z_STRIDE: usize = 1 << Z_SHIFT;

pub type ChunkMap = DashMap<ChunkPos, Chunk>;
