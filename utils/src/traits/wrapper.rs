/// Trait for types with interfaces into inner values
///
/// # Associated Type
/// - 'Inner': The wrapped type
///
/// # Required Methods
/// - 'wrap(Self::Inner) -> Self' - Constructs 'Self' from the inner value
/// - 'inner(&self) -> &Self::Inner' - Returns a shared reference to the inner value
/// - 'into_inner(self) -> Self::Inner' - Consumes 'self' and returns the inner value
pub trait Wrapper {
    type Inner;

    fn inner(&self) -> &Self::Inner;
    fn into_inner(self) -> Self::Inner;
}

/// Impliments transparent operations for a Wrapper
///
/// # Parameters
/// - `$wrapper` - The target type
/// - `$trait` - The trait to impliment (e.g., `std::ops::Add`)
/// - `$method` - The method name from the trait (e.g., `add`)
/// - `$constructor` - The method to create `Self` from the inner type
/// - `$output` - The `Output` type of the `$constructor`
#[macro_export]
macro_rules! transparent_ops {
    (
        $wrapper:ident, 
        $trait:ident, 
        $method:ident, 
        $constructor:expr,
        $output:ty
    ) => {
        impl $trait<<$wrapper as $crate::Wrapper>::Inner> for $wrapper
        where
            $wrapper: $crate::Wrapper,
            <$wrapper as $crate::Wrapper>::Inner: $trait,
        {
            type Output = $output;

            fn $method(self, rhs: <$wrapper as $crate::Wrapper>::Inner) -> Self::Output {
                $constructor(self.into_inner().$method(rhs))
            }
        }

        impl $trait for $wrapper
        where
            $wrapper: $crate::Wrapper,
            <$wrapper as $crate::Wrapper>::Inner: $trait,
        {
            type Output = $output;

            fn $method(self, rhs: Self) -> Self::Output {
                $constructor(self.into_inner().$method(rhs.into_inner()))
            }
        }
    };
}

#[cfg(test)]
pub mod tests {
    use super::Wrapper;
    use std::ops::{Add, Div, Mul, Rem, Sub};

    #[derive(Clone, Copy)]
    pub struct MyWrapper(usize);

    impl Wrapper for MyWrapper {
        type Inner = usize;

        fn inner(&self) -> &Self::Inner {
            &self.0
        }

        fn into_inner(self) -> Self::Inner {
            self.0
        }
    }

    impl MyWrapper {
        fn new(inner: usize) -> Self {
            Self(inner)
        }
    }

    transparent_ops!(MyWrapper, Add, add, MyWrapper::new, MyWrapper);
    transparent_ops!(MyWrapper, Sub, sub, MyWrapper::new, MyWrapper);
    transparent_ops!(MyWrapper, Mul, mul, MyWrapper::new, MyWrapper);
    transparent_ops!(MyWrapper, Div, div, MyWrapper::new, MyWrapper);
    transparent_ops!(MyWrapper, Rem, rem, MyWrapper::new, MyWrapper);

    #[test]
    fn ops_with_inner() {
        let wrapped = MyWrapper(64);

        let add = *wrapped.add(16).inner();
        assert_eq!(add, 64 + 16);

        let sub = *wrapped.sub(16).inner();
        assert_eq!(sub, 64 - 16);

        let mul = *wrapped.mul(16).inner();
        assert_eq!(mul, 64 * 16);

        let div = *wrapped.div(16).inner();
        assert_eq!(div, 64 / 16);

        let rem = *wrapped.rem(16).inner();
        assert_eq!(rem, 64 % 16);
    }

    #[test]
    fn ops_with_self() {
        let wrapped = MyWrapper(64);

        let add = *wrapped.add(MyWrapper(16)).inner();
        assert_eq!(add, 64 + 16);

        let sub = *wrapped.sub(MyWrapper(16)).inner();
        assert_eq!(sub, 64 - 16);

        let mul = *wrapped.mul(MyWrapper(16)).inner();
        assert_eq!(mul, 64 * 16);

        let div = *wrapped.div(MyWrapper(16)).inner();
        assert_eq!(div, 64 / 16);

        let rem = *wrapped.rem(MyWrapper(16)).inner();
        assert_eq!(rem, 64 % 16);
    }
}
