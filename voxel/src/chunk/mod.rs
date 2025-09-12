pub mod generator;
pub mod mesher;
pub mod space;
pub mod task;

use bevy::math::UVec3;
use dashmap::DashMap;
use pad::{AREA, VOL};
use std::sync::Arc;

use mesher::*;
pub use space::*;
pub use task::*;

use crate::{
    block_library::BlockLibrary, render::buffer_allocator::BufferAllocation, voxel::Voxel,
};

#[derive(Debug)]
pub struct Chunk {
    pub voxels: [Option<Voxel>; VOL],
    pub opaque_mask: [u64; AREA],
    pub transparent_mask: [u64; AREA],
}

impl Chunk {
    pub const EMPTY: Self = Self {
        voxels: [None; VOL],
        opaque_mask: [0; AREA],
        transparent_mask: [0; AREA],
    };

    pub fn set(&mut self, pos: UVec3, voxel_opt: Option<Voxel>, block_library: &BlockLibrary) {
        let index = pad::linearize(pos);
        self.voxels[index] = voxel_opt;
        self.update_masks(pos, voxel_opt, block_library);
    }

    pub fn get(&self, pos: UVec3) -> Option<Voxel> {
        let index = pad::linearize(pos);
        self.voxels[index]
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::EMPTY
    }
}

pub type ChunkMap<T> = Arc<DashMap<ChunkPos, T>>;

pub struct ChunkMesh {
    buffer_allocation: BufferAllocation<VoxelQuad>,
    offsets: VoxelQuadOffsets,
}
