pub type VoxelId = u16;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Voxel {
    pub id: VoxelId,
}

impl Voxel {
    pub fn is_solid(&self) -> bool {
        self.id != 0
    }
}
