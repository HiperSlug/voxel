pub trait BitLen: Sized {
	const BIT_LEN: usize = ::core::mem::size_of::<Self>() * 8;
}

impl<T> BitLen for T
where
	T: Sized
{}
