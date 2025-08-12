pub mod voxel {
    use bevy::prelude::{Deref, DerefMut};

    pub const LENGTH: f32 = 0.5;

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deref, DerefMut, PartialOrd, Ord)]
    pub struct Voxel {
        pub id: u16,
    }

    impl Voxel {
        #[inline]
        pub fn is_sentinel(&self) -> bool {
            self.id == u16::MAX
        }

        #[inline]
        pub fn index(&self) -> usize {
            self.id as usize
        }
    }
}

pub mod chunk {
    // TODO unpad the chunk and come up with a different solution for meshing
    // possible solution: VoxelToolRead -> read access to an arbitrary volume idk how that would work
    use super::voxel::{self, Voxel};
    use arc_swap::ArcSwap;
    use bevy::math::UVec3;
    use ndshape::{ConstPow2Shape3u32, ConstShape};

    const BITS: u32 = 5;

    pub const PADDED_LENGTH_IN_VOXELS: u32 = 1 << BITS;
    pub const PADDED_VOLUME_IN_VOXELS: usize = (PADDED_LENGTH_IN_VOXELS as usize).pow(3);

    pub const LENGTH_IN_VOXELS: u32 = PADDED_LENGTH_IN_VOXELS - 2;

    pub const LENGTH: f32 = LENGTH_IN_VOXELS as f32 * voxel::LENGTH;

    pub type Shape = ConstPow2Shape3u32<BITS, BITS, BITS>;

    #[derive(Debug)]
    pub enum Chunk {
        Uniform(Voxel),
        Mixed(ArcSwap<[Voxel; PADDED_VOLUME_IN_VOXELS]>),
    }

    impl Chunk {
        pub fn attempt_collapse(&mut self) -> bool {
            use Chunk::*;
            match self {
                Uniform(_) => true,
                Mixed(voxels) => {
                    let guard = voxels.load();
                    let base = guard[0];
                    let can_collapse = guard.iter().skip(1).all(|v| *v == base);
                    if can_collapse {
                        *self = Uniform(base);
                    }
                    can_collapse
                }
            }
        }
    }

    pub fn linearize(pos: UVec3) -> usize {
        Shape::linearize(pos.to_array()) as usize
    }

    pub fn delinearize(index: usize) -> UVec3 {
        Shape::delinearize(index as u32).into()
    }
}
