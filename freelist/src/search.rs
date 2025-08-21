use std::num::NonZeroUsize;

use super::{FreeList, Slice};

pub trait SearchStrategy {
    fn try_extract(freelist: &mut FreeList, len: NonZeroUsize) -> Option<Slice>;
}

pub struct FirstFit;

impl SearchStrategy for FirstFit {
    fn try_extract(freelist: &mut FreeList, len: NonZeroUsize) -> Option<Slice> {
        let _len = len.get();
        for i in 0..freelist.slices.len() {
            let slice = &mut freelist.slices[i];
            if slice.len() > _len {
                let start = slice.start;

                slice.start += _len;
                slice.len = (slice.len() - _len).try_into().unwrap();

                return Some(Slice { start, len });
            } else if slice.len() == _len {
                let slice = freelist.slices.remove(i);
                return Some(slice);
            }
        }
        None
    }
}

pub struct LastFit;

impl SearchStrategy for LastFit {
    fn try_extract(freelist: &mut FreeList, len: NonZeroUsize) -> Option<Slice> {
        let _len = len.get();
        for i in (0..freelist.slices.len()).rev() {
            let slice = &mut freelist.slices[i];
            if slice.len() > _len {
                let start = slice.start;

                slice.start += _len;
                slice.len = (slice.len() - _len).try_into().unwrap();

                return Some(Slice { start, len });
            } else if slice.len() == _len {
                let slice = freelist.slices.remove(i);
                return Some(slice);
            }
        }
        None
    }
}

pub struct BestFit;

impl SearchStrategy for BestFit {
    fn try_extract(freelist: &mut FreeList, len: NonZeroUsize) -> Option<Slice> {
        let _len = len.get();

        let mut best_idx = None;

        for i in 0..freelist.slices.len() {
            let slice = &freelist.slices[i];
            let slice_len = slice.len();

            if slice_len > _len {
                if let Some(last_idx) = best_idx {
                    let last_best: &Slice = &freelist.slices[last_idx];

                    if slice_len < last_best.len() {
                        best_idx = Some(i);
                    }
                } else {
                    best_idx = Some(i);
                }
            } else if slice.len() == _len {
                let slice = freelist.slices.remove(i);
                return Some(slice);
            }
        }

        best_idx.map(|min_idx| {
            let slice = &mut freelist.slices[min_idx];

            let start = slice.start;

            slice.start += _len;
            slice.len = (slice.len() - _len).try_into().unwrap();

            Slice { start, len }
        })
    }
}

pub struct WorstFit;

impl SearchStrategy for WorstFit {
    fn try_extract(freelist: &mut FreeList, len: NonZeroUsize) -> Option<Slice> {
        let _len = len.get();

        let mut worst_idx = None;

        for i in 0..freelist.slices.len() {
            let slice = &freelist.slices[i];
            let slice_len = slice.len();

            if slice_len >= _len {
                if let Some(last_idx) = worst_idx {
                    let last_worst: &Slice = &freelist.slices[last_idx];

                    if slice_len > last_worst.len() {
                        worst_idx = Some(i);
                    }
                } else {
                    worst_idx = Some(i);
                }
            }
        }

        worst_idx.map(|min_idx| {
            let slice = &mut freelist.slices[min_idx];

            let start = slice.start;

            slice.start += _len;
            slice.len = (slice.len() - _len).try_into().unwrap();

            Slice { start, len }
        })
    }
}