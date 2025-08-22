pub mod utils;

use std::ops::{Add, Range, Sub};

use utils::{adjacent_above, adjacent_below, length, overlaps, try_split_relative};

#[derive(Debug)]
pub struct FreeList<T> {
    ranges: Vec<Range<T>>,
    extent: Range<T>,
}

impl<T> FreeList<T> {
    pub fn extent(&self) -> &Range<T> {
        &self.extent
    }

    pub unsafe fn extent_mut(&mut self) -> &mut Range<T> {
        &mut self.extent
    }

    pub fn ranges(&self) -> &[Range<T>] {
        &self.ranges
    }

    pub unsafe fn ranges_mut(&mut self) -> &mut [Range<T>] {
        &mut self.ranges
    }
}

impl<T> FreeList<T>
where
    T: Copy + PartialOrd,
{
    pub fn new(extent: Range<T>) -> Self {
        debug_assert!(!extent.is_empty());

        Self {
            ranges: vec![extent.clone()],
            extent,
        }
    }

    pub fn with_internal_capacity(extent: Range<T>, capacity: usize) -> Self {
        debug_assert!(!extent.is_empty());

        let mut ranges = Vec::with_capacity(capacity);
        ranges.push(extent.clone());

        Self { ranges, extent }
    }
}

impl<T> FreeList<T>
where
    T: Copy + Ord,
{
    #[inline]
    pub unsafe fn dealloc(&mut self, range: &Range<T>) -> Result<(), AlreadyFree> {
        debug_assert!(!range.is_empty());

        let index = self
            .ranges
            .binary_search_by_key(&range.start, |r| r.start)
            .err()
            .ok_or(AlreadyFree)?;

        let (lower, upper) = self.ranges.split_at_mut(index);
        let below = lower.last_mut();
        let above = upper.first_mut();

        if let Some(below) = &below {
            if overlaps(range, below) {
                return Err(AlreadyFree);
            }
        }

        if let Some(above) = &above {
            if overlaps(range, above) {
                return Err(AlreadyFree);
            }
        }

        match (
            below.and_then(|b| adjacent_below(range, b).then_some(b)),
            above.and_then(|b| adjacent_above(range, b).then_some(b)),
        ) {
            (None, None) => {
                self.ranges.insert(index, range.clone());
            }
            (Some(below), None) => {
                below.end = range.end;
            }
            (None, Some(above)) => {
                above.start = range.start;
            }
            (Some(below), Some(above)) => {
                below.end = above.end;
                self.ranges.remove(index);
            }
        }

        if range.end > self.extent.end {
            self.extent.end = range.end;
        }
        if range.start < self.extent.start {
            self.extent.start = range.start;
        }

        Ok(())
    }
}

impl<T> FreeList<T>
where
    T: Copy + PartialOrd + Sub<Output = T> + Add<Output = T>,
{
    #[inline]
    pub fn alloc<O, E, S>(&mut self, len: T, search: S, fallback: O) -> Result<Range<T>, E>
    where
        O: FnOnce(&mut Self, T) -> E,
        S: FnOnce(&[Range<T>], T) -> Option<usize>,
    {
        match search(&self.ranges, len) {
            Some(index) => Ok(self
                .extract_range(index, len)
                .expect("corrupt [`SearchStrategy`]")),
            None => Err(fallback(self, len)),
        }
    }

    #[inline]
    fn extract_range(&mut self, index: usize, len: T) -> Option<Range<T>> {
        let range = &mut self.ranges[index];
        if length(range) == len {
            Some(self.ranges.remove(index))
        } else {
            let (below_opt, above_opt) = try_split_relative(range, len)?;
            *range = above_opt?;
            Some(below_opt?)
        }
    }

    #[inline]
    pub fn alloc_first<O, E>(&mut self, len: T, fallback: O) -> Result<Range<T>, E>
    where
        O: FnOnce(&mut Self, T) -> E,
    {
        let search = |ranges: &[Range<T>], len: T| ranges.iter().position(|r| length(r) >= len);

        self.alloc(len, search, fallback)
    }

    #[inline]
    pub fn alloc_last<O, E>(&mut self, len: T, fallback: O) -> Result<Range<T>, E>
    where
        O: FnOnce(&mut Self, T) -> E,
    {
        let search = |ranges: &[Range<T>], len: T| ranges.iter().rposition(|r| length(r) >= len);

        self.alloc(len, search, fallback)
    }

    #[inline]
    pub fn alloc_best<O, E>(&mut self, len: T, fallback: O) -> Result<Range<T>, E>
    where
        O: FnOnce(&mut Self, T) -> E,
    {
        let search = |ranges: &[Range<T>], len: T| {
            let mut best_idx = None;
            let mut best_len = None;

            for (i, s) in ranges.iter().enumerate() {
                let s_len = length(s);

                if s_len == len {
                    return Some(i);
                } else if s_len > len {
                    if let Some(best) = best_len {
                        if s_len < best {
                            best_idx = Some(i);
                            best_len = Some(s_len);
                        }
                    } else {
                        best_idx = Some(i);
                        best_len = Some(s_len)
                    }
                }
            }

            best_idx
        };

        self.alloc(len, search, fallback)
    }

    #[inline]
    pub fn alloc_worst<O, E>(&mut self, len: T, fallback: O) -> Result<Range<T>, E>
    where
        O: FnOnce(&mut Self, T) -> E,
        T: Ord,
    {
        let search = |ranges: &[Range<T>], len: T| {
            ranges
                .iter()
                .enumerate()
                .filter(|(_, s)| length(s) >= len)
                .max_by_key(|(_, s)| length(s))
                .map(|(i, _)| i)
        };

        self.alloc(len, search, fallback)
    }
}

#[derive(Debug)]
pub struct AlreadyFree;
