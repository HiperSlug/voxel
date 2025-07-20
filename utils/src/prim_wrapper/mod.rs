/// A trait for a structure that contains a single primitive.
pub trait PrimWrapper {
	type Inner;

	/// Wraps a primitive
    fn new(inner: Self::Inner) -> Self;

	/// Returns the inner value
    fn inner(&self) -> Self::Inner;
}

/// Impliments ops::trait for a PrimWrapper
/// 
/// # Parameters
/// - `$type`: The wrapper type name (e.g., `MyWrapper`)
/// - `$inner`: The inner primitive type (e.g., `u8`)
/// - `$trait`: The operator trait to implement (e.g., `Add`)
/// - `$method`: The method name from the trait (e.g., `add`)
#[macro_export]
macro_rules! prim_wrapper_ops {
	($type:ident, $inner:ty, $trait:ident, $method:ident) => {
		impl std::ops::$trait<$inner> for $type {
			type Output = Self;

			fn $method(self, rhs: $inner) -> Self::Output {
				let res = self.inner().$method(rhs);
				$type::new(res)
			}
		}

		impl std::ops::$trait<Self> for $type {
			type Output = Self;

			fn $method(self, rhs: Self) -> Self::Output {
				let res = self.inner().$method(rhs.inner());
				$type::new(res)
			}
		}
	};
}

/// Impliments default ops::trait for a PrimWrapper
/// 
/// # Traits
/// - Add
/// - Sub
/// - Mul
/// - Div
/// - Rem
/// 
/// # Parameters
/// - `$type`: The wrapper type name (e.g., `MyWrapper`)
/// - `$inner`: The inner primitive type (e.g., `u8`)
#[macro_export]
macro_rules! prim_wrapper_default_ops {
	($type:ident, $inner:ty) => {
		$crate::prim_wrapper_ops!($type, $inner, Add, add);
		$crate::prim_wrapper_ops!($type, $inner, Sub, sub);
		$crate::prim_wrapper_ops!($type, $inner, Mul, mul);
		$crate::prim_wrapper_ops!($type, $inner, Div, div);
		$crate::prim_wrapper_ops!($type, $inner, Rem, rem);
	};
}