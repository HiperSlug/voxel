use enum_map::Enum;
use serde::{Deserialize, Serialize};

#[repr(i8)]
#[derive(Debug, Clone, Copy)]
#[derive(Enum)]
#[derive(Deserialize, Serialize)]
pub enum Sign {
    Pos = 1,
    Neg = -1,
}

impl Sign {
    #[inline]
    pub const fn as_i8(&self) -> i8 {
        (*self) as i8
    }
}
