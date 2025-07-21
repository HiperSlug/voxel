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
/// - `$trait` - The ops trait to implement (e.g., `Add`)
/// - `$method` - The method name from the trait (e.g., `add`)
#[macro_export]
macro_rules! transparent_ops {
    ($wrapper:ident, $trait:ident, $method:ident, $constructor:ident) => {
        impl std::ops::$trait<<$wrapper as $crate::Wrapper>::Inner> for $wrapper
        where
            $wrapper: $crate::Wrapper,
            <$wrapper as $crate::Wrapper>::Inner:
                std::ops::$trait<Output = <$wrapper as $crate::Wrapper>::Inner>,
        {
            type Output = Self;

            fn $method(self, rhs: <$wrapper as $crate::Wrapper>::Inner) -> Self::Output {
                Self::$constructor(self.into_inner().$method(rhs))
            }
        }

        impl std::ops::$trait<$wrapper> for $wrapper
        where
            $wrapper: $crate::Wrapper,
            <$wrapper as $crate::Wrapper>::Inner:
                std::ops::$trait<Output = <$wrapper as $crate::Wrapper>::Inner>,
        {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self::Output {
                Self::$constructor(self.into_inner().$method(rhs.into_inner()))
            }
        }
    };
}

/// Impliments default ops for a wrapper
///
/// # Traits
/// - Add
/// - Sub
/// - Mul
/// - Div
/// - Rem
///
/// # Parameters
/// - `$wrapper` - The target type
#[macro_export]
macro_rules! default_transparent_ops {
    ($wrapper:ident, $constructor:ident) => {
        $crate::transparent_ops!($wrapper, Add, add, $constructor);
        $crate::transparent_ops!($wrapper, Sub, sub, $constructor);
        $crate::transparent_ops!($wrapper, Mul, mul, $constructor);
        $crate::transparent_ops!($wrapper, Div, div, $constructor);
        $crate::transparent_ops!($wrapper, Rem, rem, $constructor);
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

    default_transparent_ops!(MyWrapper, new);

    #[test]
    fn transparent_operations() {
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
