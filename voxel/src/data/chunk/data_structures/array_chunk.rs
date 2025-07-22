use crate::data::{Voxel, chunk::VOXELS_IN_CHUNK};

pub struct ArrayChunk {
    inner: [Voxel; VOXELS_IN_CHUNK as usize],
}

impl Default for ArrayChunk {
    fn default() -> Self {
        Self {
            inner: [Voxel::default(); VOXELS_IN_CHUNK as usize],
        }
    }
}

impl ArrayChunk {
    pub fn get(&self, index: usize) -> Voxel {
        self.inner[index]
    }

    pub fn set(&mut self, index: usize, voxel: Voxel) {
        self.inner[index] = voxel;
    }
}
