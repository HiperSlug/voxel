/// world space
pub const LENGTH: f32 = 0.5;

const BITS_IN_ID: u8 = 10;

const ID_MASK: u16 = (1 << BITS_IN_ID) - 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Voxel(u16);

impl Voxel {
    #[inline]
    pub fn id(&self) -> u16 {
        self.0 & ID_MASK
    }

    #[inline]
    pub fn visibility(self) -> VoxelVisibility {
        VoxelVisibility::from_bits((self.0 >> BITS_IN_ID) as u8)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VoxelVisibility{
    pub pos_x: bool,
    pub neg_x: bool,
    pub pos_y: bool,
    pub neg_y: bool,
    pub pos_z: bool,
    pub neg_z: bool,
}

impl VoxelVisibility {
    #[inline]
    pub fn from_bits(bits: u8) -> Self {
        VoxelVisibility {
            pos_x: bits & 1 != 0,
            neg_x: (bits >> 1) & 1 != 0,
            pos_y: (bits >> 2) & 1 != 0,
            neg_y: (bits >> 3) & 1 != 0,
            pos_z: (bits >> 4) & 1 != 0,
            neg_z: (bits >> 5) & 1 != 0,
        }
    }
}