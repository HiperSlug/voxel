pub mod padded {
    use bevy::math::UVec3;
    use ndshape::{ConstPow2Shape3u32, ConstShape};

    use crate::math::axis::AxisMap;

    pub const BITS: u32 = 6;

    pub const LEN: usize = 1 << BITS;
    pub const AREA: usize = LEN.pow(2);
    pub const VOL: usize = LEN.pow(3);

    pub type Shape = ConstPow2Shape3u32<BITS, BITS, BITS>;

    pub const X_SHIFT: usize = Shape::SHIFTS[0] as usize;
    pub const Y_SHIFT: usize = Shape::SHIFTS[1] as usize;
    pub const Z_SHIFT: usize = Shape::SHIFTS[2] as usize;

    pub const SHIFTS: AxisMap<usize> = AxisMap::from_array([X_SHIFT, Y_SHIFT, Z_SHIFT]);

    pub const X_STRIDE: usize = 1 << X_SHIFT;
    pub const Y_STRIDE: usize = 1 << Y_SHIFT;
    pub const Z_STRIDE: usize = 1 << Z_SHIFT;

    #[inline]
    pub fn linearize(pos: UVec3) -> usize {
        debug_assert!(pos.cmplt(UVec3::splat(LEN as u32)).all());
        Shape::linearize(pos.into()) as usize
    }

    #[inline]
    pub fn delinearize(index: usize) -> UVec3 {
        debug_assert!(index < VOL);
        Shape::delinearize(index as u32).into()
    }
}

pub mod unpadded {
    use bevy::math::UVec3;

    use super::padded::{LEN as P_LEN, delinearize as p_delinearize, linearize as p_linearize};

    pub const LEN: usize = P_LEN - 2;
    pub const AREA: usize = LEN.pow(2);
    pub const VOL: usize = LEN.pow(3);

    #[inline]
    pub fn linearize(pos: UVec3) -> usize {
        p_linearize(pos + UVec3::ONE)
    }

    #[inline]
    pub fn delinearize(index: usize) -> UVec3 {
        p_delinearize(index) - UVec3::ONE
    }
}

use bevy::prelude::*;

use crate::voxel::VOXEL_LEN;

pub const WORLD_CHUNK_LEN: f32 = unpadded::LEN as f32 * VOXEL_LEN;

#[derive(Debug, Deref, DerefMut, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ChunkPos(pub IVec3);

impl From<IVec3> for ChunkPos {
    fn from(value: IVec3) -> Self {
        Self(value)
    }
}

impl From<ChunkPos> for IVec3 {
    fn from(value: ChunkPos) -> Self {
        value.0
    }
}

impl ChunkPos {
    #[inline]
    pub fn as_world(&self) -> Vec3 {
        self.as_vec3() * WORLD_CHUNK_LEN
    }

    #[inline]
    pub fn from_world(world: Vec3) -> Self {
        (world / WORLD_CHUNK_LEN).floor().as_ivec3().into()
    }

    #[inline]
    pub fn as_voxel(&self) -> IVec3 {
        self.0 * unpadded::LEN as i32
    }

    #[inline]
    pub fn from_voxel(voxel_pos: IVec3) -> Self {
        // div_euclid acts like flooring division in this case
        voxel_pos.map(|x| x.div_euclid(unpadded::LEN as i32)).into()
    } 
}
