pub mod generator;
pub mod mesher;
pub mod space;
pub mod task;

use bevy::prelude::*;
use dashmap::DashMap;
use pad::{AREA, VOL};
use std::sync::Arc;

use mesher::*;
pub use space::*;
pub use task::*;

use crate::{block_lib::BlockLibrary, render::alloc_buffer::Allocation, voxel::Voxel};

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

#[derive(Component, Default, Clone, Deref)]
pub struct ChunkMap(pub Arc<DashMap<ChunkPos, Chunk>>);

pub struct ChunkMesh {
    allocation: Allocation<VoxelQuad>,
    offsets: VoxelQuadOffsets,
}
