use std::fmt::Debug;
use thiserror::Error;
use crate::FullInt;


#[derive(Debug, Error)]
pub enum OutOfBoundsError<T: FullInt> {
	#[error("out of bounds -- too high -- {0}")]
	Over(OutOfBounds<T>),
	#[error("out of bounds -- too low -- {0}")]
	Under(OutOfBounds<T>),
}


#[derive(Debug, Error)]
#[error("Value {value}, escaped bound {bound}")]
pub struct OutOfBounds<T: FullInt> {
	pub value: T,
	/// INCLUSIVE
	pub bound: T,
}

