use std::{cell::RefCell, num::NonZeroUsize, rc::Rc};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Slice {
    pub start: usize,
    pub len: usize,
}

impl Slice {
    #[inline]
    fn touches_start(&self, other: &Self) -> bool {
        self.start == other.end()
    }

    #[inline]
    fn touches_end(&self, other: &Self) -> bool {
        self.end() == other.start
    }

    #[inline]
    fn overlaps(&self, other: &Self) -> bool {
        self.end() > other.start && self.start < other.end()
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.start + self.len
    }
}

#[derive(Default)]
pub struct InnerFreeList(Vec<Slice>);

impl InnerFreeList {
    pub unsafe fn free(&mut self, slice: Slice) -> Result<(), ()> {
        let index = self
            .0
            .binary_search_by_key(&slice.start, |s| s.start)
            .err()
            .ok_or(())?;

        let (lower, upper) = self.0.split_at_mut(index);
        let below = lower.last_mut();
        let above = upper.first_mut();

        if let Some(below) = &below {
            if slice.overlaps(below) {
                return Err(());
            }
        }

        if let Some(above) = &above {
            if slice.overlaps(above) {
                return Err(());
            }
        }

        match (
            below.and_then(|b| slice.touches_start(b).then_some(b)),
            above.and_then(|b| slice.touches_end(b).then_some(b)),
        ) {
            (None, None) => {
                self.0.insert(index, slice);
            }
            (Some(below), None) => {
                below.len += slice.len;
            }
            (None, Some(above)) => {
                above.len += slice.len;
                above.start -= slice.len;
            }
            (Some(below), Some(above)) => {
                below.len += slice.len + above.len;
                self.0.remove(index);
            }
        }

        Ok(())
    }

    pub fn allocate(&mut self, len: NonZeroUsize) -> Option<Slice> {
        let len = len.get();

        for i in 0..self.0.len() {
            let slice = &mut self.0[i];
            if slice.len > len {
                let start = slice.start;

                slice.start += len;
                slice.len -= len;

                return Some(Slice { start, len });
            } else if slice.len == len {
                let slice = self.0.remove(i);
                return Some(slice);
            }
        }
        None
    }
}

#[derive(Default)]
pub struct FreeList(Rc<RefCell<InnerFreeList>>);

impl FreeList {
    pub fn new(len: NonZeroUsize) -> Self {
        Self(Rc::new(RefCell::new(InnerFreeList(vec![Slice {
            start: 0,
            len: len.get(),
        }]))))
    }

    pub fn allocate(&self, len: NonZeroUsize) -> Option<Allocation> {
        let slice = self.0.borrow_mut().allocate(len)?;
        Some(Allocation {
            slice,
            freelist: self.0.clone(),
        })
    }
}

pub struct Allocation {
    slice: Slice,
    freelist: Rc<RefCell<InnerFreeList>>,
}

impl Allocation {
    pub fn slice(&self) -> &Slice {
        &self.slice
    }
}

impl Drop for Allocation {
    fn drop(&mut self) {
        unsafe {
            self.freelist.borrow_mut().free(self.slice).unwrap();
        }
    }
}
