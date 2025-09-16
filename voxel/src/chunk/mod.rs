pub mod generator;
pub mod mesher;
pub mod space;
pub mod task;

use enum_map::enum_map;
use nonmax::NonMaxU16;

pub use mesher::*;
pub use space::*;
pub use task::*;

use space::pad::{AREA, VOL};

use crate::{alloc_buffer::Allocation, signed_axis::FaceMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Voxel(pub NonMaxU16);

impl Voxel {
    #[inline]
    pub fn is_transparent(&self) -> bool {
        // TODO
        false
    }

    #[inline]
    pub fn textures(&self) -> FaceMap<u32> {
        // TODO
        enum_map! {
            _ => 0
        }
    }
}

pub struct Chunk {
    voxel_opts: [Option<Voxel>; VOL],
    opaque_mask: [u64; AREA],
    transparent_mask: [u64; AREA],
}

impl Chunk {
    pub const EMPTY: Self = Self {
        voxel_opts: [None; VOL],
        opaque_mask: [0; AREA],
        transparent_mask: [0; AREA],
    };

    pub fn set(&mut self, vol_xyz: usize, voxel_opt: Option<Voxel>) {
        self.voxel_opts[vol_xyz] = voxel_opt;

        let (x, area_yz) = pad::vol_to_area(vol_xyz);

        let mask = !(1 << x);

        self.opaque_mask[area_yz] &= mask;
        self.transparent_mask[area_yz] &= mask;

        if let Some(voxel) = voxel_opt {
            let is_transparent = voxel.is_transparent();
            let is_opaque = !is_transparent;

            self.opaque_mask[area_yz] |= (is_opaque as u64) << x;
            self.transparent_mask[area_yz] |= (is_transparent as u64) << x;
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::EMPTY
    }
}

pub struct ChunkMesh {
    allocation: Allocation<VoxelQuad>,
    offsets: VoxelQuadOffsets,
}
