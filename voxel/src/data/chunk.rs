use super::voxel::{self, Voxel};
use bevy::math::U8Vec3;
use ndshape::{ConstPow2Shape3u16, ConstShape};

const BITS: u8 = 5;

pub const LENGTH_IN_VOXELS: u16 = 1 << BITS;

type ChunkShape =
    ConstPow2Shape3u16<LENGTH_IN_VOXELS, LENGTH_IN_VOXELS, LENGTH_IN_VOXELS>;

const VOLUME_IN_VOXELS: usize = (LENGTH_IN_VOXELS as usize).pow(3);

pub type VoxelArray = [Voxel; VOLUME_IN_VOXELS];

/// world space
pub const LENGTH: f32 = LENGTH_IN_VOXELS as f32 * voxel::LENGTH;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Chunk {
    Uniform(Voxel),
    Mixed(VoxelArray),
}

impl Chunk {
    pub fn attempt_collapse(&mut self) -> bool {
        use Chunk::*;
        match self {
            Uniform(_) => true,
            Mixed(voxels) => {
                let base = voxels[0];
                let can_collapse = voxels.iter().skip(1).all(|v| *v == base);
                if can_collapse {
                    *self = Uniform(base);
                }
                can_collapse
            }
        }
    }
}

pub fn pos_to_index(pos: U8Vec3) -> usize {
    ChunkShape::linearize(pos.as_u16vec3().into()) as usize
}

pub fn index_to_pos(index: usize) -> U8Vec3 {
    ChunkShape::delinearize(index as u16).map(|d| d as u8).into()
}
