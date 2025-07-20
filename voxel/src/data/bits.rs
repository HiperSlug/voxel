#![allow(dead_code)]

use num_traits::PrimInt;

pub trait BitSize: Sized {
    const BIT_SIZE: usize = ::core::mem::size_of::<Self>() * 8;
}

macro_rules! impl_bit_size {
    ($($t:ty), *) => {
        $(
            impl BitSize for $t {}
        )*
    };
}

impl_bit_size!(u8, u16, u32, u64, u128, usize);

#[derive(Clone, Debug)]
pub struct PackedBits<I> {
    data: Vec<I>,
    bits_per: usize,
    // Memory safety depends on count being unchanged
    count: usize,
    max_and_mask: I,
}

impl<I: PrimInt + BitSize> PackedBits<I> {
    /// # Safety
    /// Ensure any removed bits didnt have meaningful data.
    /// Ensure that bits_per < I::BIT_SIZE and bits_per > 0
    pub unsafe fn set_bits_per(&mut self, bits_per: usize) {
        let mut new_self = Self::new(bits_per, self.count);

        for (idx, val) in self.iter().enumerate() {
            new_self.set(idx, val);
        }

        *self = new_self;
    }

    pub fn increase_bits_per(&mut self, amount: usize) {
        let bits_per = self.bits_per + amount;
        
        assert!(
            bits_per > 0 && bits_per < I::BIT_SIZE,
            "bits_per {} out of bounds 1..{}",
            bits_per,
            I::BIT_SIZE - 1
        );

        unsafe { self.set_bits_per(bits_per) }
    }

    pub fn increment(&mut self) {
        self.increase_bits_per(1);
    }

    pub fn decrease_bits_per(&mut self, amount: usize) {
        let bits_per = self.bits_per - amount;

        assert_ne!(bits_per, 0, "cannot decrease bits_per to zero");

        let new_max = self.max_and_mask >> amount;
        for val in self.iter() {
            assert!(val <= new_max);
        }

        unsafe { self.set_bits_per(bits_per) };
    }

    pub fn decrement(&mut self) {
        self.decrease_bits_per(1)
    }

    pub fn bits_per(&self) -> usize { self.bits_per }

    pub fn count(&self) -> usize { self.count }

    /// # Panics
    /// When bits_per is 0 or greater than I:BIT_SIZE
    /// When 'bits_per * count > usize::MAX'
    pub fn new(bits_per: usize, count: usize) -> Self {
        assert!(
            bits_per > 0 && bits_per < I::BIT_SIZE,
            "bits_per {} out of bounds 1..{}",
            bits_per,
            I::BIT_SIZE - 1
        );

        let total_bits = bits_per
            .checked_mul(count)
            .expect("bits_per * count overflow");
        let data_len = (total_bits + I::BIT_SIZE - 1) / I::BIT_SIZE;

        let max_and_mask = (I::one() << bits_per) - I::one();

        Self {
            data: vec![I::zero(); data_len],
            bits_per,
            count,
            max_and_mask,
        }
    }

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

    /// # Safety
    /// Ensure 'index < self.count'
    #[inline(always)]
    pub unsafe fn get_unchecked(&self, index: usize) -> I {
        let global_index = index * self.bits_per;

        let vec_index = global_index / I::BIT_SIZE;
        let local_index = global_index % I::BIT_SIZE;

        let word = self.data[vec_index];

        let mut value = word >> local_index;

        let bits_to_end = I::BIT_SIZE - local_index;
        if self.bits_per > bits_to_end {
            let next_word = self.data[vec_index + 1];
            value = value | next_word << bits_to_end;
        }

        value & self.max_and_mask
    }

    /// # Panics
    /// When 'index < self.count'
    pub fn set(&mut self, index: usize, data: I) -> &mut Self {
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
    pub unsafe fn set_unchecked(&mut self, index: usize, data: I) -> &mut Self {
        let global_index = index * self.bits_per;

        let vec_index = global_index / I::BIT_SIZE;
        let local_index = global_index % I::BIT_SIZE;

        let word = &mut self.data[vec_index];

        let mask = self.max_and_mask << local_index;

        *word = (*word & !mask) | ((data << local_index) & mask);

        let bits_to_end = I::BIT_SIZE - local_index;
        if self.bits_per > bits_to_end {
            let spill = self.bits_per - bits_to_end;
            let next_mask = (I::one() << spill) - I::one();

            let next_word = &mut self.data[vec_index + 1];
            let shifted = data >> bits_to_end;

            *next_word = (*next_word & !next_mask) | (shifted & next_mask);
        }

        self
    }
    
    pub fn iter(&self) -> impl Iterator<Item = I> + '_ {
        // This is safe because count is immutable and data is initalized with the right amount of space for count
        (0..self.count).map(|idx| unsafe { self.get_unchecked(idx) })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fmt::Debug;

    const BIG_NUM: usize = 2usize.pow(15);

    #[test]
    fn bit_spill() {
        let mut packed = PackedBits::<u8>::new(5, 2);
        let raw = 0b10101;
        packed.set(1, raw);
        let result = packed.get(1);
        assert_eq!(result, raw);
    }

    #[test]
    #[should_panic]
    fn decrease_to_zero() {
        let mut packed = PackedBits::<usize>::new(5, 0);
        packed.decrease_bits_per(5);
    }

    #[test]
    #[should_panic]
    fn increase_past_size() {
        let mut packed = PackedBits::<u8>::new(5, 0);
        packed.increase_bits_per(3);
    }

    #[test]
    #[should_panic]
    fn remove_significant_data() {
        let mut packed = PackedBits::<usize>::new(5, 1);
        packed.set(0, 0b10000);
        packed.decrement();
    }

    #[test]
    fn increment() {
        let mut packed = PackedBits::<usize>::new(5, 0);
        packed.increment();
        assert_eq!(packed.bits_per, 6)
    }

    #[test]
    fn decrement() {
        let mut packed = PackedBits::<usize>::new(5, 0);
        packed.decrement();
        assert_eq!(packed.bits_per, 4)
    }

    #[test]
    fn zero_count() {
        let packed = PackedBits::<usize>::new(8, 0);
        assert_eq!(packed.iter().count(), 0);
    }

    #[test]
    fn edges() {
        PackedBits::<usize>::new(usize::BIT_SIZE - 1, 0);
        PackedBits::<usize>::new(1, 0);
    }

    #[test]
    #[should_panic]
    fn too_big() {
        PackedBits::<usize>::new(usize::BIT_SIZE, 0);
    }

    #[test]
    #[should_panic]
    fn too_small() {
        PackedBits::<usize>::new(0, 0);
    }

    #[test]
    #[should_panic]
    fn get_ob() {
        PackedBits::<usize>::new(8, 0).get(1);
    }

    #[test]
    #[should_panic]
    fn set_ob() {
        PackedBits::<usize>::new(8, 0).set(1, 0);
    }

    #[test]
    fn data_retention_num_types() {
        data_retention(PackedBits::<u8>::new(4, BIG_NUM));
        data_retention(PackedBits::<u16>::new(4, BIG_NUM));
        data_retention(PackedBits::<u32>::new(4, BIG_NUM));
        data_retention(PackedBits::<u64>::new(4, BIG_NUM));
        data_retention(PackedBits::<u128>::new(4, BIG_NUM));
        data_retention(PackedBits::<usize>::new(4, BIG_NUM));
    }

    fn data_retention<I: PrimInt + BitSize + From<u8> + Debug>(mut packed: PackedBits<I>) {
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
