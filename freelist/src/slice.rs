use std::num::NonZeroUsize;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Slice {
    pub start: usize,
    pub len: NonZeroUsize,
}

impl Slice {
    #[inline]
    pub fn touches_start(&self, other: &Self) -> bool {
        self.start == other.end()
    }

    #[inline]
    pub fn touches_end(&self, other: &Self) -> bool {
        self.end() == other.start
    }

    #[inline]
    pub fn overlaps(&self, other: &Self) -> bool {
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