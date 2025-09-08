pub mod mesher;
pub mod space;
pub mod task;

pub use mesher::*;
pub use space::*;
pub use task::*;

use space::pad::{AREA, VOL};

use crate::voxel::Voxel;

#[derive(Debug)]
pub struct Chunk {
    pub voxels: [Option<Voxel>; VOL],
    pub opaque_mask: [u64; AREA],
    pub transparent_mask: [u64; AREA],
}
