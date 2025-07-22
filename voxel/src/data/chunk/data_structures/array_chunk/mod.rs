use crate::data::chunk::VOXELS_IN_CHUNK;
use crate::data::voxel::Voxel;

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
    pub fn new(inner: [Voxel; VOXELS_IN_CHUNK as usize]) -> Self {
        Self { inner }
    }

    pub fn get(&self, index: usize) -> Voxel {
        self.inner[index]
    }

    pub fn set(&mut self, index: usize, voxel: Voxel) {
        self.inner[index] = voxel;
    }
}
