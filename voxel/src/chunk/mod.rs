pub mod generator;
pub mod mesher;
pub mod space;
pub mod task;

use bevy::prelude::*;
use dashmap::DashMap;
use pad::{AREA, VOL};
use std::sync::Arc;

pub use mesher::*;
pub use space::*;
pub use task::*;

use derive_more::{From, Into};
use nonmax::NonMaxU16;

use crate::{block_lib::BlockLibrary, render::alloc_buffer::Allocation};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, From, Into)]
pub struct VoxelIndex(pub NonMaxU16);

pub struct Chunk {
    voxels: [Option<VoxelIndex>; VOL],
    some_mask: [u64; AREA],
    // opaque_mask: [u64; AREA], // implicitly opaque if some & !transparent
    transparent_mask: [u64; AREA],
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
