#![allow(dead_code)]

use std::ops::{BitAnd, BitOr, BitOrAssign, Not, Shl, Shr, Sub};
use traits::*;

pub mod traits {
    pub trait BitSize {
        const BIT_SIZE: usize;
    }

    pub trait Zero {
        const ZERO: Self;
    }
    pub trait One {
        const ONE: Self;
    }

    macro_rules! impl_traits {
		($($t:ty), *) => {
			$(
				impl BitSize for $t {
					const BIT_SIZE: usize = ::core::mem::size_of::<Self>() * 8;
				}

				impl Zero for $t {
					const ZERO: Self = 0 as $t;
				}

				impl One for $t {
					const ONE: Self = 1 as $t;
				}
			)*
		};
	}

    impl_traits!(u8, u16, u32, u64, u128, usize);
}

#[derive(Clone, Debug)]
pub struct PackedBits<I> {
    data: Vec<I>,
    bits_per: usize,
    count: usize,
    base_mask: I,
}

impl<I> PackedBits<I>
where
    I: BitSize
        + One
        + Zero
        + Shl<usize, Output = I>
        + Shr<usize, Output = I>
        + BitAnd<Output = I>
        + BitOrAssign
        + BitOr<I, Output = I>
        + Sub<I, Output = I>
        + Not<Output = I>
        + Clone
        + Copy,
{
    pub fn bits_per(&self) -> usize {
        self.bits_per
    }

    pub fn count(&self) -> usize {
        self.count
    }

    /// # Panics
    /// When ![1..I::BIT_SIZE].contains(bits_per)
    /// When (bits_per * count > usize::MAX)
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

        let base_mask = (I::ONE << bits_per) - I::ONE;

        Self {
            data: vec![I::ZERO; data_len],
            bits_per,
            count,
            base_mask,
        }
    }

    /// # Panics
    /// When (index < self.count)
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
    /// ensure (index < self.count)
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
            value |= next_word << bits_to_end;
        }

        let mask = self.base_mask;
        value & mask
    }

    /// # Panics
    /// When (index < self.count)
    pub fn set(&mut self, index: usize, data: I) -> &mut Self {
        assert!(
            index < self.count,
            "index {} out of bounds (0..{})",
            index,
            self.count
        );

        unsafe { self.set_unchecked(index, data) }
    }

    #[inline(always)]
    pub unsafe fn set_unchecked(&mut self, index: usize, data: I) -> &mut Self {
        let global_index = index * self.bits_per;

        let vec_index = global_index / I::BIT_SIZE;
        let local_index = global_index % I::BIT_SIZE;

        let word = &mut self.data[vec_index];

        let mask = self.base_mask << local_index;

        *word = (*word & !mask) | ((data << local_index) & mask);

        let bits_to_end = I::BIT_SIZE - local_index;
        if self.bits_per > bits_to_end {
            let spill = self.bits_per - bits_to_end;
            let next_mask = (I::ONE << spill) - I::ONE;

            let next_word = &mut self.data[vec_index + 1];
            let shifted = data >> bits_to_end;

            *next_word = (*next_word & !next_mask) | (shifted & next_mask);
        }

        self
    }

    pub fn iter(&self) -> impl Iterator<Item = I> + '_ {
        (0..self.count).map(|idx| self.get(idx))
    }

    pub fn iter_unchecked(&self) -> impl Iterator<Item = I> + '_ {
        (0..self.count).map(|idx| unsafe { self.get_unchecked(idx) })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fmt::Debug;

    const BIG_NUM: usize = 2usize.pow(15);

    #[test]
    fn zero_count() {
        let packed = PackedBits::<usize>::new(8, 0);
        assert_eq!(packed.iter().count(), 0);
    }

    #[test]
    #[should_panic]
    fn too_big() {
        PackedBits::<usize>::new(usize::BIT_SIZE, 0);
    }

    #[test]
    fn almost_too_big() {
        PackedBits::<usize>::new(usize::BIT_SIZE - 1, 0);
    }

    #[test]
    #[should_panic]
    fn too_small() {
        PackedBits::<usize>::new(0, 0);
    }

    #[test]
    fn almost_too_small() {
        PackedBits::<usize>::new(1, 0);
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
    fn data_retention() {
        eq_result(PackedBits::<usize>::new(4, BIG_NUM));
        eq_result(PackedBits::<u8>::new(4, BIG_NUM));
        eq_result(PackedBits::<u64>::new(4, BIG_NUM));
        eq_result(PackedBits::<u32>::new(4, BIG_NUM));
        eq_result(PackedBits::<u16>::new(4, BIG_NUM));
        eq_result(PackedBits::<u128>::new(4, BIG_NUM));

        eq_result(PackedBits::<usize>::new(1, BIG_NUM));
        eq_result(PackedBits::<u8>::new(7, BIG_NUM));

        eq_result(PackedBits::<usize>::new(3, 1));
    }

    fn eq_result<I>(mut packed: PackedBits<I>)
    where
        I: BitSize
            + One
            + Zero
            + Shl<usize, Output = I>
            + Shr<usize, Output = I>
            + BitAnd<Output = I>
            + BitOrAssign
            + BitOr<I, Output = I>
            + Sub<I, Output = I>
            + Not<Output = I>
            + Clone
            + Copy
            + From<u8>
            + PartialEq
            + Eq
            + Debug,
    {
        let mut raw = vec![I::ZERO; packed.count];

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
