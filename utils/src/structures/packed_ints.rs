use crate::{FullInt, PackedIntsError};
use std::ops::Range;

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
    /// The max number of bits_per.
    ///
    /// Trying to use values over this will return 'PackedIntsError::MaxedBitsPer'
    pub const MAX_BITS_PER: usize = I::BIT_LEN;

    /// Creates an empty collection
    ///
    /// # Error
    /// - 'Err(PackedIntsError::ZeroBitsPer)' when 'bits_per == 0'
    /// - 'Err(PackedIntsError::MaxedBitsPer)' when 'bits_per > Self::MAX_BITS_PER'
    pub fn new(bits_per: usize, count: usize) -> Result<Self, PackedIntsError> {
        if bits_per == 0 {
            return Err(PackedIntsError::ZeroBitsPer);
        }
        if bits_per > Self::MAX_BITS_PER {
            return Err(PackedIntsError::MaxedBitsPer(bits_per, Self::MAX_BITS_PER));
        }

        let total_bits = bits_per * count;
        let data_len = (total_bits + I::BIT_LEN - 1) / I::BIT_LEN;

        Ok(Self {
            data: vec![I::zero(); data_len].into_boxed_slice(),
            bits_per,
            count,
            max_and_mask: (I::one() << bits_per) - I::one(),
        })
    }

    /// Changes the number of bits allocated per value to the set value
    ///
    /// Truncates data
    ///
    /// # Errors
    /// - 'Err(PackedIntsError::ZeroBitsPer)' when 'new_bits_per == 0'
    /// - 'Err(PackedIntsError::MaxedBitsPer)' when 'new_bits_per > Self::MAX_BITS_PER'
    pub fn set_bits_per(&mut self, bits_per: usize) -> Result<(), PackedIntsError> {
        let mut new_self = Self::new(bits_per, self.count)?;

        for (idx, val) in self.iter().enumerate() {
            new_self.set(idx, val).expect("iter() is always in bounds");
        }

        *self = new_self;

        Ok(())
    }

    /// Alias for self.set_bits_per(self.bits_per + 1)
    ///
    /// # Errors
    /// - 'Err(PackedIntsError::MaxedBitsPer)' when 'self.bits_per + 1 > Self::MAX_BITS_PER'
    pub fn increment_bits_per(&mut self) -> Result<(), PackedIntsError> {
        self.set_bits_per(self.bits_per + 1)
    }

    /// Lowers the number of bits this storage uses by one
    ///
    /// To force truncation use 'set_bits_per()'
    ///
    /// # Errors
    /// - 'Err(PackedIntsError::ZeroBitsPer)' when 'new_bits_per == 0'
    /// - 'Err(PackedIntsError::TruncateSignificant)' when this operation would truncate a value
    pub fn decrement_bits_per(&mut self) -> Result<(), PackedIntsError> {
        let new_max = self.max_and_mask >> 1;

        for (index, val) in self.iter().enumerate() {
            if val > new_max {
                return Err(PackedIntsError::TruncateSignificant(index));
            }
        }

        self.set_bits_per(self.bits_per - 1)?;

        Ok(())
    }

    /// Returns the 'data' stored at 'index'
    ///
    /// # Errors
    /// 'Err(PackedIntsError::IndexOutOfBounds)' when 'index >= self.count'
    pub fn get(&self, index: usize) -> Result<I, PackedIntsError> {
        if index >= self.count {
            return Err(PackedIntsError::IndexOutOfBounds(index, self.count));
        }

        let global_index = index * self.bits_per;

        let block_index = global_index / I::BIT_LEN;
        let local_index = global_index % I::BIT_LEN;

        let word = self.data[block_index];

        let mut value = word >> local_index;

        let bits_to_end = I::BIT_LEN - local_index;
        if self.bits_per > bits_to_end {
            let next_word = self.data[block_index + 1];
            value = value | next_word << bits_to_end;
        }

        Ok(value & self.max_and_mask)
    }

    /// Sets the value at 'index' to 'data'
    ///
    /// # Errors
    /// 'Err(PackedIntsError::IndexOutOfBounds)' when 'index >= self.count'
    pub fn set(&mut self, index: usize, data: I) -> Result<(), PackedIntsError> {
        if index >= self.count {
            return Err(PackedIntsError::IndexOutOfBounds(index, self.count));
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

            let next_word = &mut self.data[block_index + 1];

            *next_word = (*next_word & !next_mask) | (data >> bits_to_end);
        };

        Ok(())
    }

    /// The index range from [0..count)
    pub fn range(&self) -> Range<usize> {
        0..self.count
    }

    /// Iterator over each element
    ///
    /// This iterator iterates over clones of the data due to the nature of the structure
    pub fn iter(&self) -> impl Iterator<Item = I> + '_ {
        self.range().map(|idx| self.get(idx).unwrap())
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
            self.set(idx, map(idx)).unwrap()
        }
    }

    /// Sets each element to a value
    pub fn set_all(&mut self, data: I) {
        if data == I::zero() {
            self.data.fill(I::zero());
        } else {
            for idx in self.range() {
                self.set(idx, data).unwrap()
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

        for (value, index) in packed.iter().enumerate() {
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

        PackedInts::<usize>::new(1, 0)
            .unwrap()
            .decrement_bits_per()
            .unwrap_err();

        PackedInts::<usize>::new(PackedInts::<usize>::MAX_BITS_PER, 0)
            .unwrap()
            .increment_bits_per()
            .unwrap_err();

        PackedInts::<usize>::new(0, 0).unwrap_err();

        PackedInts::<usize>::new(PackedInts::<usize>::MAX_BITS_PER + 1, 0).unwrap_err();
    }

    #[test]
    fn set_get_bits_between_boundaries() {
        let mut packed = PackedInts::<u8>::new(5, 2).unwrap();

        packed.set(1, 0b10101).unwrap();
        let result = packed.get(1).unwrap();

        assert_eq!(result, 0b10101);
    }

    #[test]
    fn zero_count() {
        let packed = PackedInts::<usize>::new(4, 0).unwrap();
        assert_eq!(packed.iter().count(), 0);

        packed.get(0).unwrap_err();
    }

    #[test]
    fn get_set_result() {
        let mut packed = packed();
        packed.get(64).unwrap_err();
        packed.set(64, 0).unwrap_err();

        packed.get(0).unwrap();
        packed.set(0, 0).unwrap();
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
        let mut packed = PackedInts::<I>::new(8, 64).unwrap();
        let mut raw = vec![I::zero(); packed.count];

        let max = packed.max();

        for index in packed.range() {
            let data = I::from(index % max.to_usize().unwrap()).unwrap();

            raw[index] = data;
            packed.set(index, data).unwrap();
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
