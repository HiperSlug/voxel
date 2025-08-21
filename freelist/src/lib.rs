pub mod oom;
pub mod search;
pub mod slice;

pub use slice::Slice;

use std::num::NonZeroUsize;

use crate::{
    oom::{Noop, OomStrategy},
    search::{FirstFit, SearchStrategy},
};

#[derive(Debug)]
pub struct FreeList {
    slices: Vec<Slice>,
    extent: Slice,
}

impl FreeList {
    pub fn new(extent: Slice) -> Self {
        Self {
            slices: vec![extent],
            extent,
        }
    }

    pub fn with_internal_capacity(extent: Slice, capacity: usize) -> Self {
        let mut slices = Vec::with_capacity(capacity);
        slices.push(extent);

        Self { slices, extent }
    }

    pub fn extent(&self) -> &Slice {
        &self.extent
    }

    pub unsafe fn extent_mut(&mut self) -> &mut Slice {
        &mut self.extent
    }

    pub fn slices(&self) -> &[Slice] {
        &self.slices
    }

    pub unsafe fn ranges_mut(&mut self) -> &mut [Slice] {
        &mut self.slices
    }

    pub unsafe fn dealloc(&mut self, slice: &Slice) -> Result<(), AlreadyFree> {
        let index = self
            .slices
            .binary_search_by_key(&slice.start(), Slice::start)
            .err()
            .ok_or(AlreadyFree)?;

        let (lower, upper) = self.slices.split_at_mut(index);
        let below = lower.last_mut();
        let above = upper.first_mut();

        if let Some(below) = &below {
            if slice.overlaps(below) {
                return Err(AlreadyFree);
            }
        }

        if let Some(above) = &above {
            if slice.overlaps(above) {
                return Err(AlreadyFree);
            }
        }

        match (
            below.and_then(|b| slice.adjacent_below(b).then_some(b)),
            above.and_then(|b| slice.adjacent_above(b).then_some(b)),
        ) {
            (None, None) => {
                self.slices.insert(index, *slice);
            }
            (Some(below), None) => {
                below.set_end(slice.end()).unwrap();
            }
            (None, Some(above)) => {
                above.set_start(slice.start).unwrap();
            }
            (Some(below), Some(above)) => {
                below.set_end(above.end()).unwrap();
                self.slices.remove(index);
            }
        }

        if slice.end() > self.extent.end() {
            self.extent.set_end(slice.end()).unwrap();
        }
        if slice.start < self.extent.start {
            self.extent.set_start(slice.start).unwrap();
        }

        Ok(())
    }

    pub fn alloc<O, S>(&mut self, len: NonZeroUsize) -> Result<Slice, O::Output>
    where
        O: OomStrategy,
        S: SearchStrategy,
    {
        match S::search(self.slices(), len) {
            Some(index) => Ok(self
                .extract_slice(index, len)
                .expect("Corrupt `SearchStrategy`")),
            None => Err(O::strategy(self, len)),
        }
    }

    fn extract_slice(&mut self, index: usize, len: NonZeroUsize) -> Option<Slice> {
        let slice = &mut self.slices[index];
        if slice.len() == len.get() {
            Some(self.slices.remove(index))
        } else {
            let (below_opt, above_opt) = slice.split(len.get());
            let below = below_opt?;
            let above = above_opt?;
            *slice = above;
            Some(below)
        }
    }

    pub fn alloc_infallible<S: SearchStrategy>(&mut self, len: NonZeroUsize) -> Slice {
        self.alloc::<Noop, S>(len).unwrap()
    }

    pub fn alloc_first<O: OomStrategy>(&mut self, len: NonZeroUsize) -> Result<Slice, O::Output> {
        self.alloc::<O, FirstFit>(len)
    }
}

#[derive(Debug)]
pub struct AlreadyFree;
