use bevy::math::U8Vec3;
use std::array;

use crate::data::voxel::{self, Voxel};

pub const BITS: u8 = 4;

pub const LENGTH_IN_VOXELS: u8 = 1 << BITS;

pub const VOLUME_IN_VOXELS: usize = (LENGTH_IN_VOXELS as usize).pow(3);

pub const LENGTH: f32 = voxel::LENGTH * LENGTH_IN_VOXELS as f32;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Brick {
    pub voxels: Option<Box<[Voxel; VOLUME_IN_VOXELS]>>,
}

impl Brick {
    pub fn from_fn_indices<F>(function: F) -> Self
    where
        F: Fn(usize) -> Voxel,
    {
        let voxels = Box::new(array::from_fn(|index| function(index)));
        Self {
            voxels: Some(voxels),
        }
    }

    pub fn from_fn_positions<F>(function: F) -> Self
    where
        F: Fn(U8Vec3) -> Voxel,
    {
        let voxels = Box::new(array::from_fn(|index| {
            let position = super::utils::subdivide_index::<BITS>(index);
            function(position)
        }));
        Self {
            voxels: Some(voxels),
        }
    }

    pub fn attempt_collapse(&mut self) {
        if let Some(voxels) = &self.voxels {
            if voxels.iter().all(|v| v.is_empty()) {
                self.voxels = None;
            }
        }
    }
}
