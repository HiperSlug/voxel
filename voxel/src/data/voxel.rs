pub const LENGTH: f32 = 0.5;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Voxel {
    pub id: u16,
}

impl Voxel {
    pub fn is_empty(self) -> bool {
        self.id == 0
    }
}
