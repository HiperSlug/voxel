use crate::{FullInt, Wrapper};

pub use errors::BoundsError;

mod errors {
    use crate::FullInt;
    use thiserror::Error;

    #[derive(Debug, Error)]
    #[error("value {value} is out of bounds [{lower}..{upper})")]
    pub struct BoundsError<T: FullInt> {
        pub value: T,
        /// Exclusive
        pub upper: T,
        /// Inclusive
        pub lower: T,
    }

    impl<T: FullInt> BoundsError<T> {
        pub fn implicit(value: T) -> Self {
            Self {
                value,
                upper: T::max_value(),
                lower: T::min_value(),
            }
        }
    }
}

/// Trait for Wrappers around Integers that restricts values
///
/// # Associated Constants
/// 'MAX_EXCLUSIVE: Self::Inner' - Exclusive upper bound
/// 'MIN: Self::Inner' - Inclusive lower bound
pub trait BoundInt: Sized + Wrapper
where
    Self::Inner: FullInt,
{
    /// Exclusive upper bound
    const MAX_EXCLUSIVE: Self::Inner;
    /// Inclusive lower bound
    const MIN_INCLUSIVE: Self::Inner;

    /// Alias for Self::MAX_EXCLUSIVE
    fn max() -> Self::Inner {
        Self::MAX_EXCLUSIVE
    }

    /// Alias for Self::MIN_INCLUSIVE
    fn min() -> Self::Inner {
        Self::MIN_INCLUSIVE
    }

    /// The number of possible values
    ///
    /// Alias for 'Self::MAX_EXCLUSIVE - Self::MIN_INCLUSIVE'
    fn span() -> Self::Inner {
        Self::max() - Self::min()
    }

    /// Creates a bound wrapper around a base type 'Self::Inner'
    ///
    /// # Returns
    /// - 'Ok(Self)'
    /// - 'Err(OutOfBoundsError)' - if value is out of bounds
    ///
    /// # Example
    /// 'fn bounded_wrap(inner: Self::Inner) -> Result<Self, BoundsError<Self::Inner>> {
    ///     Self::validate_value(inner).map(|_| Self(inner))
    /// }'
    fn bounded_wrap(inner: Self::Inner) -> Result<Self, BoundsError<Self::Inner>>;

    /// Returns if the value is out of bounds
    ///
    /// # Returns
    /// 'Ok(_)' if value is in bounds
    /// 'Err(OutOfBoundsError)' if value is out of bounds
    fn validate_value(inner: Self::Inner) -> Result<(), BoundsError<Self::Inner>> {
        if inner >= Self::max() || inner < Self::min() {
            Err(BoundsError {
                value: inner,
                upper: Self::max(),
                lower: Self::min(),
            })
        } else {
            Ok(())
        }
    }

    /// Alias for Self::validate_value(*self.inner())
    fn validate(&self) -> Result<(), BoundsError<Self::Inner>> {
        Self::validate_value(*self.inner())
    }
}

/// A trait for BoundInt that includes a function to get wrapped values
pub trait CyclicBoundInt: BoundInt<Inner: FullInt> {
    /// Creates a bound wrapper around a base type 'Self::Inner'
    ///
    /// # Bounds
    /// Values exceeding 'MAX_EXCLUSIVE' wrap to 'MIN_INCLUSIVE'
    ///
    /// Values below 'MIN_INCLUSIVE' wrap to 'MAX_EXCLUSIVE - 1'
    fn normalized_wrap(inner: Self::Inner) -> Self {
        let cycled = Self::normalize(inner);
        Self::bounded_wrap(cycled).unwrap()
    }

    /// Returns 'Self::Inner' known to be in bounds
    ///
    /// # Bounds
    /// Values exceeding 'MAX_EXCLUSIVE' wrap to 'MIN_INCLUSIVE'
    ///
    /// Values below 'MIN_INCLUSIVE' wrap to 'MAX_EXCLUSIVE - 1'
    fn normalize(inner: Self::Inner) -> Self::Inner {
        let shifted = inner - Self::min();
        let rem = shifted % Self::span();

        let normalized = rem + Self::span();
        let bounded_offset = normalized % Self::span();

        bounded_offset + Self::min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MyBoundInt(isize);

    impl Wrapper for MyBoundInt {
        type Inner = isize;

        fn inner(&self) -> &Self::Inner {
            &self.0
        }

        fn into_inner(self) -> Self::Inner {
            self.0
        }
    }

    impl BoundInt for MyBoundInt {
        const MAX_EXCLUSIVE: Self::Inner = 11;
        const MIN_INCLUSIVE: Self::Inner = -11;

        fn bounded_wrap(inner: Self::Inner) -> Result<Self, BoundsError<Self::Inner>> {
            Self::validate_value(inner).map(|_| Self(inner))
        }
    }

    impl CyclicBoundInt for MyBoundInt {}

    #[test]
    fn neg() {
        MyBoundInt::bounded_wrap(-11).unwrap();
        MyBoundInt::bounded_wrap(-12).unwrap_err();
    }

    #[test]
    fn pos() {
        MyBoundInt::bounded_wrap(10).unwrap();
        MyBoundInt::bounded_wrap(11).unwrap_err();
    }

    #[test]
    fn cycle() {
        // shouldnt wrap
        let result = MyBoundInt::normalized_wrap(-11);
        assert_eq!(result.into_inner(), -11);

        let result = MyBoundInt::normalized_wrap(10);
        assert_eq!(result.into_inner(), 10);

        // wrap once
        let result = MyBoundInt::normalized_wrap(11);
        assert_eq!(result.into_inner(), -11);

        let result = MyBoundInt::normalized_wrap(-12);
        assert_eq!(result.into_inner(), 10);

        // wrap twice
        let result = MyBoundInt::normalized_wrap(11 + MyBoundInt::span());
        assert_eq!(result.into_inner(), -11);

        let result = MyBoundInt::normalized_wrap(-12 - MyBoundInt::span());
        assert_eq!(result.into_inner(), 10);
    }
}
