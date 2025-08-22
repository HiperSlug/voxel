use std::ops::{Add, Range, Sub};

pub fn nz_opt<T>(range: Range<T>) -> Option<Range<T>>
where
    T: PartialOrd,
{
    (!range.is_empty()).then_some(range)
}

pub fn length<T>(range: &Range<T>) -> T
where
    T: Copy + Sub<Output = T> + PartialOrd,
{
    debug_assert!(!range.is_empty());

    range.end - range.start
}

#[inline]
pub fn try_split<T>(range: &Range<T>, at: T) -> Option<(Option<Range<T>>, Option<Range<T>>)>
where
    T: PartialOrd + Copy,
{
    range
        .contains(&at)
        .then(|| (nz_opt(range.start..at), nz_opt(at..range.end)))
}

#[inline]
pub fn try_split_relative<T>(
    range: &Range<T>,
    at: T,
) -> Option<(Option<Range<T>>, Option<Range<T>>)>
where
    T: PartialOrd + Copy + Add<Output = T>,
{
    let at = range.start + at;
    try_split(range, at)
}

#[inline]
pub fn overlaps<T>(range: &Range<T>, other: &Range<T>) -> bool
where
    T: PartialOrd,
{
    range.end > other.start && range.start < other.end
}

#[inline]
pub fn adjacent_below<T>(range: &Range<T>, other: &Range<T>) -> bool
where
    T: PartialEq,
{
    range.start == other.end
}

#[inline]
pub fn adjacent_above<T>(range: &Range<T>, other: &Range<T>) -> bool
where
    T: PartialEq,
{
    range.end == other.start
}
