use std::ops::Range;
use num_traits::PrimInt;
use crate::BitLen;

#[derive(Clone, Debug)]
pub struct PackedInts<I> {
    /// The heap allocated array of I.
	data: Box<[I]>,
    
	/// The number of significant bits per number.
	/// 
	/// # Safety
	/// Must not be mutated internally.
	bits_per: usize,
	
	/// The number of values stored in this vec.
	/// 
	/// # Safety
	/// Must not be mutated internally.
    count: usize,

	/// Both:
	/// - The max value an I of size bits_per can be.
	/// - The unshifted mask of length bits_per.
    max_and_mask: I,
}

impl<I: PrimInt + BitLen> PackedInts<I> {
    /// Changes the number of bits allocated per value to the set value.
	/// Will truncate any lost bits.
	/// 
	/// # Safety
    /// Ensure truncate bits have no significant data.
    /// Ensure that 'bits_per < I::BIT_LEN' and 'bits_per > 0'
    pub unsafe fn set_bits_per(&mut self, bits_per: usize) {
        let mut new_self = Self::new(bits_per, self.count);

        for (idx, val) in self.iter().enumerate() {
            new_self.set(idx, val);
        }

        *self = new_self;
    }

	/// Increases bits_per for this struct by amount
	/// 
	/// # Panics
	/// When the 'new_value >= I::BIT_LEN'
    pub fn increase_bits_per(&mut self, amount: usize) {
        let bits_per = self.bits_per + amount;
        
		// Does not need to assert != 0 b/c bits_per must always be at least 1 and its being added with a usize
        assert!(
            bits_per < I::BIT_LEN,
            "bits_per {} out of bounds 1..{}",
            bits_per,
            I::BIT_LEN - 1
        );

        unsafe { self.set_bits_per(bits_per) }
    }

	/// Increases bits_per for this struct by one
    pub fn increment(&mut self) {
        self.increase_bits_per(1);
    }

	/// Decreases bits_per for this struct by amount
	/// 
	/// # Panics
	/// When 'bits_per == 0'
    pub fn decrease_bits_per(&mut self, amount: usize) {
        let bits_per = self.bits_per - amount;

        assert_ne!(bits_per, 0, "cannot decrease bits_per to zero");

        let new_max = self.max_and_mask >> amount;
        for val in self.iter() {
            assert!(val <= new_max);
        }

        unsafe { self.set_bits_per(bits_per) };
    }

	/// Decreases bits_per for this struct by one
    pub fn decrement(&mut self) {
        self.decrease_bits_per(1)
    }

	/// How many bits are allocated per int
    pub fn bits_per(&self) -> usize { self.bits_per }

	/// How many ints are allocated
    pub fn count(&self) -> usize { self.count }

    /// Creates a empty storage
	/// 
	/// # Panics
    /// When 'bits_per == 0' or 'bits_per >= I:BITS'
    /// When 'bits_per * count > usize::MAX'
    pub fn new(bits_per: usize, count: usize) -> Self {
        assert!(
            bits_per > 0 && bits_per < I::BIT_LEN,
            "bits_per {} out of bounds 1..{}",
            bits_per,
            I::BIT_LEN - 1
        );

        let total_bits = bits_per
            .checked_mul(count)
            .expect("bits_per * count overflow");
        let data_len = (total_bits + I::BIT_LEN - 1) / I::BIT_LEN;

        let max_and_mask = (I::one() << bits_per) - I::one();

        Self {
            data: vec![I::zero(); data_len].into_boxed_slice(),
            bits_per,
            count,
            max_and_mask,
        }
    }

    /// Returns the 'data' stored at 'index'
	/// 
	/// # Panics
    /// When 'index < self.count'
    pub fn get(&self, index: usize) -> I {
        assert!(
            index < self.count,
            "index {} out of bounds 0..{}",
            index,
            self.count
        );

        unsafe { self.get_unchecked(index) }
    }

    /// Returns the 'data' stored at 'index'
	/// 
	///  # Safety
    /// Ensure 'index < self.count'
    #[inline(always)]
    pub unsafe fn get_unchecked(&self, index: usize) -> I {
        let global_index = index * self.bits_per;

        let vec_index = global_index / I::BIT_LEN;
        let local_index = global_index % I::BIT_LEN;

        let word = self.data[vec_index];

        let mut value = word >> local_index;

        let bits_to_end = I::BIT_LEN - local_index;
        if self.bits_per > bits_to_end {
            let next_word = self.data[vec_index + 1];
            value = value | next_word << bits_to_end;
        }

        value & self.max_and_mask
    }

    /// Sets the value at 'index' to 'data'
	/// 
	/// # Panics
    /// When 'index < self.count'
    pub fn set(&mut self, index: usize, data: I) {
        assert!(
            index < self.count,
            "index {} out of bounds (0..{})",
            index,
            self.count
        );

        unsafe { self.set_unchecked(index, data) }
    }
    
