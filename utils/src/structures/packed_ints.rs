use crate::FullInt;
use std::ops::Range;

pub use errors::{MaxedBitsPerError, TruncateError};

pub mod errors {
    use thiserror::Error;

    /// Returned when bits_per exceeded the maximum
    #[derive(Debug, Error)]
    #[error("bits_per ({0}) cannot exceed maximum of {1}")]
    pub struct MaxedBitsPerError(pub usize, pub usize);

    /// Returned when decrementing bits would truncate a value
    #[derive(Debug, Error)]
    #[error("value {0} does not fit in the new bit width")]
    pub struct TruncateError(pub usize);
}

#[derive(Clone, Debug)]
pub struct PackedInts<I> {
    /// The heap allocated array of 'I'
    data: Box<[I]>,

    /// The number of significant bits per number
    bits_per: usize,

    /// The number of values stored in this Collection
    count: usize,

    /// Both:
    /// - The max value an 'I' of size 'bits_per' can be
    /// - The unshifted mask of length 'bits_per'
    max_and_mask: I,
}

impl<I: FullInt> PackedInts<I> {
    /// Creates an empty collection
    ///
    /// # Errors
    /// - 'Err(MaxedBitsPerError)' when 'bits_per >= I::BIT_LEN'
    pub fn new(bits_per: usize, count: usize) -> Result<Self, MaxedBitsPerError> {
        if bits_per >= I::BIT_LEN {
            return Err(MaxedBitsPerError(bits_per, I::BIT_LEN));
        }

        let total_bits = bits_per
            .checked_mul(count)
            .expect("bits_per and count too large");
        let data_len = (total_bits + I::BIT_LEN - 1) / I::BIT_LEN;

        let max_and_mask = if bits_per == 0 {
            I::zero()
        } else {
            (I::one() << bits_per) - I::one()
        };

        Ok(Self {
            data: vec![I::zero(); data_len].into_boxed_slice(),
            bits_per,
            count,
            max_and_mask,
        })
    }

    /// Changes the number of bits allocated per value to the set value
    ///
    /// Truncates data
    ///
    /// # Errors
    /// - 'Err(MaxedBitsPerError)' when 'bits_per >= I::BIT_LEN'
    pub fn set_bits_per(&mut self, bits_per: usize) -> Result<(), MaxedBitsPerError> {
        let mut new_self = Self::new(bits_per, self.count)?;

        for (idx, val) in self.iter().enumerate() {
            new_self.set(idx, val);
        }

        *self = new_self;

        Ok(())
    }

    /// Alias for self.set_bits_per(self.bits_per + 1)
    ///
    /// # Errors
    /// - 'Err(MaxedBitsPerError)' when 'bits_per >= I::BIT_LEN'
    pub fn increment_bits_per(&mut self) -> Result<(), MaxedBitsPerError> {
        self.set_bits_per(self.bits_per + 1)
    }

    /// Lowers the number of bits this storage uses by one
    ///
    /// To force truncation use 'set_bits_per()'
    ///
    /// When 'bits_per == 0' nothing happens
    ///
    /// # Errors
    /// - 'Err(TruncateError)' when this operation would truncate a value
    pub fn decrement_bits_per(&mut self) -> Result<(), TruncateError> {
        if self.bits_per == 0 {
            return Ok(());
        }

        let new_max = self.max_and_mask >> 1;

        for index in self.range() {
            let value = self.get(index);
            if value > new_max {
                return Err(TruncateError(index));
            }
        }

        self.set_bits_per(self.bits_per - 1)
            .expect("Decrementing cannot exceed maximum");
        Ok(())
    }

    /// Returns the 'data' stored at 'index'
    ///
    /// When 'self.bits_per == 0' returns 'I::zero()'
    ///
    /// # Panics
    /// - When 'index' is out of bounds
    pub fn get(&self, index: usize) -> I {
        assert!(index < self.count, "index out of bounds");

        if self.bits_per == 0 {
            return I::zero();
        }

        let global_index = index * self.bits_per;

        let block_index = global_index / I::BIT_LEN;
        let local_index = global_index % I::BIT_LEN;

        let word = self.data[block_index];

        let mut value = word >> local_index;

        let bits_to_end = I::BIT_LEN - local_index;
        if self.bits_per > bits_to_end {
            debug_assert!(block_index + 1 < self.data.len());

            let next_word = self.data[block_index + 1];
            value = value | next_word << bits_to_end;
        }

        value & self.max_and_mask
    }

    /// Sets the value at 'index' to 'data'
    ///
    /// When 'self.bits_per == 0' does nothing
    ///
    /// # Panics
    /// - When 'index' is out of bounds
    pub fn set(&mut self, index: usize, data: I) {
        assert!(index < self.count, "index out of bounds");

        if self.bits_per == 0 {
            return;
        }

        let data = data & self.max_and_mask;

        let global_index = index * self.bits_per;

        let block_index = global_index / I::BIT_LEN;
        let local_index = global_index % I::BIT_LEN;

        let word = &mut self.data[block_index];

        let mask = self.max_and_mask << local_index;

        *word = (*word & !mask) | (data << local_index);

        let bits_to_end = I::BIT_LEN - local_index;
        if self.bits_per > bits_to_end {
            let spill = self.bits_per - bits_to_end;
            let next_mask = (I::one() << spill) - I::one();

            debug_assert!(block_index + 1 < self.data.len());
            let next_word = &mut self.data[block_index + 1];

            *next_word = (*next_word & !next_mask) | (data >> bits_to_end);
        };
    }

