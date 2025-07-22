use crate::data::Voxel;
use crate::data::chunk::VOXELS_IN_CHUNK;
use utils::PackedInts;

pub use pallet::{GetOrInsertResult, Pallet, Variant};

pub mod pallet;

pub type VoxelPallet = Pallet<Voxel>;
pub type PackedIndices = PackedInts<usize>;

enum Remap {
    Up,
    Down,
}

/// A chunk stored as a `VoxelPallet` and flat array of `PackedIndices` pointing to `VoxelPallet`.
///
/// # Efficiency
/// This is more efficient for sparse chunks but less efficient for complex chunks
pub struct PalletChunk {
    pallet: VoxelPallet,
    packed: PackedIndices,
}

impl PalletChunk {
    /// Creates an empty `PalletChunk` with a assigned `VoxelPallet`
    ///
    /// Pallets are not meant to be large, and they must always be non-zero.
    ///
    /// # Panics
    /// If `pallet.len() >= usize::MAX`
    pub fn new(pallet: Pallet<Voxel>) -> Self {
        Self {
            packed: PackedInts::new(pallet.req_bits(), VOXELS_IN_CHUNK as usize).unwrap(),
            pallet,
        }
    }

    /// Returns the Voxel stored at a certain index
    ///
    /// # Panics
    /// If `index >= VOXELS_IN_CHUNK`
    pub fn get(&self, idx: usize) -> Voxel {
        self.get_variant(idx).inner()
    }

    /// Returns the Variant<Voxel> stored at a certain index
    ///
    /// # Errors
    /// - `Err(PackedIntsError::IndexOutOfBounds)` when `index >= self.count`
    pub fn get_variant(&self, idx: usize) -> &Variant<Voxel> {
        let pallet_idx = self.packed.get(idx);
        self.pallet
            .get(pallet_idx)
            .expect("Every PackedIndex is guaranteed to point at a Variant<Voxel>")
    }

    /// Sets the Voxel stored at a certain index
    ///
    /// # Panics
    /// If `index >= VOXELS_IN_CHUNK`
    pub fn set(&mut self, idx: usize, voxel: Voxel) {
        let pallet_idx = self.packed.get(idx);

        let was_shifted = self.pallet.decrement_index(pallet_idx);

        if was_shifted {
            self.remap(pallet_idx, Remap::Down);
        }

        match self.pallet.get_or_insert(voxel) {
            GetOrInsertResult::Gotten(pallet_idx) => {
                self.packed.set(idx, pallet_idx);

                self.pallet
                    .increment_index(pallet_idx)
                    .expect("Not enough voxels in chunk to overflow count");

                if was_shifted {
                    if self.pallet.req_bits() < self.packed.bits_per() {
                        self.packed.decrement_bits_per().expect("Because we know that req_bits < bits_per, we cannot truncate signficant data unless PackedIndices has bad data");
                    }
                }
            }

            GetOrInsertResult::Inserted(pallet_idx) => {
                self.remap(pallet_idx + 1, Remap::Up);

                if self.pallet.req_bits() > self.packed.bits_per() {
                    self.packed.increment_bits_per().expect("In order to overflow bits_per in a PackedInts<usize> Pallet.len() would need to have 2^(usize::MAX) variants.");
                }

                self.packed.set(idx, pallet_idx);
            }
        }
    }

    /// Shifts every `PackedIndex` greater than or equal to `above` either up or down by `1`
    fn remap(&mut self, min_pallet_idx: usize, remap: Remap) {
        if min_pallet_idx >= self.pallet.len() {
            return;
        }

        for idx in self.packed.range() {
            let pallet_idx = self.packed.get(idx);

            if pallet_idx >= min_pallet_idx {
                let new_pallet_index = match remap {
                    Remap::Up => pallet_idx + 1,
                    Remap::Down => pallet_idx - 1,
                };
                self.packed.set(idx, new_pallet_index);
            }
        }
    }
}
