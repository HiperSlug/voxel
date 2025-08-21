use std::{num::NonZeroUsize, usize};

use crate::Slice;

pub trait SearchStrategy {
    fn search(slices: &[Slice], len: NonZeroUsize) -> Option<usize>;
}

pub struct FirstFit;

impl SearchStrategy for FirstFit {
    fn search(slices: &[Slice], len: NonZeroUsize) -> Option<usize> {
        let len = len.get();
        slices.iter().position(|s| s.len() >= len)
    }
}

pub struct LastFit;

impl SearchStrategy for LastFit {
    fn search(slices: &[Slice], len: NonZeroUsize) -> Option<usize> {
        let len = len.get();
        slices.iter().rposition(|s| s.len() >= len)
    }
}

pub struct BestFit;

impl SearchStrategy for BestFit {
    fn search(slices: &[Slice], len: NonZeroUsize) -> Option<usize> {
        let len = len.get();
        let mut best_idx = None;
        let mut best_len = usize::MAX;

        for (i, s) in slices.iter().enumerate() {
            let slice_len = s.len();

            if slice_len == len {
                return Some(i);
            } else if slice_len > best_len {
                best_len = slice_len;
                best_idx = Some(i);
            }
        }

        best_idx
    }
}

pub struct WorstFit;

impl SearchStrategy for WorstFit {
    fn search(slices: &[Slice], len: NonZeroUsize) -> Option<usize> {
        let len = len.get();
        slices
            .iter()
            .enumerate()
            .filter(|(_, s)| s.len() >= len)
            .max_by_key(|(_, s)| s.len())
            .map(|(i, _)| i)
    }
}
