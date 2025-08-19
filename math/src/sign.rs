#[repr(i8)]
#[derive(Debug, Clone, Copy)]
pub enum Sign {
    Pos = 1,
    Neg = -1,
}

impl Sign {
    #[inline]
    pub const fn as_i8(&self) -> i8 {
        (*self) as i8
    }

    #[inline]
    pub const fn as_isize(&self) -> isize {
        (*self) as isize
    }
}
