use enum_map::Enum;
use serde::{Deserialize, Serialize};

use super::axis::*;

pub use AxisPermutation::*;

#[derive(Debug, Clone, Copy, Enum, Deserialize, Serialize)]
pub enum AxisPermutation {
    XYZ,
    YZX,
    ZXY,
    XZY,
    YXZ,
    ZYX,
}

impl AxisPermutation {
    #[inline]
    pub const fn even(axis: Axis) -> Self {
        match axis {
            X => XYZ,
            Y => YZX,
            Z => ZXY,
        }
    }

    #[inline]
    pub const fn odd(axis: Axis) -> Self {
        match axis {
            X => XZY,
            Y => YXZ,
            Z => ZYX,
        }
    }

    #[inline]
    pub const fn sigificance_map(&self) -> AxisMap<usize> {
        match self {
            XYZ => AxisMap::from_array([0, 1, 2]),
            YZX => AxisMap::from_array([2, 0, 1]),
            ZXY => AxisMap::from_array([1, 2, 0]),
            XZY => AxisMap::from_array([0, 2, 1]),
            YXZ => AxisMap::from_array([1, 0, 2]),
            ZYX => AxisMap::from_array([2, 1, 0]),
        }
    }
}
