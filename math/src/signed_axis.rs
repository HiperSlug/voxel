use enum_map::{Enum, EnumMap};
use glam::IVec3;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

pub type SignedAxisMap<T> = EnumMap<SignedAxis, T>;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Enum)]
#[derive(Deserialize, Serialize)]
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
    pub fn is_positive(&self) -> bool {
        ((*self as u8) & 1) == 0
    }
}
