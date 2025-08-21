use bevy::render::render_resource::ShaderType;
use bytemuck::{Pod, Zeroable};
use freelist::Slice;
use std::fmt::Debug;

use crate::{
    render::buffer::{BufferAllocation, MultiBufferAllocation},
    voxel::Voxel,
};

use super::{PADDED_CHUNK_AREA, PADDED_CHUNK_VOLUME};

const MASK_6: u32 = 0x3F;

#[derive(Debug)]
pub struct Chunk {
    pub voxels: [Voxel; PADDED_CHUNK_VOLUME],
    pub opaque_mask: [u64; PADDED_CHUNK_AREA],
    pub transparent_mask: [u64; PADDED_CHUNK_AREA],
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Pod, Zeroable, ShaderType)]
pub struct VoxelQuad {
    spatial: u32,
    pub id: u32,
}

impl VoxelQuad {
    #[inline]
    pub const fn new(x: u32, y: u32, z: u32, w: u32, h: u32, id: u32) -> Self {
        Self {
            spatial: h << 24 | w << 18 | z << 12 | y << 6 | x,
            id,
        }
    }

    #[inline]
    pub const fn x(&self) -> u32 {
        self.spatial & MASK_6
    }

    #[inline]
    pub const fn y(&self) -> u32 {
        (self.spatial >> 6) & MASK_6
    }

    #[inline]
    pub const fn z(&self) -> u32 {
        (self.spatial >> 12) & MASK_6
    }

    #[inline]
    pub const fn w(&self) -> u32 {
        (self.spatial >> 18) & MASK_6
    }

    #[inline]
    pub const fn h(&self) -> u32 {
        (self.spatial >> 24) & MASK_6
    }
}

impl Debug for VoxelQuad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "VoxelQuad {{ x: {}, y: {}, z: {}, w: {}, h: {}, texture_id: {} }}",
            self.x(),
            self.y(),
            self.z(),
            self.w(),
            self.h(),
            self.id
        )
    }
}

pub struct ChunkMesh {
    pub allocation: Option<MultiBufferAllocation<VoxelQuad, 6>>,
}
