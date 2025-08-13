use bevy::prelude::{Deref, DerefMut};

pub const VOXEL_LENGTH: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref, DerefMut, PartialOrd, Ord)]
pub struct Voxel {
    pub id: u16,
}

impl Voxel {
    #[inline]
    pub const fn is_sentinel(&self) -> bool {
        self.id == u16::MAX
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.id as usize
    }
}

impl Default for Voxel {
    fn default() -> Self {
        Self { id: u16::MAX }
    }
}
