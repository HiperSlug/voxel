use crate::prelude::*;

use AxisPermutation::*;

#[derive(Debug, Clone, Copy)]
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
    pub const fn sigificance_map(&self) -> [usize; 3] {
        match self {
            XYZ => [0, 1, 2],
            YZX => [2, 0, 1],
            ZXY => [1, 2, 0],
            XZY => [0, 2, 1],
            YXZ => [1, 0, 2],
            ZYX => [2, 1, 0],
        }
    }

    #[inline]
    pub const fn significance(&self, axis: Axis) -> usize {
        self.sigificance_map()[axis.as_usize()]
    }
}
