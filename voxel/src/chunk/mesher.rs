use bevy::math::IVec3;
use bytemuck::{Pod, Zeroable};
use enum_map::enum_map;
use std::ops::Range;

use crate::signed_axis::*;

use super::{
    Chunk, Voxel, chunk_origin,
    pad::{AREA, LEN, VOL},
    pad::{SHIFT_0, SHIFT_1, SHIFT_2, STRIDE_0, STRIDE_1, STRIDE_2},
};

// vol_*: X - `SHIFT_0`, Y - `SHIFT_1`, Z - `SHIFT_2`
// area_*: Y - `SHIFT_0`, Z - `SHIFT_1`,
// * is replaced by whichever axes are present

const UNPADDED_MASK: u64 = !(1 << 63 | 1);

pub struct Mesher {
    quads: Vec<VoxelQuad>,
    visible_masks: Box<FaceMap<[u64; AREA]>>,
    upward_merged: Box<[u8; LEN]>,
    forward_merged: Box<[u8; AREA]>,
}

impl Mesher {
    pub fn new() -> Self {
        Self {
            quads: Vec::new(),
            visible_masks: Box::new(enum_map! { _ => [0; AREA] }),
            upward_merged: Box::new([0; LEN]),
            forward_merged: Box::new([0; AREA]),
        }
    }

    pub fn clear(&mut self) {
        self.quads.clear();
        self.visible_masks.as_mut_array().fill([0; AREA]);
        self.upward_merged.fill(0);
        self.forward_merged.fill(0);
    }

    fn face_culling(
        &mut self,
        voxel_opts: &[Option<Voxel>; VOL],
        opaque_mask: &[u64; AREA],
        transparent_mask: &[u64; AREA],
    ) {
        for face in Face::ALL {
            let visible_mask = &mut self.visible_masks[face];

            let vol_adj_offset = match face {
                PosX => STRIDE_0 as isize,
                NegX => -(STRIDE_0 as isize),
                PosY => STRIDE_1 as isize,
                NegY => -(STRIDE_1 as isize),
                PosZ => STRIDE_2 as isize,
                NegZ => -(STRIDE_2 as isize),
            };

            for z in 1..LEN - 1 {
                let vol_z = z << SHIFT_2;

                let area_z = z << SHIFT_1;

                for y in 1..LEN - 1 {
                    let vol_y = y << SHIFT_1;
                    let vol_yz = vol_y | vol_z;

                    let area_y = y << SHIFT_0;
                    let area_yz = area_y | area_z;

                    let opaque = opaque_mask[area_yz];
                    let unpad_opaque = opaque & UNPADDED_MASK;
                    let unpad_transparent = transparent_mask[area_yz] & UNPADDED_MASK;

                    if unpad_opaque == 0 && unpad_transparent == 0 {
                        continue;
                    }

                    let adj_opaque = match face {
                        PosX => opaque >> 1,
                        NegX => opaque << 1,
                        PosY => opaque_mask[area_yz + STRIDE_0],
                        NegY => opaque_mask[area_yz - STRIDE_0],
                        PosZ => opaque_mask[area_yz + STRIDE_1],
                        NegZ => opaque_mask[area_yz - STRIDE_1],
                    };

                    visible_mask[area_yz] = unpad_opaque & !adj_opaque;

                    let mut visible_transparent = unpad_transparent & !adj_opaque;

                    while visible_transparent != 0 {
                        let x = visible_transparent.trailing_zeros() as usize;
                        visible_transparent &= visible_transparent - 1;

                        let vol_x = x << SHIFT_0;
                        let vol_xyz = vol_x | vol_yz;

                        let voxel_opt = voxel_opts[vol_xyz];

                        let adj_index = (vol_xyz as isize + vol_adj_offset) as usize;
                        let adj_voxel_opt = voxel_opts[adj_index];

                        visible_mask[area_yz] |= ((voxel_opt != adj_voxel_opt) as u64) << x;
                    }
                }
            }
        }
    }

