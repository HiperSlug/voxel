use thiserror::Error;
use utils::{BoundsError, IndexOutOfBounds};
pub use variant::Variant;

pub mod variant {
    use utils::BoundsError;

    /// A data piece in a 'Pallet' that stores both its inner data and the number of times it is used.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Variant<T> {
        inner: T,
        pub count: u16,
    }

    impl<T: Copy> Variant<T> {
        /// Creates a new Variant
        pub fn new(inner: T, count: u16) -> Self {
            Self { inner, count }
        }

        /// The inner value
        pub fn inner(&self) -> T {
            self.inner
        }

        /// Increases count by one
        ///
        /// # Errors
        /// - Err(BoundsError<u16>) when count would increase beyond u16::MAX
        pub fn increment(&mut self) -> Result<(), BoundsError<u16>> {
            if self.count == u16::MAX {
                Err(BoundsError::implicit(self.count))
            } else {
                self.count += 1;
                Ok(())
            }
        }

        /// Decreases count by one
        ///
        /// # Errors
        /// - Err(BoundsError<u16>) when count would decrease below 0
        pub fn decrement(&mut self) -> Result<(), BoundsError<u16>> {
            if self.count == 0 {
                Err(BoundsError::implicit(self.count))
            } else {
                self.count -= 1;
                Ok(())
            }
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

/// Sorted Vec<Variant>
pub struct Pallet<T> {
    inner: Vec<Variant<T>>,
}

impl<T: Ord + Copy> Pallet<T> {
    /// Creates a new pallet from a 'Vec<Variant<T>>'
    ///
    /// The 'Vec<Variant<T>>' doesnt need to be sorted
    pub fn new(mut inner: Vec<Variant<T>>) -> Self {
        inner.sort_unstable();
        Self { inner }
    }

    /// Alias for 'inner.binary_search(variant)'
    pub fn binary_search(&self, variant: &Variant<T>) -> Result<usize, usize> {
        self.inner.binary_search(&variant)
    }

    /// Alias for 'inner.get(idx)'
    pub fn get(&self, idx: usize) -> Option<&Variant<T>> {
        self.inner.get(idx)
    }

    /// Gets a value if it exists, inserts it if it doesnt.
    ///
    /// # Returns
    /// 'Ok(index: usize)' if the value existed
    ///
    /// 'Err(index: usize)' if the value was inserted with a default value of one
    pub fn get_or_insert(&mut self, data: T) -> GetOrInsertResult {
        let variant = Variant::new(data, 1);

        match self.binary_search(&variant) {
            Ok(i) => GetOrInsertResult::Found(i),
            Err(i) => {
                self.inner.insert(i, variant);
                GetOrInsertResult::Inserted(i)
            }
        }
    }

    /// Decreases the 'count' of 'Variant' at 'index' by 1.
    /// If 'Variant' reaches '0', removes the variant.
    ///
    /// # Returns
    /// - 'Ok(bool)' if variant.count was decremented
    ///     - 'true' if a variant was removed (indices where shifted)
    ///     - 'false' if the count was decremented without removal.
    /// - 'Err(IndexOutOfBounds)' if index is out of bounds
    pub fn decrement_index(&mut self, index: usize) -> Result<bool, ChangeCountError> {
        if index >= self.inner.len() {
            Err(IndexOutOfBounds(index, self.inner.len()))?
        }
        let variant = &mut self.inner[index];

        let should_remove = variant.count <= 1;
        if should_remove {
            self.inner.remove(index);
        } else {
            variant.decrement()?;
        }

        Ok(should_remove)
    }

    /// Increases the 'count' of 'Variant' at 'index' by 1.
    ///
    /// # Returns
    /// - 'Ok(_)' if variant.count was incremented
    /// - 'Err(IndexOutOfBounds)' if index is out of bounds
    pub fn increment_index(&mut self, index: usize) -> Result<(), ChangeCountError> {
        if index >= self.inner.len() {
            Err(IndexOutOfBounds(index, self.inner.len()))?
        }
        self.inner[index].increment()?;

        Ok(())
    }

    /// Returns the minimum number of bits required to index this pallet
    pub fn req_bits(&self) -> usize {
        let len = self.inner.len();
        if len <= 1 {
            1
        } else {
            (len - 1).ilog2() as usize + 1
        }
    }
}

#[derive(Debug, Error)]
pub enum ChangeCountError {
    #[error("{0}")]
    IndexOutOfBounds(#[from] IndexOutOfBounds),
    #[error("{0}")]
    BoundsError(#[from] BoundsError<u16>),
}

#[derive(Debug, Clone)]
pub enum GetOrInsertResult {
    Found(usize),
    Inserted(usize),
}
