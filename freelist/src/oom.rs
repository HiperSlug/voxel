use std::num::NonZeroUsize;

use super::FreeList;

pub trait OomStrategy {
    type Output;

    fn strategy(freelist: &mut FreeList, failed_len: NonZeroUsize) -> Self::Output;
}

pub struct Noop;

impl OomStrategy for Noop {
    type Output = ();

    fn strategy(_: &mut FreeList, _: NonZeroUsize) -> Self::Output {
        ()
    }
}