    fn face_merging(
        &mut self,
        voxel_opts: &[Option<Voxel>; VOL],
        chunk_origin: IVec3,
    ) -> VoxelQuadOffsets {
        let mut offsets = [0; 7];

        for (index, face) in [PosX, PosY, PosZ, NegX, NegY, NegZ].into_iter().enumerate() {
            let visible_mask = &self.visible_masks[face];

            for z in 1..LEN - 1 {
                let vol_z = z << SHIFT_2;

                let area_z = z << SHIFT_1;

                for y in 1..LEN - 1 {
                    let vol_y = y << SHIFT_1;
                    let vol_yz = vol_y | vol_z;

                    let area_y = y << SHIFT_0;
                    let area_yz = area_y | area_z;

                    let mut column = visible_mask[area_yz];
                    if column == 0 {
                        continue;
                    }

                    match face {
                        PosX | NegX => {
                            let upward_column = visible_mask[area_yz + STRIDE_0];

                            let forward_column = visible_mask[area_yz + STRIDE_1];

                            while column != 0 {
                                let x = column.trailing_zeros() as usize;
                                column &= column - 1;

                                let vol_x = x << SHIFT_0;
                                let vol_xyz = vol_x | vol_yz;

                                let vol_xy = vol_x | vol_y;

                                let voxel_opt = voxel_opts[vol_xyz];
                                let voxel = voxel_opt.unwrap();

                                if self.upward_merged[vol_x] == 0
                                    && (forward_column >> x) & 1 != 0
                                    && voxel_opt == voxel_opts[vol_xyz + STRIDE_2]
                                {
                                    self.forward_merged[vol_xy] += 1;
                                    continue;
                                }

                                if (upward_column >> x) & 1 != 0
                                    && self.forward_merged[vol_xy]
                                        == self.forward_merged[vol_xy + STRIDE_1]
                                    && voxel_opt == voxel_opts[vol_xyz + STRIDE_1]
                                {
                                    self.forward_merged[vol_xy] = 0;
                                    self.upward_merged[vol_x] += 1;
                                    continue;
                                }

                                let w = self.forward_merged[vol_xy] as u32;
                                let h = self.upward_merged[vol_x] as u32 + 1;

                                let x = x as i32;
                                let y = y as i32 - self.upward_merged[vol_x] as i32;
                                let z = z as i32 - self.forward_merged[vol_xy] as i32;

                                self.forward_merged[vol_xy] = 0;
                                self.upward_merged[vol_x] = 0;

                                let pos = chunk_origin + IVec3::new(x, y, z);
                                let texture_index = voxel.textures()[face];

                                let quad = VoxelQuad::new(pos, texture_index, w, h, face);
                                self.quads.push(quad);
                            }
                        }
                        PosY | NegY => {
                            let forward_column = visible_mask[area_yz + STRIDE_1];

                            while column != 0 {
                                let x = column.trailing_zeros() as usize;

                                let vol_x = x << SHIFT_0;
                                let vol_xyz = vol_x | vol_yz;

                                let vol_xy = vol_x | vol_y;

                                let voxel_opt = voxel_opts[vol_xyz];
                                let voxel = voxel_opt.unwrap();

                                if (forward_column >> x) & 1 != 0
                                    && voxel_opt == voxel_opts[vol_xyz + STRIDE_2]
                                {
                                    self.forward_merged[vol_xy] += 1;
                                    column &= column - 1;
                                    continue;
                                }

                                let mut right_merged = 1;
                                for right in (x + 1)..LEN - 1 {
                                    let r_vol_x = right << SHIFT_0;
                                    let r_vol_xy = r_vol_x | vol_y;

                                    if (column >> right) & 1 == 0
                                        || self.forward_merged[vol_xy]
                                            != self.forward_merged[r_vol_xy]
                                        || voxel_opt != voxel_opts[r_vol_xy | vol_z]
                                    {
                                        break;
                                    }
                                    self.forward_merged[r_vol_xy] = 0;
                                    right_merged += 1;
                                }
                                let cleared = x + right_merged;
                                column &= !((1 << cleared) - 1);

                                let w = right_merged as u32;
                                let h = self.forward_merged[vol_xy] as u32 + 1;

                                let x = x as i32;
                                let y = y as i32;
                                let z = z as i32 - self.forward_merged[vol_xy] as i32;

                                self.forward_merged[vol_xy] = 0;

                                let pos = chunk_origin + IVec3::new(x, y, z);
                                let texture_index = voxel.textures()[face];

                                let quad = VoxelQuad::new(pos, texture_index, w, h, face);
                                self.quads.push(quad);
                            }
                        }
                        PosZ | NegZ => {
                            let upward_column = visible_mask[area_yz + STRIDE_0];

                            while column != 0 {
                                let x = column.trailing_zeros() as usize;

                                let vol_x = x << SHIFT_0;
                                let vol_xyz = vol_x | vol_yz;

                                let voxel_opt = voxel_opts[vol_xyz];
                                let voxel = voxel_opt.unwrap();

                                if (upward_column >> x) & 1 != 0
                                    && voxel_opt == voxel_opts[vol_xyz + STRIDE_1]
                                {
                                    self.upward_merged[vol_x] += 1;
                                    column &= column - 1;
                                    continue;
                                }

                                let mut right_merged = 1;
                                for right in (x + 1)..LEN - 1 {
                                    if (column >> right) & 1 == 0
                                        || self.upward_merged[vol_x] != self.upward_merged[right]
                                        || voxel_opt != {
                                            let vol_x = right << SHIFT_0;
                                            let vol_xyz = vol_x | vol_yz;
                                            voxel_opts[vol_xyz]
                                        }
                                    {
                                        break;
                                    }
                                    self.upward_merged[right] = 0;
                                    right_merged += 1;
                                }
                                let cleared = x + right_merged;
                                column &= !((1 << cleared) - 1);

                                let w = right_merged as u32;
                                let h = self.upward_merged[vol_x] as u32 + 1;

                                let x = x as i32;
                                let y = y as i32 - self.upward_merged[vol_x] as i32;
                                let z = z as i32;

                                self.upward_merged[vol_x] = 0;

                                let pos = chunk_origin + IVec3::new(x, y, z);
                                let texture_index = voxel.textures()[face];

                                let quad = VoxelQuad::new(pos, texture_index, w, h, face);
                                self.quads.push(quad);
                            }
                        }
                    }
                }
            }
            offsets[index + 1] = self.quads.len() as u32;
        }

        VoxelQuadOffsets(offsets)
    }

