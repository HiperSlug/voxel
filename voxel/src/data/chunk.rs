use crate::data::brick::{self, Brick};
use bevy::math::U8Vec3;
use std::array;

pub const BITS: u8 = 1;

pub const LENGTH_IN_BRICKS: u8 = 1 << BITS;

pub const VOLUME_IN_BRICKS: usize = (LENGTH_IN_BRICKS as usize).pow(3);

pub const LENGTH_IN_VOXELS: u8 = LENGTH_IN_BRICKS * brick::LENGTH_IN_VOXELS;

pub const LENGTH: f32 = LENGTH_IN_BRICKS as f32 * brick::LENGTH;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Chunk {
    pub bricks: [Brick; VOLUME_IN_BRICKS],
}

impl Chunk {
    pub fn from_fn_indices<F>(function: F) -> Self
    where
        F: Fn(usize) -> Brick,
    {
        let bricks = array::from_fn(|index| function(index));
        Self { bricks }
    }

    pub fn from_fn_positions<F>(function: F) -> Self
    where
        F: Fn(U8Vec3) -> Brick,
    {
        let bricks = array::from_fn(|index| {
            let position = super::utils::subdivide_index::<BITS>(index);
            function(position)
        });
        Self { bricks }
    }
}
