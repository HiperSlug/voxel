use crate::data::chunk::VOXELS_IN_CHUNK;
use crate::data::Voxel;
use utils::PackedInts;

pub use pallet::{Pallet, Variant, GetOrInsertResult};

pub mod pallet;

/// A chunk stored as a `Pallet` and `PackedInts`.
/// The PackedInts are the smallest possible indices in the pallet.
///
/// This is more efficient data storage for sparse chunks
/// For complex chunks this is slower and less memory efficient than a raw ArrayChunk
pub struct PalletChunk {
    pallet: Pallet<Voxel>,
    packed: PackedInts<usize>,
}

impl PalletChunk {
    /// Creates an empty `PalletChunk`
    ///
    /// Pallets are not meant to be large, and they must always be non-zero.
    ///
    /// # Returns
    /// - `Err(PackedIntsError::ZeroBitsPer)` when `bits_per == 0`
    /// - `Err(PackedIntsError::MaxedBitsPer)` when `bits_per >= Self::MAX_BITS_PER`
    pub fn new(pallet: Pallet<Voxel>) -> Result<Self, PackedIntsError> {
        Ok(Self {
            packed: PackedInts::new(pallet.req_bits(), VOXELS_IN_CHUNK as usize)?,
            pallet,
        })
    }

    /// Returns the Voxel stored at a certain index
    ///
    /// # Errors
    /// - `Err(PackedIntsError::IndexOutOfBounds)` when `index >= self.count`
    pub fn get(&self, idx: usize) -> Result<Voxel, PackedIntsError> {
        self.get_variant(idx).map(|variant| variant.inner())
    }

    /// Returns the Variant<Voxel> stored at a certain index
    ///
    /// # Errors
    /// - `Err(PackedIntsError::IndexOutOfBounds)` when `index >= self.count`
    pub fn get_variant(&self, idx: usize) -> Result<&Variant<Voxel>, PackedIntsError> {
        let pallet_idx = self.packed.get(idx)?;
        Ok(self
            .pallet
            .get(pallet_idx)
            .expect("Every PackedInt should be guaranteed to point at a Pallet Variant"))
    }

    /// Sets the Voxel stored at a certain index
    ///
    /// # Errors
    /// - `Err(PackedIntsError::IndexOutOfBounds)` when `index >= self.count`
    pub fn set(&mut self, target_idx: usize, voxel: Voxel) -> Result<(), PackedIntsError> {
        let pallet_idx = self.packed.get(target_idx)?;
        let removed = self
            .pallet
            .get(pallet_idx)
            .expect("Every PackedInt should be guaranteed to point at a Pallet Variant");

        let was_shifted = self
            .pallet
            .decrement_index(
                self.pallet
                    .binary_search(removed)
                    .expect("Every PackedInt should be guaranteed to point at a Pallet Variant"),
            )
            .expect("Index cannot be out of bounds because we got it from a binary_search");

        if was_shifted {
            self.remap(pallet_idx, Remap::Down);
        }

        match self.pallet.get_or_insert(voxel) {
            GetOrInsertResult::Found(voxel_idx) => {
                self.packed
                    .set(target_idx, voxel_idx)
                    .expect("Already verified this index");

                self.pallet
                    .increment_index(voxel_idx)
                    .expect("Already verified this index");

                if was_shifted {
                    if self.pallet.req_bits() < self.packed.bits_per() {
                        // These errors can be handled but most likely point to implimentation faults
                        self.packed.decrement_bits_per().unwrap();
                    }
                }
            }

            GetOrInsertResult::Inserted(voxel_idx) => {
                self.remap(voxel_idx + 1, Remap::Up);

                if self.pallet.req_bits() > self.packed.bits_per() {
                    // These errors can be handled but most likely point to implimentation faults
                    self.packed.increment_bits_per().unwrap();
                }

                self.packed
                    .set(target_idx, voxel_idx)
                    .expect("Already verified this index");
            }
        }

        Ok(())
    }

    /// Shifts all stored pallet_indexs greater than or equal to min_pallet_idx either up or down one pallet index
    fn remap(&mut self, min_pallet_idx: usize, remap: Remap) {
        for idx in self.packed.range() {
            let pallet_idx = self
                .packed
                .get(idx)
                .expect("Index in range always in bounds");

            if pallet_idx >= min_pallet_idx {
                let new_pallet_index = match remap {
                    Remap::Up => pallet_idx + 1,
                    Remap::Down => pallet_idx - 1,
                };
                self.packed
                    .set(idx, new_pallet_index)
                    .expect("Index in range always in bounds");
            }
        }
    }
}

enum Remap {
    Up,
    Down,
}