    /// The index range from [0..count)
    pub fn range(&self) -> Range<usize> {
        0..self.count
    }

    /// Iterator over each element
    ///
    /// This iterates over cloned data due to the nature of the structure
    pub fn iter(&self) -> impl Iterator<Item = I> + '_ {
        self.range().map(|idx| self.get(idx))
    }

    /// Applies a function that maps indices to values to all indices.
    ///
    /// # Function
    /// 'Fn(index: usize) -> 'data'
    pub fn index_map<F>(&mut self, map: F)
    where
        F: Fn(usize) -> I,
    {
        for idx in self.range() {
            self.set(idx, map(idx))
        }
    }

    /// Sets each element to a value
    pub fn set_all(&mut self, data: I) {
        if data == I::zero() {
            self.data.fill(I::zero());
        } else {
            for idx in self.range() {
                self.set(idx, data)
            }
        }
    }

    /// How many bits are allocated per int
    pub fn bits_per(&self) -> usize {
        self.bits_per
    }

    /// How many ints are allocated
    pub fn count(&self) -> usize {
        self.count
    }

    /// The max value possible to store with this bits_per
    ///
    /// Also the base bitmask for a num of length bits_per
    pub fn max(&self) -> I {
        self.max_and_mask
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn packed() -> PackedInts<usize> {
        PackedInts::<usize>::new(4, 64).unwrap()
    }

    #[test]
    fn set_all() {
        let mut packed = packed();
        packed.set_all(0b1010);
        for val in packed.iter() {
            assert_eq!(val, 0b1010);
        }
        packed.set_all(0);
        for val in packed.iter() {
            assert_eq!(val, 0);
        }
    }

    #[test]
    fn index_map() {
        let mut packed = packed();

        let map = |idx| idx % 8;

        packed.index_map(map);

        for (index, value) in packed.iter().enumerate() {
            assert_eq!(value, map(index))
        }
    }

    #[test]
    fn bits_per() {
        let mut packed = packed();
        packed.set_all(0b1111);

        packed.decrement_bits_per().unwrap_err();

        packed.increment_bits_per().unwrap();

        for val in packed.iter() {
            assert_eq!(val, 0b1111);
        }

        packed.increment_bits_per().unwrap();

        assert_eq!(packed.bits_per(), 6);

        packed.set_bits_per(usize::BITS as usize - 1).unwrap();

        packed.increment_bits_per().unwrap_err();

        packed.set_bits_per(0).unwrap();

        packed.decrement_bits_per().unwrap();

        for val in packed.iter() {
            assert_eq!(val, 0);
        }

        PackedInts::<usize>::new(usize::BITS as usize, 0).unwrap_err();
    }

    #[test]
    fn set_get_bits_between_boundaries() {
        let mut packed = PackedInts::<u8>::new(5, 2).unwrap();

        packed.set(1, 0b10101);
        let result = packed.get(1);

        assert_eq!(result, 0b10101);
    }

    #[test]
    fn zero_data() {
        let packed = PackedInts::<usize>::new(4, 0).unwrap();
        assert_eq!(packed.iter().count(), 0);

        let mut packed = PackedInts::<usize>::new(0, 4).unwrap();
        assert_eq!(packed.iter().count(), 0);

        packed.set_all(1);
        for val in packed.iter() {
            assert_eq!(val, 0)
        }

        let packed = PackedInts::<usize>::new(0, 0).unwrap();
        assert_eq!(packed.iter().count(), 0);
    }

    #[test]
    #[should_panic]
    fn get_ob() {
        let packed = packed();
        packed.get(64);
    }

    #[test]
    #[should_panic]
    fn set_ob() {
        let mut packed = packed();
        packed.set(64, 0);
    }

    #[test]
    fn data_retention_all() {
        data_retention::<u8>();
        data_retention::<u16>();
        data_retention::<u32>();
        data_retention::<u64>();
        data_retention::<u128>();
        data_retention::<usize>();
    }

    fn data_retention<I: FullInt>() {
        let mut packed = PackedInts::<I>::new(4, 64).unwrap();
        let mut raw = vec![I::zero(); packed.count];

        let max = packed.max();

        for index in packed.range() {
            let data = I::from(index % max.to_usize().unwrap()).unwrap();

            raw[index] = data;
            packed.set(index, data);
        }

        for (unpacked, raw) in packed.iter().zip(raw.into_iter()) {
            assert_eq!(
                unpacked,
                raw,
                "type: {}, bits: {}, count: {}",
                std::any::type_name::<I>(),
                packed.bits_per,
                packed.count
            )
        }
    }
}
