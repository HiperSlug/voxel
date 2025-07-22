pub use coord::*;
pub use position::*;

/// Side length of standard chunks
pub const CHUNK_LENGTH: u8 = 16;

/// Number of voxels in a standard chunk
///
/// Equal to CHUNK_LENGTH.pow(3)
pub const VOXELS_IN_CHUNK: u16 = (CHUNK_LENGTH as u16).pow(3);

/// Bounded [0..CHUNK_LENGTH) coordinate
pub mod coord {
    use utils::transparent_ops;
    use utils::{BoundInt, BoundsError, CyclicBoundInt, Wrapper};

    use super::CHUNK_LENGTH;

    /// Wrapper structure that binds a u8 between [0..CHUNK_LENGTH)
    ///
    /// # Bounds
    /// Use 'bounded_wrap(inner)' or 'normalized_wrap(inner)' to create bounded variants.
    ///
    /// # Operations
    /// All operations done on a localcoord wrap will overflow from MAX (15) to MIN (0) and vice versa
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct LocalCoord(u8);

    impl Wrapper for LocalCoord {
        type Inner = u8;

        fn inner(&self) -> &Self::Inner {
            &self.0
        }

        fn into_inner(self) -> Self::Inner {
            self.0
        }
    }

    impl BoundInt for LocalCoord {
        const MAX_EXCLUSIVE: Self::Inner = CHUNK_LENGTH;
        const MIN_INCLUSIVE: Self::Inner = 0;

        fn bounded_wrap(inner: Self::Inner) -> Result<Self, BoundsError<Self::Inner>> {
            Self::validate_value(inner).map(|_| Self(inner))
        }
    }

    impl CyclicBoundInt for LocalCoord {}

    use std::ops::{Add, Div, Mul, Rem, Sub};

    transparent_ops!(
        LocalCoord,
        Add,
        add,
        LocalCoord::normalized_wrap,
        LocalCoord
    );
    transparent_ops!(
        LocalCoord,
        Sub,
        sub,
        LocalCoord::normalized_wrap,
        LocalCoord
    );
    transparent_ops!(
        LocalCoord,
        Mul,
        mul,
        LocalCoord::normalized_wrap,
        LocalCoord
    );
    transparent_ops!(
        LocalCoord,
        Div,
        div,
        LocalCoord::normalized_wrap,
        LocalCoord
    );
    transparent_ops!(
        LocalCoord,
        Rem,
        rem,
        LocalCoord::normalized_wrap,
        LocalCoord
    );
}

/// Position in local chunk space
pub mod position {
    use super::{CHUNK_LENGTH, LocalCoord};
    use utils::Wrapper;

    /// Representation of a voxels position in local chunk space relative to the -x, -y, -z corner of the chunk
    ///
    /// Chunk space is constrained from [0..CHUNK_LENGTH)
    ///
    /// Represented by 3 bound LocalCoords
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct VLocalPos {
        pub x: LocalCoord,
        pub y: LocalCoord,
        pub z: LocalCoord,
    }

    impl VLocalPos {
        pub fn new(x: LocalCoord, y: LocalCoord, z: LocalCoord) -> Self {
            Self { x, y, z }
        }

        pub fn flat_index(&self) -> usize {
            *self.x.inner() as usize
                + *self.y.inner() as usize * CHUNK_LENGTH as usize
                + *self.z.inner() as usize * (CHUNK_LENGTH as usize).pow(2)
        }
    }
}
