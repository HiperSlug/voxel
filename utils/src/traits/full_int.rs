use crate::BitLen;
use num_traits::{FromPrimitive, Num, PrimInt, ToPrimitive};
use std::fmt::{Debug, Display};

/// Trait alias for generic operations on primitive integers
pub trait FullInt:
    PrimInt
    + Num
    + Copy
    + Send
    + Sync
    + 'static
    + Debug
    + Display
    + BitLen
    + FromPrimitive
    + ToPrimitive
{
}

impl<T> FullInt for T where
    T: PrimInt
        + Num
        + Copy
        + Send
        + Sync
        + 'static
        + Debug
        + Display
        + BitLen
        + FromPrimitive
        + ToPrimitive
{
}
