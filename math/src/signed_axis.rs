use glam::IVec3;

use crate::prelude::*;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignedAxis {
    PosX = 0,
    NegX = 1,
    PosY = 2,
    NegY = 3,
    PosZ = 4,
    NegZ = 5,
}

impl SignedAxis {
    #[inline]
    pub const fn as_u8(&self) -> u8 {
        (*self) as u8
    }

    #[inline]
    pub const fn as_usize(&self) -> usize {
        (*self) as usize
    }

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
        match self {
            PosX | NegX => X,
            PosY | NegY => Y,
            PosZ | NegZ => Z,
        }
    }

    #[inline]
    pub const fn is_positive(&self) -> bool {
        (self.as_u8() & 1) == 0
    }
}

pub type PerSignedAxis<T> = PerEnum<SignedAxis, T, 6>;

impl TryFrom<usize> for SignedAxis {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => PosX,
            1 => NegX,
            2 => PosY,
            3 => NegY,
            4 => PosZ,
            5 => NegZ,
            _ => return Err(()),
        })
    }
}

impl From<SignedAxis> for usize {
    fn from(value: SignedAxis) -> Self {
        value.as_usize()
    }
}
