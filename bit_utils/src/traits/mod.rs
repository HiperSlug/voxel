pub mod bit_len {
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
}
