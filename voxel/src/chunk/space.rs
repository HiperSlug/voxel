pub mod pad {
    use bevy::math::U8Vec3;
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

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ChunkOffset(U8Vec3);

    impl ChunkOffset {
        pub fn new(value: U8Vec3) -> Option<Self> {
            const MAX: U8Vec3 = U8Vec3::splat(LEN as u8);

            if value.cmplt(MAX).all() {
                Some(Self(value))
            } else {
                None
            }
        }

        pub fn new_unchecked(value: U8Vec3) -> Self {
            Self(value)
        }

        pub fn get(&self) -> U8Vec3 {
            self.0
        }

        pub fn linearize(&self) -> usize {
            Shape::linearize(self.0.as_uvec3().to_array()) as usize
        }

        pub fn delinearize(index: usize) -> Option<Self> {
            let u8vec3  = U8Vec3::from(Shape::delinearize(index as u32).map(|u32| u32 as u8));
            Self::new(u8vec3)
        }

        pub fn delinearize_unchecked(index: usize) -> Self {
            let u8vec3  = U8Vec3::from(Shape::delinearize(index as u32).map(|u32| u32 as u8));
            Self::new_unchecked(u8vec3)
        }
    }

    impl TryFrom<U8Vec3> for ChunkOffset {
        type Error = ();

        fn try_from(value: U8Vec3) -> Result<Self, Self::Error> {
            Self::new(value).ok_or(())
        }
    }

    impl From<ChunkOffset> for U8Vec3 {
        fn from(value: ChunkOffset) -> Self {
            value.0
        }
    }
}

pub mod unpad {
    use bevy::math::U8Vec3;

    use crate::voxel::WORLD_VOXEL_LEN;

    use super::pad::{LEN as PAD_LEN, ChunkOffset as PadChunkOffset};

    pub const LEN: usize = PAD_LEN - 2;
    pub const AREA: usize = LEN.pow(2);
    pub const VOL: usize = LEN.pow(3);

    pub const WORLD_LEN: f32 = LEN as f32 * WORLD_VOXEL_LEN;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ChunkOffset(U8Vec3);

    impl ChunkOffset {
        pub fn new(value: U8Vec3) -> Option<Self> {
            const MAX: U8Vec3 = U8Vec3::splat(LEN as u8);

            if value.cmplt(MAX).all() {
                Some(Self(value))
            } else {
                None
            }
        }

        pub fn new_unchecked(value: U8Vec3) -> Self {
            Self(value)
        }

        pub fn get(&self) -> U8Vec3 {
            self.0
        }

        pub fn linearize(&self) -> usize {
            PadChunkOffset::from(*self).linearize()
        }

        pub fn delinearize(index: usize) -> Option<ChunkOffset> {
            let u8vec3 = PadChunkOffset::delinearize_unchecked(index).get();
            Self::new(u8vec3)
        }

        pub fn delinearize_unchecked(index: usize) -> ChunkOffset {
            let u8vec3 = PadChunkOffset::delinearize_unchecked(index).get();
            Self::new_unchecked(u8vec3)
        }
    }

    impl TryFrom<U8Vec3> for ChunkOffset {
        type Error = ();

        fn try_from(value: U8Vec3) -> Result<Self, Self::Error> {
            Self::new(value).ok_or(())
        }
    }

    impl From<ChunkOffset> for U8Vec3 {
        fn from(value: ChunkOffset) -> Self {
            value.0
        }
    }

    impl From<ChunkOffset> for PadChunkOffset {
        fn from(value: ChunkOffset) -> Self {
            PadChunkOffset::new_unchecked(value.0 + U8Vec3::ONE)
        }
    }

    impl TryFrom<PadChunkOffset> for ChunkOffset {
        type Error = ();

        fn try_from(value: PadChunkOffset) -> Result<Self, Self::Error> {
            Self::new(value.get() - U8Vec3::ONE).ok_or(())
        }
    }
}

pub use unpad::{ChunkOffset, WORLD_LEN as WORLD_CHUNK_LEN, LEN as CHUNK_LEN};

use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::voxel::WORLD_VOXEL_LEN;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct VoxelPos(pub IVec3);

impl VoxelPos {
    pub fn chunk_pos(&self) -> ChunkPos {
        ChunkPos(self.0.div_euclid(IVec3::splat(CHUNK_LEN as i32)))
    }

    pub fn chunk_offset(&self) -> ChunkOffset {
        ChunkOffset::new_unchecked(self.0.rem_euclid(IVec3::splat(CHUNK_LEN as i32)).as_u8vec3())
    }

    pub fn from_components(chunk_pos: ChunkPos, chunk_offset: ChunkOffset) -> Self {
        Self(chunk_pos.voxel_origin() + chunk_offset.get().as_ivec3())
    }

    pub fn as_components(&self) -> (ChunkPos, ChunkOffset) {
        (self.chunk_pos(), self.chunk_offset())
    }

    pub fn world_origin(&self) -> Vec3 {
        self.0.as_vec3() * WORLD_VOXEL_LEN
    }

    pub fn from_world_pos(world_pos: Vec3) -> Self {
        Self(world_pos.div_euclid(Vec3::splat(WORLD_VOXEL_LEN)).as_ivec3())
    }
}

impl From<IVec3> for VoxelPos {
    fn from(value: IVec3) -> Self {
        VoxelPos(value)
    }
}

impl From<VoxelPos> for IVec3 {
    fn from(value: VoxelPos) -> Self {
        value.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkPos(pub IVec3);

impl ChunkPos {
    pub fn new_checked(value: IVec3) -> Option<Self> {
        const MIN: IVec3 = IVec3::MIN.wrapping_div(IVec3::splat(CHUNK_LEN as i32));
        const MAX: IVec3 = IVec3::MAX.wrapping_div(IVec3::splat(CHUNK_LEN as i32));

        if value.cmpge(MIN).all() && value.cmple(MAX).all() {
            Some(Self(value))
        } else {
            None
        }
    }

    pub fn voxel_origin(&self) -> IVec3 {
        self.0 * CHUNK_LEN as i32
    }

    pub fn world_origin(&self) -> Vec3 {
        self.0.as_vec3() * WORLD_CHUNK_LEN
    }

    pub fn from_world_pos(world_pos: Vec3) -> Self {
        Self(world_pos.div_euclid(Vec3::splat(WORLD_CHUNK_LEN)).as_ivec3())
    }
}

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
