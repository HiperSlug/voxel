pub mod slice;
pub mod oom;
pub mod search;

pub use slice::*;
pub use oom::*;
pub use search::*;

use std::num::NonZeroUsize;

#[derive(Default, Debug)]
pub struct FreeList {
    slices: Vec<Slice>,
    tail: usize,
}

impl FreeList {
    pub fn new(len: NonZeroUsize) -> Self {
        Self {
            slices: vec![Slice { start: 0, len }],
            tail: len.get(),
        }
    }

    pub fn with_internal_capacity(len: NonZeroUsize, capacity: usize) -> Self {
        let mut slices = Vec::with_capacity(capacity);
        slices.push(Slice { start: 0, len });

        Self {
            slices,
            tail: len.get(),
        }
    }

    pub fn tail(&self) -> usize {
        self.tail
    }

    pub unsafe fn tail_mut(&mut self) -> &mut usize {
        &mut self.tail
    }

    pub fn slices(&self) -> &Vec<Slice> {
        &self.slices
    }

    pub unsafe fn slices_mut(&mut self) -> &mut Vec<Slice> {
        &mut self.slices
    }

    pub unsafe fn dealloc(&mut self, slice: &Slice) -> Result<(), AlreadyFree> {
        let index = self
            .slices
            .binary_search_by_key(&slice.start, |s| s.start)
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
            below.and_then(|b| slice.touches_start(b).then_some(b)),
            above.and_then(|b| slice.touches_end(b).then_some(b)),
        ) {
            (None, None) => {
                self.slices.insert(index, *slice);
            }
            (Some(below), None) => {
                let len = below.len() + slice.len();
                below.len = len.try_into().unwrap();
            }
            (None, Some(above)) => {
                let len = above.len() + slice.len();
                above.len = len.try_into().unwrap();
                above.start -= slice.len();
            }
            (Some(below), Some(above)) => {
                let len = slice.len() + above.len() + below.len();
                below.len = len.try_into().unwrap();
                self.slices.remove(index);
            }
        }

        if slice.end() > self.tail {
            self.tail = slice.end()
        }

        Ok(())
    }

    pub fn alloc<Oom, Sch>(&mut self, len: NonZeroUsize) -> Result<Slice, Oom::Output>
    where
        Oom: OomStrategy,
        Sch: SearchStrategy,
    {
        Sch::try_extract(self, len).ok_or(Oom::strategy(self, len))
    }

    pub fn try_alloc(&mut self, len: NonZeroUsize) -> Option<Slice> {
        self.alloc::<DefaultOomStrategy, FirstFit>(len).ok()
    }
}

#[derive(Debug)]
pub struct AlreadyFree;
