use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};

pub use Axis::*;

pub type AxisMap<T> = EnumMap<Axis, T>;

#[derive(Debug, Clone, Copy, Enum, Deserialize, Serialize)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub const ALL: [Self; 3] = [X, Y, Z];

    #[inline]
    pub const fn coords(&self) -> [u32; 3] {
        match self {
            X => [1, 0, 0],
            Y => [0, 1, 0],
            Z => [0, 0, 1],
        }
    }
}
