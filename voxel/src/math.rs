use bevy::math::{IVec3, UVec3};

#[repr(i8)]
#[derive(Debug, Clone, Copy)]
pub enum Sign {
    Pos = 1,
    Neg = -1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Axis {
    pub const fn as_uvec3(&self) -> UVec3 {
        use Axis::*;

        match self {
            X => UVec3::new(1, 0, 0),
            Y => UVec3::new(0, 1, 0),
            Z => UVec3::new(0, 0, 1),
        }
    }

    pub const fn as_usize(&self) -> usize {
        (*self) as usize
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SignedAxis {
    PosX = 0,
    NegX = 1,
    PosY = 2,
    NegY = 3,
    PosZ = 4,
    NegZ = 5,
}

impl SignedAxis {
    pub const ALL: [SignedAxis; 6] = [
        SignedAxis::PosX,
        SignedAxis::NegX,
        SignedAxis::PosY,
        SignedAxis::NegY,
        SignedAxis::PosZ,
        SignedAxis::NegZ,
    ];

    #[inline]
    pub const fn split(&self) -> (Sign, Axis) {
        use Axis::*;
        use Sign::*;
        use SignedAxis::*;

        match self {
            PosX => (Pos, X),
            NegX => (Neg, X),
            PosY => (Pos, Y),
            NegY => (Neg, Y),
            PosZ => (Pos, Z),
            NegZ => (Neg, Z),
        }
    }

    #[inline]
    pub const fn as_ivec3(&self) -> IVec3 {
        use SignedAxis::*;

        match self {
            PosX => IVec3::new(1, 0, 0),
            NegX => IVec3::new(-1, 0, 0),
            PosY => IVec3::new(0, 1, 0),
            NegY => IVec3::new(0, -1, 0),
            PosZ => IVec3::new(0, 0, 1),
            NegZ => IVec3::new(0, 0, -1),
        }
    }

    #[inline]
    pub const fn as_coords(&self) -> [i32; 3] {
        use SignedAxis::*;

        match self {
            PosX => [1, 0, 0],
            NegX => [-1, 0, 0],
            PosY => [0, 1, 0],
            NegY => [0, -1, 0],
            PosZ => [0, 0, 1],
            NegZ => [0, 0, -1],
        }
    }

    #[inline]
    pub const fn abs(&self) -> Axis {
        use Axis::*;
        use SignedAxis::*;

        match self {
            PosX | NegX => X,
            PosY | NegY => Y,
            PosZ | NegZ => Z,
        }
    }

    #[inline]
    pub const fn axis_major_index(&self) -> usize {
        (*self) as usize
    }

    #[inline]
    pub const fn as_u8(&self) -> u8 {
        (*self) as u8
    }

    #[inline]
    pub const fn is_positive(&self) -> bool {
        (self.as_u8() & 1) == 0
    }
}

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
        use Axis::*;
        use AxisPermutation::*;

        match axis {
            X => XYZ,
            Y => YZX,
            Z => ZXY,
        }
    }

    #[inline]
    pub const fn odd(axis: Axis) -> Self {
        use Axis::*;
        use AxisPermutation::*;

        match axis {
            X => XZY,
            Y => YXZ,
            Z => ZYX,
        }
    }

    #[inline]
    pub const fn sigificance(&self, axis: Axis) -> usize {
        self.sigificance_map()[axis.as_usize()]
    }

    #[inline]
    pub const fn sigificance_map(&self) -> [usize; 3] {
        use AxisPermutation::*;

        match self {
            XYZ => [0, 1, 2],
            YZX => [2, 0, 1],
            ZXY => [1, 2, 0],
            XZY => [0, 2, 1],
            YXZ => [1, 0, 2],
            ZYX => [2, 1, 0],
        }
    }
}
