use std::fmt::Debug;
use thiserror::Error;
use crate::FullInt;


#[derive(Debug, Error)]
#[error("value {value} is out of bounds {lower}..{upper}")]
pub struct BoundsError<T: FullInt> {
	pub value: T,
	/// Exclusive
	pub upper: T,
	/// Inclusive
	pub lower: T,
}

#[derive(Debug, Error)]
pub enum PackedIntsError {
	/// bits_per was zero
	#[error("bits_per cannot be zero")]
	ZeroBitsPer,
	/// bits_per exceeded the maximum
	#[error("bits_per ({0}) cannot exceed maximum of {1}")]
	MaxedBitsPer(usize, usize),
	/// reducing bits_per would truncate an existing element
	#[error("value {0} does not fin in the new bit width")]
	TruncateSignificant(usize),
	/// index out of range [0..count)
	#[error("index out of bounds: index: {0}, max: {1}")]
	IndexOutOfBounds(usize, usize),
}