    pub fn mesh(&mut self, chunk: &Chunk, chunk_pos: IVec3) -> (&[VoxelQuad], VoxelQuadOffsets) {
        let Chunk {
            voxel_opts,
            opaque_mask,
            transparent_mask,
        } = chunk;

        let chunk_origin = chunk_origin(chunk_pos);

        self.face_culling(voxel_opts, opaque_mask, transparent_mask);

        let voxel_quad_offsets = self.face_merging(voxel_opts, chunk_origin);

        (&self.quads, voxel_quad_offsets)
    }
}

// This can be aligned to 8 bytes instead of 16 bytes by
// storing the `ChunkOffset` (`U6Vec3`) and a `chunk_index`
// (`u16`) that points to a `ChunkPos` (`I26Vec3`) in a
// storage buffer instead of the full `VoxelPos` (`IVec3`).
// However at this point the complication is not worth it.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct VoxelQuad {
    pos: IVec3,
    data: u32,
}

impl VoxelQuad {
    #[inline]
    pub const fn new(pos: IVec3, texture_index: u32, w: u32, h: u32, face: Face) -> Self {
        // this must match the shader
        let face = match face {
            PosX => 0,
            PosY => 1,
            PosZ => 2,
            NegX => 3,
            NegY => 4,
            NegZ => 5,
        };

        Self {
            pos,
            data: face << 28 | h << 22 | w << 16 | texture_index,
        }
    }
}

pub struct VoxelQuadOffsets([u32; 7]);

impl VoxelQuadOffsets {
    pub fn range(&self, face: Face) -> Range<u32> {
        // must match the ordering in `face_merging`
        match face {
            PosX => self.0[0]..self.0[1],
            PosY => self.0[1]..self.0[2],
            PosZ => self.0[2]..self.0[3],
            NegX => self.0[3]..self.0[4],
            NegY => self.0[4]..self.0[5],
            NegZ => self.0[5]..self.0[6],
        }
    }

    pub fn shift(&mut self, shift: u32) {
        for offset in &mut self.0 {
            *offset += shift
        }
    }
}
