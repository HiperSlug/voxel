use super::voxel::{self, Voxel, VoxelId};
use crate::utils::subdivide_index;
use bevy::math::{U8Vec3, Vec3};
use std::array;

const BITS: u8 = 4;

pub const LENGTH_IN_VOXELS: u8 = 1 << BITS;

pub const VOLUME_IN_VOXELS: usize = (LENGTH_IN_VOXELS as usize).pow(3);

/// world space
pub const LENGTH: f32 = voxel::LENGTH * LENGTH_IN_VOXELS as f32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Brick {
    Uniform(Voxel),
    NonUniform(Box<[Voxel; VOLUME_IN_VOXELS]>),
}

impl Brick {
    pub fn from_fn_indices<F>(function: F) -> Self
    where
        F: Fn(usize) -> VoxelId,
    {
        let voxels = Box::new(array::from_fn(|index| Voxel::from_id(function(index))));
        Self::NonUniform(voxels)
    }

    pub fn from_fn_positions<F>(function: F) -> Self
    where
        F: Fn(U8Vec3) -> VoxelId,
    {
        let voxels = Box::new(array::from_fn(|index| {
            let position = subdivide_index::<BITS>(index);
            Voxel::from_id(function(position))
        }));
        Self::NonUniform(voxels)
    }

    pub fn attempt_collapse(&mut self) {
        if let Self::NonUniform(voxels) = &self {
            let first = voxels[0];
            if voxels.iter().skip(1).all(|v| *v == first) {
                *self = Self::Uniform(first);
            }
        }
    }
}

pub fn index_to_position(index: usize) -> Vec3 {
    voxel::LENGTH * subdivide_index::<BITS>(index).as_vec3()
}
