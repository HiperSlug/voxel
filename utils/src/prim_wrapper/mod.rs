pub trait PrimWrapper<T> {
    fn new(inner: T) -> Self;

    fn inner(&self) -> T;
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

		impl $trait<Self> for $type {
			type Output = Self;

			fn $method(self, rhs: Self) -> Self::Output {
				let res = self.inner().$method(rhs.inner());
				$type::new(res)
			}
		}
	};
}