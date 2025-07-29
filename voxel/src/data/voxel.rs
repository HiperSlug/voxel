use block_mesh::{MergeVoxel, Voxel as VoxelVisibilityTrait, VoxelVisibility};

/// world space
pub const LENGTH: f32 = 0.5;

pub type VoxelId = u16;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Voxel(pub VoxelId);

impl VoxelVisibilityTrait for Voxel {
    fn get_visibility(&self) -> VoxelVisibility {
        if self.0 == 0 {
            VoxelVisibility::Empty
        } else {
            VoxelVisibility::Opaque
        }
    }
}

impl MergeVoxel for Voxel {
    type MergeValue = VoxelId;

    fn merge_value(&self) -> Self::MergeValue {
        self.0
    }
}
