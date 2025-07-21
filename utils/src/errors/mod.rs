use std::fmt::Debug;
use thiserror::Error;
use crate::FullInt;


#[derive(Debug, Error)]
#[error("Value {value} is out of bounds {min}..{max}")]
pub struct  OutOfBoundsError<T>
where 
	T: FullInt,
{
	pub value: T,
	pub min: T,
	pub max: T,
}
