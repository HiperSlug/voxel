use std::num::NonZeroUsize;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Slice {
    pub start: usize,
    pub len: NonZeroUsize,
}

impl Slice {
    #[inline]
    pub fn new(start: usize, len: NonZeroUsize) -> Self {
        Self { start, len }
    }

    #[inline]
    pub fn new_from_zero(len: NonZeroUsize) -> Self {
        Self {
            start: 0,
            len,
        }
    }

    #[inline]
    pub fn try_new(start: usize, len: usize) -> Option<Self> {
        Some(Self {
            start,
            len: len.try_into().ok()?,
        })
    }

    #[inline]
    pub fn try_from_range(start: usize, end: usize) -> Option<Self> {
        let len = end.checked_sub(start)?.try_into().ok()?;
        Some(Self { start, len })
    }

    #[inline]
    pub fn start(&self) -> usize {
        self.start
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.start + self.len()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len.get()
    }

    #[inline]
    pub fn set_start(&mut self, to: usize) -> Result<(), ZeroLength> {
        *self = Self::try_from_range(to, self.end()).ok_or(ZeroLength)?;

        Ok(())
    }

    #[inline]
    pub fn set_end(&mut self, to: usize) -> Result<(), ZeroLength> {
        *self = Self::try_from_range(self.start, to).ok_or(ZeroLength)?;

        Ok(())
    }

    #[inline]
    pub fn split(&self, at: usize) -> (Option<Slice>, Option<Slice>) {
        assert!(at < self.len());

        let global_at = self.start() + at;

        (
            Slice::try_from_range(self.start, global_at),
            Slice::try_from_range(global_at, self.end()),
        )
    }

    #[inline]
    pub(crate) fn adjacent_below(&self, other: &Self) -> bool {
        self.start == other.end()
    }

    #[inline]
    pub(crate) fn adjacent_above(&self, other: &Self) -> bool {
        self.end() == other.start
    }

    #[inline]
    pub(crate) fn overlaps(&self, other: &Self) -> bool {
        self.end() > other.start && self.start < other.end()
    }
}

impl PartialOrd for Slice {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.start.partial_cmp(&other.start)
    }
}

impl Ord for Slice {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.cmp(&other.start)
    }
}

#[derive(Debug)]
pub struct ZeroLength;
