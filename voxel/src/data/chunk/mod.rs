use crate::data::voxel::Voxel;

pub enum Chunk {
    Array(ArrayChunk),
    Pallet(PalletChunk),
}

impl Default for Chunk {
    fn default() -> Self {
        Self::Array(ArrayChunk::default())
    }
}

impl Chunk {
    pub fn get(&self, pos: VLocalPos) -> Voxel {
        let index = pos.flat_index();
        match self {
            Self::Array(a) => a.get(index),
            Self::Pallet(p) => p.get(index),
        }
    }

    pub fn set(&mut self, pos: VLocalPos, voxel: Voxel) {
        let index = pos.flat_index();
        match self {
            Self::Array(a) => a.set(index, voxel),
            Self::Pallet(p) => p.set(index, voxel),
        }
    }
}

pub use data_structures::{array_chunk::ArrayChunk, pallet_chunk::PalletChunk};
pub use space::*;

/// Data related to voxel and chunk space.
pub mod space;

// A module containing all structures than can store a chunks data
pub mod data_structures;
