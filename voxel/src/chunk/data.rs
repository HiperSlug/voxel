use bevy::{math::IVec3, render::render_resource::ShaderType};
use bytemuck::{Pod, Zeroable};
use std::ops::Range;

use crate::{math::signed_axis::*, voxel::Voxel};

use super::padded::{AREA, VOL};

#[derive(Debug)]
pub struct Chunk {
    pub voxels: [Voxel; VOL],
    pub opaque_mask: [u64; AREA],
    pub transparent_mask: [u64; AREA],
}

// This can be aligned to 8 bytes instead of 16 bytes by
// storing the voxel_position (u6) and a chunk_index that
// points to a chunk_pos (i32) in a storage buffer.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, ShaderType)]
pub struct VoxelQuad {
    pos: IVec3,
    data: u32,
}

impl VoxelQuad {
    #[inline]
    pub const fn new(
        pos: IVec3,
        w: u32,
        h: u32,
        signed_axis: SignedAxis,
        texture_index: u32,
    ) -> Self {
        // this must match the shader
        let signed_axis = match signed_axis {
            PosX => 0,
            PosY => 1,
            PosZ => 2,
            NegX => 3,
            NegY => 4,
            NegZ => 5,
        };

        Self {
            pos,
            data: signed_axis << 28 | h << 22 | w << 16 | texture_index,
        }
    }
}

pub struct ChunkMesh {
    pub offsets: [u32; 7],
}

impl ChunkMesh {
    pub fn range(&self, signed_axis: SignedAxis) -> Range<u32> {
        match signed_axis {
            PosX => self.offsets[0]..self.offsets[1],
            NegX => self.offsets[1]..self.offsets[2],
            PosY => self.offsets[2]..self.offsets[3],
            NegY => self.offsets[3]..self.offsets[4],
            PosZ => self.offsets[4]..self.offsets[5],
            NegZ => self.offsets[5]..self.offsets[6],
        }
    }
}
