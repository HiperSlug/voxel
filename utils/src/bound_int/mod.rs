use std::fmt::Debug;
use num_traits::PrimInt;
use crate::PrimWrapper;

/// A trait for PrimWrappers that restrict an integers possible values
pub trait BoundInt: Sized + PrimWrapper
where
	Self::Inner: PrimInt + Debug,
{
	/// Exclusive
	const MAX: Self::Inner;
	/// Inclusive
	const MIN: Self::Inner;

	/// The difference between MAX and MIN
	fn difference() -> Self::Inner { Self::MAX - Self::MIN }

	/// Creates a bounded wrapper around a base type 'Self::Inner'
	///
	/// # Panics
	/// If the value is out of bounds
	fn bounded_new(inner: Self::Inner) -> Self {
		assert!(
			Self::check(inner),
			"{} {inner:?} is out of bounds {:?}..={:?}",
			std::any::type_name::<Self>(),
			Self::MIN,
			Self::MAX
		);
		Self::new(inner)
	}

	/// Assertion that a value is in bounds
	fn check(val: Self::Inner) -> bool {
		val < Self::MAX && val >= Self::MIN
	}
}

/// A trait for BoundInt that includes a function to get wrapped values
pub trait WrapBoundInt: BoundInt
where
	Self::Inner: PrimInt + Debug, 
{
	type WrapInput;

	/// Creates a wrapper around a base type 'Self::Inner'
	/// 
	/// Wrapping values over MAX back to MIN
	fn wrapped_new(input: Self::WrapInput) -> Self;
}
