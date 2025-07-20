use std::fmt::Debug;
use num_traits::PrimInt;

use crate::PrimWrapper;

pub trait BoundedInt<I>: Sized + PrimWrapper<I>
where
	I: PartialOrd + PrimInt + Debug + Clone + Copy,
{
	/// Inclusive
	const MAX: I;
	/// Inclusive
	const MIN: I;

	/// Take a inner value and create self
	/// 
	/// Requires PrimWrapper impl
	fn bounded_new_unchecked(value: I) -> Self;

	/// Creates a bounded wrapper around a base type I
	///
	/// See also try_new
	///
	/// # Panics
	/// If the value is out of bounds
	fn bounded_new(value: I) -> Self {
		assert!(
			Self::check(value),
			"{} {value:?} is out of bounds {:?}..={:?}",
			std::any::type_name::<Self>(),
			Self::MIN,
			Self::MAX
		);
		Self::bounded_new_unchecked(value)
	}

	/// Creates a bounded wrapper around a base type I
	///
	/// Doesnt PANIC instead returns an option.
	/// See also new
	fn bounded_try_new(value: I) -> Option<Self> {
		if Self::check(value) {
			Some(Self::bounded_new_unchecked(value))
		} else {
			None
		}
	}

	/// Assertion that a value is in bounds
	fn check(val: I) -> bool {
		val <= Self::MAX && val >= Self::MIN
	}
}
