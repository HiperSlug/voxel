use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};

use super::{axis::*, sign::*};

pub use SignedAxis::*;

pub type SignedAxisMap<T> = EnumMap<SignedAxis, T>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Enum, Deserialize, Serialize)]
pub enum SignedAxis {
    PosX,
    PosY,
    PosZ,
    NegX,
    NegY,
    NegZ,
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
    pub const fn coords(&self) -> [i32; 3] {
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
    pub const fn sign(&self) -> Sign {
        match self {
            PosX | PosY | PosZ => Pos,
            NegX | NegY | NegZ => Neg,
        }
    }

    #[inline]
    pub const fn axis(&self) -> Axis {
        match self {
            PosX | NegX => X,
            PosY | NegY => Y,
            PosZ | NegZ => Z,
        }
    }

    #[inline]
    pub const fn components(&self) -> (Sign, Axis) {
        match self {
            PosX => (Pos, X),
            NegX => (Neg, X),
            PosY => (Pos, Y),
            NegY => (Neg, Y),
            PosZ => (Pos, Z),
            NegZ => (Neg, Z),
        }
    }
}
