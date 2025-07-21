use std::fmt::{Debug, Display};
use num_traits::{Num, PrimInt};
use crate::BitLen;

/// Trait alias for generic operations on primitive integers
pub trait FullInt: PrimInt
	+ Num
	+ Copy
	+ Send
	+ Sync
	+ 'static
	+ Debug
	+ Display
	+ BitLen
{}

impl<T> FullInt for T
where 
	T: PrimInt
	+ Copy
	+ Send
	+ Sync
	+ 'static
	+ Debug
	+ Display
	+ BitLen,
{}