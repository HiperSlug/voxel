use crate::data::voxel::Voxel;
use std::array;

#[derive(Default)]
pub enum Chunk {
    #[default]
    Empty,
    Filled(Voxel),
    Array(ArrayChunk),
    Pallet(PalletChunk),
}

impl Chunk {
    pub fn get(&self, pos: VLocalPos) -> Voxel {
        match self {
            Self::Empty => Voxel(0),
            Self::Filled(v) => *v,
            Self::Array(a) => a.get(pos.flat_index()),
            Self::Pallet(p) => p.get(pos.flat_index()),
        }
    }

    pub fn set(&mut self, pos: VLocalPos, voxel: Voxel) {
        let index = pos.flat_index();
        match self {
            Self::Empty => {
                let mut pallet_chunk = PalletChunk::filled(Voxel(0));
                pallet_chunk.set(index, voxel);
                *self = Chunk::Pallet(pallet_chunk);
            }
            Self::Filled(v) => {
                let mut pallet_chunk = PalletChunk::filled(v);
                pallet_chunk.set(index, voxel);
                *self = Chunk::Pallet(pallet_chunk);
            }
            Self::Array(a) => a.set(index, voxel),
            Self::Pallet(p) => {
                let do_promote = p.set(index, voxel);
                if do_promote {
                    let array = array::from_fn(|idx| p.get(idx));
                    *self = Chunk::Array(ArrayChunk::new(array));
                }
            }
        }
    }
}

pub use data_structures::{array_chunk::*, pallet_chunk::*};
pub use space::*;

/// Data related to voxel and chunk space.
pub mod space;

// A module containing all structures than can store a chunks data
pub mod data_structures;
