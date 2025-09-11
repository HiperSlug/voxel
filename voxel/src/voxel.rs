use bevy::prelude::{Deref, DerefMut};
use derive_more::{From, Into};
use nonmax::NonMaxU16;

pub const WORLD_VOXEL_LEN: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, From, Into, Deref, DerefMut)]
pub struct Voxel(pub NonMaxU16);
