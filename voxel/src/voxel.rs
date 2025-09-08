use nonmax::NonMaxU16;

pub const WORLD_VOXEL_LEN: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Voxel(pub NonMaxU16);
