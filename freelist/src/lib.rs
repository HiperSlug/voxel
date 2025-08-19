use std::{
    cell::RefCell,
    num::NonZeroUsize,
    rc::Rc,
    sync::{Arc, Mutex},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Slice {
    pub start: usize,
    pub len: NonZeroUsize,
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
        self.start + self.len()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len.get()
    }
}

#[derive(Default)]
struct InnerFreeList(Vec<Slice>);

impl InnerFreeList {
    unsafe fn free(&mut self, slice: &Slice) -> Result<(), ()> {
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
                self.0.insert(index, *slice);
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
                self.0.remove(index);
            }
        }

        Ok(())
    }

    fn allocate(&mut self, len: NonZeroUsize) -> Option<Slice> {
        let _len = len.get();
        for i in 0..self.0.len() {
            let slice = &mut self.0[i];
            if slice.len() > _len {
                let start = slice.start;

                slice.start += _len;
                slice.len = (slice.len() - _len).try_into().unwrap();

                return Some(Slice { start, len });
            } else if slice.len() == _len {
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
            len,
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
            self.freelist.borrow_mut().free(&self.slice).unwrap();
        }
    }
}

#[derive(Default)]
pub struct AsyncFreeList(Arc<Mutex<InnerFreeList>>);

impl AsyncFreeList {
    pub fn new(len: NonZeroUsize) -> Self {
        Self(Arc::new(Mutex::new(InnerFreeList(vec![Slice {
            start: 0,
            len,
        }]))))
    }

    pub fn allocate(&self, len: NonZeroUsize) -> Option<AsyncAllocation> {
        let mut guard = self.0.lock().unwrap();
        let slice = guard.allocate(len)?;

        Some(AsyncAllocation {
            slice,
            freelist: self.0.clone(),
        })
    }
}

pub struct AsyncAllocation {
    slice: Slice,
    freelist: Arc<Mutex<InnerFreeList>>,
}

impl AsyncAllocation {
    pub fn slice(&self) -> &Slice {
        &self.slice
    }
}

impl Drop for AsyncAllocation {
    fn drop(&mut self) {
        let mut guard = self.freelist.lock().unwrap();
        unsafe {
            guard.free(&self.slice).unwrap();
        }
    }
}
