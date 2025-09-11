pub mod pad {
    use ndshape::{ConstPow2Shape3u32, ConstShape};

    pub const BITS: u32 = 6;

    pub const LEN: usize = 1 << BITS;
    pub const AREA: usize = LEN.pow(2);
    pub const VOL: usize = LEN.pow(3);

    pub type Shape = ConstPow2Shape3u32<BITS, BITS, BITS>;

    pub const SHIFT_0: usize = Shape::SHIFTS[0] as usize;
    pub const SHIFT_1: usize = Shape::SHIFTS[1] as usize;
    pub const SHIFT_2: usize = Shape::SHIFTS[2] as usize;

    pub const STRIDE_0: usize = 1 << SHIFT_0;
    pub const STRIDE_1: usize = 1 << SHIFT_1;
    pub const STRIDE_2: usize = 1 << SHIFT_2;

    pub fn linearize<T>(p: T) -> usize
    where
        T: Into<[u32; 3]>,
    {
        Shape::linearize(p.into()) as usize
    }

    pub fn delinearize<T>(i: usize) -> T
    where
        T: From<[u32; 3]>,
    {
        Shape::delinearize(i as u32).into()
    }
}

pub mod unpad {
    use crate::voxel::WORLD_VOXEL_LEN;

    use super::pad::{
        LEN as PAD_LEN, STRIDE_0, STRIDE_1, STRIDE_2, delinearize as pad_delinearize,
        linearize as pad_linearize,
    };

    pub const LEN: usize = PAD_LEN - 2;
    pub const AREA: usize = LEN.pow(2);
    pub const VOL: usize = LEN.pow(3);

    pub const WORLD_LEN: f32 = LEN as f32 * WORLD_VOXEL_LEN;

    /// `pad_linearize([1, 1, 1])`
    const INDEX_PADDING: usize = STRIDE_0 + STRIDE_1 + STRIDE_2;

    pub fn linearize<T>(p: T) -> usize
    where
        T: Into<[u32; 3]>,
    {
        pad_linearize(p) + INDEX_PADDING
    }

    pub fn delinearize<T>(i: usize) -> T
    where
        T: From<[u32; 3]>,
    {
        pad_delinearize(i - INDEX_PADDING)
    }
}

use bevy::math::IVec3;
pub use unpad::{LEN as CHUNK_LEN, WORLD_LEN as WORLD_CHUNK_LEN};

pub type ChunkPos = IVec3;

pub const fn chunk_origin(chunk_pos: IVec3) -> IVec3 {
    chunk_pos.wrapping_mul(IVec3::splat(CHUNK_LEN as i32))
}
