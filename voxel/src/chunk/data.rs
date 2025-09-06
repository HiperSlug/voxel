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

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, ShaderType)]
pub struct VoxelQuad {
    position: IVec3,
    data: u32,
}

impl VoxelQuad {
    #[inline]
    pub const fn new(
        x: u32,
        y: u32,
        z: u32,
        w: u32,
        h: u32,
        signed_axis: SignedAxis,
        texture_index: u32,
        chunk_index: u32,
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
            first: h << 24 | w << 18 | z << 12 | y << 6 | x,
            second: chunk_index << 16 | texture_index << 3 | signed_axis,
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
            PosY => self.offsets[1]..self.offsets[2],
            PosZ => self.offsets[2]..self.offsets[3],
            NegX => self.offsets[3]..self.offsets[4],
            NegY => self.offsets[4]..self.offsets[5],
            NegZ => self.offsets[5]..self.offsets[6],
        }
    }
}
