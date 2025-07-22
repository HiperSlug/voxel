use crate::FullInt;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("value {value} is out of bounds [{lower}..{upper})")]
pub struct BoundsError<T: FullInt> {
    pub value: T,
    /// Exclusive
    pub upper: T,
    /// Inclusive
    pub lower: T,
}

impl<T: FullInt> BoundsError<T> {
    pub fn implicit(value: T) -> Self {
        Self {
            value,
            upper: T::max_value(),
            lower: T::min_value(),
        }
    }
}