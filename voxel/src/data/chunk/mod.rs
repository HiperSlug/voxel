use crate::data::Voxel;

use array::ArrayChunk;
use pallet::PalletChunk;

pub mod array;
pub mod pallet;

pub enum Chunk {
    Array(ArrayChunk), // matrix chunk
    Pallet(PalletChunk),
}

impl Default for Chunk {
    fn default() -> Self {
        Self::Array(ArrayChunk::default())
    }
}

// impl Chunk {
//     pub fn get(&self, pos: impl Into<LocalPos>) -> Voxel {
//         let pos: LocalPos = pos.into();
//         match self {
//             Self::Array(a) => a.get(pos),
//             Self::Pallet(p) => p.get(pos),
//         }
//     }

//     pub fn set(&mut self, pos: impl Into<LocalPos>, value: Voxel) {
//         let pos: LocalPos = pos.into();
//         match self {
//             Self::Array(a) => a.set(pos, value),
//             Self::Pallet(p) => p.set(pos, value),
//         }
//     }
// }

pub use space::*;
pub use data_structures::*;

/// Data related to voxel and chunk space.
pub mod space;

// A module containing all structures than can store a chunks data
pub mod data_structures;