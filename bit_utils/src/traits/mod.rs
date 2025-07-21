pub trait BitLen: Sized {
	const BIT_LEN: usize = ::core::mem::size_of::<Self>() * 8;
}

macro_rules! prim_impl_bit_size {
	($($t:ty), *) => {
		$(
			impl BitLen for $t {}
		)*
	};
}

prim_impl_bit_size!(u8, u16, u32, u64, u128, usize);


use num_traits::PrimInt;

pub trait RequiredBits {
	fn req_bits(&self) -> u32;
}

impl<T: PrimInt + ILog2> RequiredBits for T {
	fn req_bits(&self) -> u32 {
		if *self == T::zero() { 1 } else { self.to_usize().unwrap().ilog2() as u32 + 1 }
	}
}



pub trait ILog2 {
	fn ilog2(self) -> u32;
}

macro_rules! impl_ilog2 {
	($($t:ty),*) => {
		$(impl ILog2 for $t {
			fn ilog2(self) -> u32 {
				self.ilog2()
			}
		})*
	}
}

impl_ilog2!(u8, u16, u32, u64, u128, usize);


