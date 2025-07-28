/// world space
pub const LENGTH: f32 = 0.5;

/// only the first 10 bits are valid allowing 1024 voxels
pub type VoxelId = u16;

const BITS_IN_ID: u8 = 10;

const ID_MASK: u16 = (1 << BITS_IN_ID) - 1;
const VISIBILITY_MASK: u16 = !ID_MASK;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Voxel(u16);

impl Voxel {
    /// writes metatdata from param
    #[inline]
    pub fn new(data: u16) -> Self {
        Voxel(data)
    }

    /// all metadata set to 0
    #[inline]
    pub fn from_id(id: u16) -> Self {
        Voxel(id & ID_MASK)
    }

    // temp function used for testing meshing.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.id() == 0
    }

    #[inline]
    pub fn id(&self) -> VoxelId {
        self.0 & ID_MASK
    }

    #[inline]
    pub fn set_id(&mut self, to: u16) {
        self.0 &= VISIBILITY_MASK;
        self.0 |= to & ID_MASK;
    }

    #[inline]
    pub fn visibility(&self) -> VoxelVisibility {
        VoxelVisibility::from_bits((self.0 >> BITS_IN_ID) as u8)
    }

    #[inline]
    pub fn set_visibility(&mut self, to: VoxelVisibility) {
        let shifted = (to.as_bits() as u16) << BITS_IN_ID;
        self.0 &= ID_MASK;
        self.0 |= shifted;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VoxelVisibility {
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

    #[inline]
    pub fn as_bits(&self) -> u8 {
        self.pos_x as u8
            | (self.neg_x as u8) << 1
            | (self.pos_y as u8) << 2
            | (self.neg_y as u8) << 3
            | (self.pos_z as u8) << 4
            | (self.neg_z as u8) << 5
    }
}
