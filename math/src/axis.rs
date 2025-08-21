use enum_map::Enum;
use glam::UVec3;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Enum, Deserialize, Serialize)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Axis {
    pub const fn as_uvec3(&self) -> UVec3 {
        match self {
            X => UVec3::new(1, 0, 0),
            Y => UVec3::new(0, 1, 0),
            Z => UVec3::new(0, 0, 1),
        }
    }

    #[inline]
    pub const fn as_coords(&self) -> [u32; 3] {
        match self {
            X => [1, 0, 0],
            Y => [0, 1, 0],
            Z => [0, 0, 1],
        }
    }
}
