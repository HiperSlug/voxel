use super::voxel::{self, Voxel};
use arc_swap::ArcSwap;
use bevy::math::U8Vec3;
use ndshape::{ConstPow2Shape3u32, ConstShape};

const BITS: u32 = 5;

pub const PADDED_LENGTH_IN_VOXELS: u32 = 1 << BITS;

pub type ChunkShape = ConstPow2Shape3u32<BITS, BITS, BITS>;

pub const PADDED_VOLUME_IN_VOXELS: usize = (PADDED_LENGTH_IN_VOXELS as usize).pow(3);

pub const LENGTH_IN_VOXELS: u32 = PADDED_LENGTH_IN_VOXELS - 2;

/// world space
pub const LENGTH: f32 = LENGTH_IN_VOXELS as f32 * voxel::LENGTH;

#[derive(Debug)]
pub enum RawChunk {
    Uniform(Voxel),
    Mixed(ArcSwap<[Voxel; PADDED_VOLUME_IN_VOXELS]>),
}

impl RawChunk {
    pub fn attempt_collapse(&mut self) -> bool {
        use RawChunk::*;
        match self {
            Uniform(_) => true,
            Mixed(voxels) => {
                let guard = voxels.load();
                let base = guard[0];
                let can_collapse = guard.iter().skip(1).all(|v| *v == base);
                if can_collapse {
                    *self = Uniform(base);
                }
                can_collapse
            }
        }
    }
}

pub fn pos_to_index(pos: U8Vec3) -> usize {
    ChunkShape::linearize(pos.to_array().map(|num| num as u32)) as usize
}

pub fn index_to_pos(index: usize) -> U8Vec3 {
    ChunkShape::delinearize(index as u32)
        .map(|d| d as u8)
        .into()
}
