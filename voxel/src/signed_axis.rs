use enum_map::{Enum, EnumMap};

pub use SignedAxis::*;

pub type Face = SignedAxis;
pub type FaceMap<T> = EnumMap<Face, T>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Enum)]
pub enum SignedAxis {
    PosX,
    PosY,
    PosZ,
    NegX,
    NegY,
    NegZ,
}

impl SignedAxis {
    pub const ALL: [Self; 6] = [PosX, PosY, PosZ, NegX, NegY, NegZ];
}
