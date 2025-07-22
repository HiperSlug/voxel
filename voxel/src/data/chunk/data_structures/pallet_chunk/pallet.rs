pub use variant::*;

pub mod variant {
    pub use errors::*;

    mod errors {
        use thiserror::Error;

        #[derive(Debug, Error)]
        #[error("Pallet count has overflowed")]
        pub struct CountOverflow;

        #[derive(Debug, Error)]
        #[error("Pallet count has underflowed")]
        pub struct CountUnderflow;
    }

    /// A data piece in a `Pallet` that stores both its `inner` data and its `count`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Variant<T> {
        inner: T,
        pub count: u16,
    }

    impl<T: Copy> Variant<T> {
        /// Creates a new `Variant`
        pub fn new(inner: T, count: u16) -> Self {
            Self { inner, count }
        }

        /// The `inner` data
        pub fn inner(&self) -> T {
            self.inner
        }

        /// Increases `count` by `1`
        ///
        /// # Errors
        /// - `Err(CountOverflow)` if count increases beyond `u16::MAX`
        pub fn increment(&mut self) -> Result<(), CountOverflow> {
            self.count = self.count.checked_add(1).ok_or(CountOverflow)?;

            Ok(())
        }

        /// Decreases `count` by `1`
        ///
        /// # Errors
        /// - `Err(CountUnderflow)` when count decreases beyond `0`
        pub fn decrement(&mut self) -> Result<(), CountUnderflow> {
            self.count = self.count.checked_sub(1).ok_or(CountUnderflow)?;

            Ok(())
        }
    }

    impl<T: PartialOrd> PartialOrd for Variant<T> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.inner.partial_cmp(&other.inner)
        }
    }

    impl<T: Ord> Ord for Variant<T> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.inner.cmp(&other.inner)
        }
    }
}

/// Either [`Gotten(index)`] or [`Inserted(index)`]
#[derive(Debug, Clone)]
pub enum GetOrInsertResult {
    Gotten(usize),
    Inserted(usize),
}

/// Sorted `Vec<Variant>` with helper functions
pub struct Pallet<T> {
    inner: Vec<Variant<T>>,
}

impl<T: Ord + Copy> Pallet<T> {
    /// Creates a new pallet from a `Vec<Variant<T>>`
    ///
    /// The `Vec<Variant<T>>` doesn't need to be sorted
    pub fn new(mut inner: Vec<Variant<T>>) -> Self {
        inner.sort_unstable();
        Self { inner }
    }

    /// Alias for `self.inner.len()`
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Alias for `inner.binary_search(variant)`
    pub fn binary_search(&self, variant: &Variant<T>) -> Result<usize, usize> {
        self.inner.binary_search(&variant)
    }

    /// Alias for `inner.get(idx)`
    pub fn get(&self, idx: usize) -> Option<&Variant<T>> {
        self.inner.get(idx)
    }

    /// Gets a value if it exists, inserts it if it doesnt
    ///
    /// # Returns
    /// - `GetOrInsertResult::Gotten(index)` when gotten
    /// -`GetOrInsertResult::Inserted(index)` when inserted with a default `count` of `1`
    pub fn get_or_insert(&mut self, data: T) -> GetOrInsertResult {
        let variant = Variant::new(data, 1);

        match self.binary_search(&variant) {
            Ok(i) => GetOrInsertResult::Gotten(i),
            Err(i) => {
                self.inner.insert(i, variant);
                GetOrInsertResult::Inserted(i)
            }
        }
    }

    /// Decreases the `count` of `Variant` at `index` by `1`
    ///
    /// If `count` reaches `0`, removes the variant
    ///
    /// # Panics
    /// When index is out of bounds
    ///
    /// # Returns
    /// - `true` if a variant was removed (indices where shifted)
    /// - `false` if the count was decremented without removal.
    pub fn decrement_index(&mut self, index: usize) -> bool {
        assert!(index < self.inner.len(), "index out of bounds");

        let variant = &mut self.inner[index];

        let should_remove = variant.count <= 1;
        if should_remove {
            self.inner.remove(index);
        } else {
            variant
                .decrement()
                .expect("count must be at least 1 before decrementing");
        }

        should_remove
    }

    /// Increases the `count` of `Variant` at `index` by `1`
    ///
    /// # Panics
    /// If index is out of bounds
    pub fn increment_index(&mut self, index: usize) -> Result<(), CountOverflow> {
        assert!(index < self.inner.len(), "index out of bounds");
        self.inner[index].increment()
    }

    /// Returns the minimum number of bits required to index this pallet
    pub fn req_bits(&self) -> usize {
        let len = self.inner.len();
        match len {
            0 => 0,
            1 => 1,
            _ => (len - 1).ilog2() as usize + 1,
        }
    }
}
