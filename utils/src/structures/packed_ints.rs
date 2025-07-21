use std::{ops::Range, usize};
use anyhow::{ensure, Result};
use crate::{errors::OutOfBounds, FullInt, OutOfBoundsError};

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
    /// How many bits are allocated per int
    pub fn bits_per(&self) -> usize { self.bits_per }

	/// How many ints are allocated
    pub fn count(&self) -> usize { self.count }
	
	


	/// Changes the number of bits allocated per value to the set value
	/// 
	/// Truncates data
	/// 
	/// # Errors
    /// 'Err(OutOfBoundsError)' when 'bits_per == 0'
    pub fn set_bits_per(&mut self, bits_per: usize) -> Result<()> {
        let mut new_self = Self::new(bits_per, self.count)?;

        for (idx, val) in self.iter().enumerate() {
            new_self.set(idx, val).expect("iter() is always in bounds");
        }

        *self = new_self;

		Ok(())
    }

	/// Alias for self.set_bits_per(self.bits_per + 1)
    pub fn increment_bits_per(&mut self) -> Result<()> {
        self.set_bits_per(self.bits_per + 1)
    }

	/// Alias for self.set_bits_per(self.bits_per - 1)
	/// 
	/// # Errors
	/// 'Err(OutOfBoundsError)' when 'self.bits_per - 1 == 0'
	/// 'Err()' when any stored value uses the last bit
    pub fn decrement_bits_per(&mut self) -> Result<()> {
        let new_max = self.max_and_mask >> 1;
        for val in self.iter() {
            ensure!(val <= new_max);
        }

        self.set_bits_per(self.bits_per - 1)?;

		Ok(())
    }
	

    /// Creates an empty storage
	/// 
	/// # Error
	/// 'Err(OutOfBoundsError)' when 'bits_per == 0'
    pub fn new(bits_per: usize, count: usize) -> Result<Self> {
		ensure!(
			bits_per != 0,
			OutOfBoundsError::Under(OutOfBounds { 
				value: bits_per, 
				bound: 1,
			})
		);

        let total_bits = bits_per * count;
        let data_len = (total_bits + I::BIT_LEN - 1) / I::BIT_LEN;

        Ok(Self {
            data: vec![I::zero(); data_len].into_boxed_slice(),
            bits_per,
            count,
            max_and_mask: (I::one() << bits_per) - I::one(),
        })
    }




    /// Returns the 'data' stored at 'index'
	/// 
	/// # Errors
    /// 'Err(OutOfBoundsError)' - When index >= self.count
    pub fn get(&self, index: usize) -> Result<I> {
        let global_index = index * self.bits_per;

        let int_index = global_index / I::BIT_LEN;
        let local_index = global_index % I::BIT_LEN;

        let word = *self
			.data
			.get(int_index)
			.ok_or(OutOfBoundsError::Over(OutOfBounds { 
				value: index, 
				bound: self.count - 1, 
			}))?;

        let mut value = word >> local_index;

        let bits_to_end = I::BIT_LEN - local_index;
        if self.bits_per > bits_to_end {
            let next_word = self.data[int_index + 1];
            value = value | next_word << bits_to_end;
        }

        Ok(value & self.max_and_mask)
    }
    
	/// Sets the value at 'index' to 'data'
	/// 
	/// # Errors
    /// 'Err(OutOfBoundsError)' - When index >= self.count
	pub fn set(&mut self, index: usize, data: I) -> Result<()>{
        let global_index = index * self.bits_per;

        let int_index = global_index / I::BIT_LEN;
        let local_index = global_index % I::BIT_LEN;

        let word = self
			.data
			.get_mut(int_index)
			.ok_or(OutOfBoundsError::Over(OutOfBounds { 
				value: index, 
				bound: self.count - 1, 
			}))?;

        let mask = self.max_and_mask << local_index;

        *word = (*word & !mask) | ((data << local_index) & mask);

        let bits_to_end = I::BIT_LEN - local_index;
        if self.bits_per > bits_to_end {
            let spill = self.bits_per - bits_to_end;
            let next_mask = (I::one() << spill) - I::one();

            let next_word = &mut self.data[int_index + 1];
            let shifted = data >> bits_to_end;

            *next_word = (*next_word & !next_mask) | (shifted & next_mask);
        };

		Ok(())
    }
    



	/// The index range from 0..count
	pub fn range(&self) -> Range<usize> { 0..self.count }

	/// Iterator over each element
	/// 
	/// This iterator iterates over clones of the data due to the nature of the structure
    pub fn iter(&self) -> impl Iterator<Item = I> + '_ {
        self.range().map(|idx| self.get(idx).unwrap())
    }

	/// Sets each element to the result from the function taking the index of the value
	/// 
	/// # Closure
	/// 'Fn(index: usize) -> 'data'
	pub fn set_all_from<F>(&mut self, map: F)
	where 
		F: Fn(usize) -> I, 
	{
		for idx in self.range() {
			let data = map(idx);
			self.set(idx, data).unwrap()
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
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fmt::Debug;

    const BIG_NUM: usize = 2usize.pow(15);

	#[test]
	fn set_all_nonzero() {
		let mut packed = PackedInts::<u8>::new(3, 100).unwrap();
		packed.set_all(0b101);
		for val in packed.iter() {
			assert_eq!(val, 0b101);
		}
	}

	#[test]
	fn set_all_from() {
		let mut packed = PackedInts::<u8>::new(3, 100).unwrap();
		
		let closure = |idx| (idx % 8) as u8;
		
		let mut raw = Vec::new();
		for i in packed.range() {
			raw.push(closure(i))
		}

		packed.set_all_from(closure);

		for (packed, raw) in packed.iter().zip(raw.into_iter()) {
			assert_eq!(packed, raw, "packed: {}, raw {}", packed, raw);
		}
	}

	#[test]
	fn increment_bits_per_preserves_data() {
		let mut packed = PackedInts::<u8>::new(4, 100).unwrap();
		packed.set_all(0b1111);
		packed.increment_bits_per().unwrap();
		for val in packed.iter() {
			assert_eq!(val, 0b1111);
		}
	}

    #[test]
    fn bit_spill() {
        let mut packed = PackedInts::<u8>::new(5, 2).unwrap();
        let raw = 0b10101;
        packed.set(1, raw).unwrap();
        let result = packed.get(1).unwrap();
        assert_eq!(result, raw);
    }

    #[test]
    fn decrease_to_zero() {
        let mut packed = PackedInts::<usize>::new(1, 0).unwrap();
        packed.decrement_bits_per().unwrap_err();
    }

    #[test]
    fn increase_past_size() {
        let mut packed = PackedInts::<u8>::new(5, 0).unwrap();
        packed.increment_bits_per().unwrap_err();
    }

    #[test]
    fn remove_significant_data() {
        let mut packed = PackedInts::<usize>::new(5, 1).unwrap();
        packed.set(0, 0b10000).unwrap();
        packed.decrement_bits_per().unwrap();
		assert_ne!(packed.get(0).unwrap(), 0b10000);
    }

    #[test]
    fn increment_and_decrement() {
        let mut packed = PackedInts::<usize>::new(5, 0).unwrap();
        packed.increment_bits_per().unwrap();
        assert_eq!(packed.bits_per, 6);

		let mut packed = PackedInts::<usize>::new(5, 0).unwrap();
        packed.decrement_bits_per().unwrap();
        assert_eq!(packed.bits_per, 4);
    }

    #[test]
    fn zero_count() {
        let packed = PackedInts::<usize>::new(8, 0);
        assert_eq!(packed.iter().count(), 0);
    }

    #[test]
    fn bits_per_lowerbound() {
		PackedInts::<usize>::new(0, 0).unwrap_err();
        PackedInts::<usize>::new(1, 0).unwrap();
    }

    #[test]
    fn index_edges() {
		let mut packed = PackedInts::<usize>::new(8, 1).unwrap();
        packed.get(1).unwrap_err();
		packed.set(1, 0).unwrap_err();

		packed.get(0).unwrap();
		packed.set(0, 0).unwrap();
    }

    #[test]
    fn data_retention_types() {
        data_retention::<u8>();
        data_retention::<u16>();
        data_retention::<u32>();
        data_retention::<u64>();
        data_retention::<u128>();
        data_retention::<usize>();
    }

    fn data_retention<I: FullInt + From<u8> + Debug>() {
		let mut packed = PackedInts::<I>::new(4, BIG_NUM).unwrap();
        let mut raw = vec![I::zero(); packed.count];

        let max = 1usize << packed.bits_per;

        for index in 0..packed.count {
            let data = ((index % max) as u8).into();

            raw[index] = data;
            packed.set(index, data).unwrap();
        }

        packed
            .iter()
            .zip(raw.into_iter())
            .for_each(|(unpacked, raw)| {
                assert_eq!(
                    unpacked,
                    raw,
                    "type: {}, bits: {}, count: {}",
                    std::any::type_name::<I>(),
                    packed.bits_per,
                    packed.count
                )
            });
    }
}