    /// # Safety
    /// Ensure 'index < self.count'
    #[inline(always)]
    pub unsafe fn set_unchecked(&mut self, index: usize, data: I) {
        let global_index = index * self.bits_per;

        let vec_index = global_index / I::BIT_LEN;
        let local_index = global_index % I::BIT_LEN;

        let word = &mut self.data[vec_index];

        let mask = self.max_and_mask << local_index;

        *word = (*word & !mask) | ((data << local_index) & mask);

        let bits_to_end = I::BIT_LEN - local_index;
        if self.bits_per > bits_to_end {
            let spill = self.bits_per - bits_to_end;
            let next_mask = (I::one() << spill) - I::one();

            let next_word = &mut self.data[vec_index + 1];
            let shifted = data >> bits_to_end;

            *next_word = (*next_word & !next_mask) | (shifted & next_mask);
        };
    }
    
	/// Iterater over each element.
    pub fn iter(&self) -> impl Iterator<Item = I> + '_ {
		// Safety
		// self.range() is guaranteed to only contain indexes in the correct range
        self.range().map(|idx| unsafe { self.get_unchecked(idx) })
    }

	/// Sets each element to the result from the function.
	/// 
	/// # Closure
	/// F(idx: usize) -> I
	pub fn set_all_from<F>(&mut self, map: F)
	where 
		F: Fn(usize) -> I, 
	{
		for idx in self.range() {
			let data = map(idx);
			unsafe { self.set_unchecked(idx, data) }
		}
	}

	/// Sets each element to a value
	pub fn set_all(&mut self, data: I) {
		for idx in self.range() {
			unsafe { self.set_unchecked(idx, data) }
		}
	}

	/// The index range from 0..count
	/// 
	/// Iteraters over this range are guaranteed to be safe
	pub fn range(&self) -> Range<usize> { 0..self.count }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fmt::Debug;

    const BIG_NUM: usize = 2usize.pow(15);

	#[test]
	fn set_all() {
		let mut packed = PackedInts::<u8>::new(3, 100);
		packed.set_all(0b101);
		for val in packed.iter() {
			assert_eq!(val, 0b101);
		}
	}

	#[test]
	fn set_all_from() {
		let mut packed = PackedInts::<u8>::new(3, 100);
		
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
	fn increase_bits_per_preserves_data() {
		let mut packed = PackedInts::<u8>::new(4, 100);
		packed.set_all(0b1111);
		packed.increase_bits_per(2); // Now 6 bits
		for val in packed.iter() {
			assert_eq!(val, 0b1111);
		}
	}

    #[test]
    fn bit_spill() {
        let mut packed = PackedInts::<u8>::new(5, 2);
        let raw = 0b10101;
        packed.set(1, raw);
        let result = packed.get(1);
        assert_eq!(result, raw);
    }

    #[test]
    #[should_panic]
    fn decrease_to_zero() {
        let mut packed = PackedInts::<usize>::new(5, 0);
        packed.decrease_bits_per(5);
    }

    #[test]
    #[should_panic]
    fn increase_past_size() {
        let mut packed = PackedInts::<u8>::new(5, 0);
        packed.increase_bits_per(3);
    }

    #[test]
    #[should_panic]
    fn remove_significant_data() {
        let mut packed = PackedInts::<usize>::new(5, 1);
        packed.set(0, 0b10000);
        packed.decrement();
    }

    #[test]
    fn increment() {
        let mut packed = PackedInts::<usize>::new(5, 0);
        packed.increment();
        assert_eq!(packed.bits_per, 6)
    }

    #[test]
    fn decrement() {
        let mut packed = PackedInts::<usize>::new(5, 0);
        packed.decrement();
        assert_eq!(packed.bits_per, 4)
    }

    #[test]
    fn zero_count() {
        let packed = PackedInts::<usize>::new(8, 0);
        assert_eq!(packed.iter().count(), 0);
    }

    #[test]
    fn edges() {
        PackedInts::<usize>::new(usize::BIT_LEN - 1, 0);
        PackedInts::<usize>::new(1, 0);
    }

    #[test]
    #[should_panic]
    fn too_big() {
        PackedInts::<usize>::new(usize::BIT_LEN, 0);
    }

    #[test]
    #[should_panic]
    fn too_small() {
        PackedInts::<usize>::new(0, 0);
    }

    #[test]
    #[should_panic]
    fn get_ob() {
        PackedInts::<usize>::new(8, 0).get(1);
    }

    #[test]
    #[should_panic]
    fn set_ob() {
        PackedInts::<usize>::new(8, 0).set(1, 0);
    }

    #[test]
    fn data_retention_num_types() {
        data_retention(PackedInts::<u8>::new(4, BIG_NUM));
        data_retention(PackedInts::<u16>::new(4, BIG_NUM));
        data_retention(PackedInts::<u32>::new(4, BIG_NUM));
        data_retention(PackedInts::<u64>::new(4, BIG_NUM));
        data_retention(PackedInts::<u128>::new(4, BIG_NUM));
        data_retention(PackedInts::<usize>::new(4, BIG_NUM));
    }

    fn data_retention<I: PrimInt + BitLen + From<u8> + Debug>(mut packed: PackedInts<I>) {
        let mut raw = vec![I::zero(); packed.count];

        let max = 1usize << packed.bits_per;

        for index in 0..packed.count {
            let data = ((index % max) as u8).into();

            raw[index] = data;
            packed.set(index, data);
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
