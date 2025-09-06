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
    pub const fn axis_map(&self) -> AxisMap<Axis> {
        match self {
            XYZ => AxisMap::from_array([X, Y, Z]),
            YZX => AxisMap::from_array([Z, X, Y]),
            ZXY => AxisMap::from_array([Y, Z, X]),
            XZY => AxisMap::from_array([X, Z, Y]),
            YXZ => AxisMap::from_array([Y, X, Z]),
            ZYX => AxisMap::from_array([Z, Y, X]),
        }
    }
}
