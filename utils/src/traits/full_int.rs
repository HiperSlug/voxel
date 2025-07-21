use std::fmt::{Debug, Display};
use num_traits::PrimInt;

/// Trait alias for generic operations on primitive integers
pub trait FullInt: PrimInt
	+ Copy
	+ Send
	+ Sync
	+ 'static
	+ Debug
	+ Display
{}

impl<T> FullInt for T
where 
	T: PrimInt
	+ Copy
	+ Send
	+ Sync
	+ 'static
	+ Debug
	+ Display,
{}