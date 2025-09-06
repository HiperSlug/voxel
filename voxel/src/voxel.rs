pub const VOXEL_LEN: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Voxel {
    pub id: u16,
}

impl Voxel {
    pub const SENTINEL: u16 = u16::MAX;

    #[inline]
    pub const fn new(id: u16) -> Self {
        Self { id }
    }

    #[inline]
    pub const fn is_sentinel(&self) -> bool {
        self.id == Self::SENTINEL
    }
}

impl Default for Voxel {
    fn default() -> Self {
        Self { id: Self::SENTINEL }
    }
}
