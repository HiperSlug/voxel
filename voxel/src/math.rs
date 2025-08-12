#[repr(i8)]
#[derive(Debug, Clone, Copy)]
pub enum Sign {
    Pos = 1,
    Neg = -1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SignedAxis {
    PosX = 0,
    NegX = 1,
    PosY = 2,
    NegY = 3,
    PosZ = 4,
    NegZ = 5,
}

impl SignedAxis {
    #[inline]
    pub const fn as_unsigned(&self) -> Axis {
        match self {
            Self::PosX | Self::NegX => Axis::X,
            Self::PosY | Self::NegY => Axis::Y,
            Self::PosZ | Self::NegZ => Axis::Z,
        }
    }

    #[inline]
    pub const fn as_index(&self) -> usize {
        (*self) as usize
    }

    #[inline]
    pub const fn is_positive(&self) -> bool {
        (self.as_index() & 1) == 0
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum AxisPermutation {
    XYZ = 0,
    YZX = 1,
    ZXY = 2,
    XZY = 3,
    YXZ = 4,
    ZYX = 5,
}

impl AxisPermutation {
    #[inline]
    pub const fn linearize_cubic<const LENGTH: usize>(
        &self,
        x: usize,
        y: usize,
        z: usize,
    ) -> usize {
        match self {
            Self::XYZ => x + y * LENGTH + z * LENGTH * LENGTH,
            Self::ZXY => z + x * LENGTH + y * LENGTH * LENGTH,
            Self::YZX => y + z * LENGTH + x * LENGTH * LENGTH,
            Self::ZYX => z + y * LENGTH + x * LENGTH * LENGTH,
            Self::XZY => x + z * LENGTH + y * LENGTH * LENGTH,
            Self::YXZ => y + x * LENGTH + z * LENGTH * LENGTH,
        }
    }

    #[inline]
    pub const fn even(axis: Axis) -> Self {
        match axis {
            Axis::X => Self::XYZ,
            Axis::Y => Self::YZX,
            Axis::Z => Self::ZXY,
        }
    }

    #[inline]
    pub const fn odd(axis: Axis) -> Self {
        match axis {
            Axis::X => Self::XZY,
            Axis::Y => Self::YXZ,
            Axis::Z => Self::ZYX,
        }
    }
}
