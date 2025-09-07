use bevy::math::IVec3;
use enum_map::enum_map;

use crate::{
    block_library::BlockLibrary, math::signed_axis::*, voxel::Voxel
};

use super::{
    Chunk, VoxelQuad, ChunkMesh, ChunkPos, 
    padded::{AREA, LEN, VOL},
    padded::{
        X_SHIFT as SHIFT_0, X_STRIDE as STRIDE_0, Y_SHIFT as SHIFT_1, Y_STRIDE as STRIDE_1,
        Z_SHIFT as SHIFT_2, Z_STRIDE as STRIDE_2,
    },
};

// vol_*: X - SHIFT_0, Y - SHIFT_1, Z - SHIFT_2
// area_*: Y - SHIFT_0, Z - SHIFT_1,
// * is replaced by whichever axes are present

const UNPADDED_MASK: u64 = !(1 << 63 | 1);

pub struct Mesher {
    quads: Vec<VoxelQuad>,
    visible_masks: Box<SignedAxisMap<[u64; AREA]>>,
    upward_merged: Box<[u8; LEN]>,
    forward_merged: Box<[u8; AREA]>, // atm up and for can be merged
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
        voxels: &[Voxel; VOL],
        opaque_mask: &[u64; AREA],
        transparent_mask: &[u64; AREA],
    ) {
        for signed_axis in SignedAxis::ALL {
            let visible_mask = &mut self.visible_masks[signed_axis];

            let vol_adj_offset = match signed_axis {
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

                    let adj_opaque = match signed_axis {
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

                        let voxel = voxels[vol_xyz];

                        let adj_index = (vol_xyz as isize + vol_adj_offset) as usize;
                        let adj_voxel = voxels[adj_index];

                        visible_mask[area_yz] |= ((voxel != adj_voxel) as u64) << x;
                    }
                }
            }
        }
    }

    fn face_merging(
        &mut self,
        voxels: &[Voxel; VOL],
        offset: IVec3,
        block_library: &BlockLibrary,
    ) -> ChunkMesh {
        let mut offsets = [0; 7];

        for (index, signed_axis) in [PosX, PosY, PosZ, NegX, NegY, NegZ].into_iter().enumerate() {
            let visible_mask = &self.visible_masks[signed_axis];

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

                    match signed_axis {
                        PosX | NegX => {
                            let upward_column = visible_mask[area_yz + STRIDE_0];
                    
                            let forward_column = visible_mask[area_yz + STRIDE_1];

                            while column != 0 {
                                let x = column.trailing_zeros() as usize;
                                column &= column - 1;

                                let vol_x = x << SHIFT_0;
                                let vol_xyz = vol_x | vol_yz;

                                let vol_xy = vol_x | vol_y;

                                let voxel = voxels[vol_xyz];

                                if self.upward_merged[x] == 0 && (forward_column >> x) & 1 != 0 && voxel == voxels[vol_xyz + STRIDE_2] {
                                    self.forward_merged[vol_xy] += 1;
                                    continue;
                                }

                                if (upward_column >> x) & 1 != 0
                                    && self.forward_merged[vol_xy] == self.forward_merged[vol_xy + STRIDE_1]
                                    && voxel == voxels[vol_xyz + STRIDE_1]
                                {
                                    self.forward_merged[vol_xy] = 0;
                                    self.upward_merged[x] += 1;
                                    continue;
                                }

                                let w = self.forward_merged[vol_xy] as u32;
                                let h = self.upward_merged[x] as u32;

                                self.forward_merged[vol_xy] = 0;
                                self.upward_merged[x] = 0;

                                let x = x as i32; // TODO
                                let y = y as i32;
                                let z = z as i32 - h as i32;

                                let pos = offset + IVec3::new(x, y, z);

                                let texture_index = block_library[voxel].textures[signed_axis];

                                let quad = VoxelQuad::new(pos, w, h + 1, signed_axis, texture_index);

                                self.quads.push(quad);
                            }
                        },
                        PosY | NegY => {
                            let forward_column = visible_mask[area_yz + STRIDE_1];

                            while column != 0 {
                                let x = column.trailing_zeros() as usize;

                                let vol_x = x << SHIFT_0;
                                let vol_xyz = vol_x | vol_yz;

                                let vol_xy = vol_x | vol_y;

                                let voxel = voxels[vol_xyz];

                                if (forward_column >> x) & 1 != 0 && voxel == voxels[vol_xyz + STRIDE_2] {
                                    self.forward_merged[vol_xy] += 1;
                                    column &= column - 1;
                                    continue;
                                }

                                let mut right_merged = 1;
                                for right in (x + 1)..LEN - 1 {
                                    let r_vol_x = right << SHIFT_0;
                                    let r_vol_xy = r_vol_x | vol_y;

                                    if (column >> right) & 1 == 0
                                        || self.forward_merged[vol_xy] != self.forward_merged[r_vol_xy]
                                        || voxel != voxels[r_vol_xy | vol_z]
                                    {
                                        break;
                                    }
                                    self.forward_merged[r_vol_xy] = 0;
                                    right_merged += 1;
                                }
                                let cleared = x + right_merged;
                                column &= !((1 << cleared) - 1);

                                let w = right_merged as u32;
                                let h = self.forward_merged[vol_xy] as u32;

                                self.forward_merged[vol_xy] = 0;

                                let x = x as i32;
                                let y = y as i32;
                                let z = z as i32 - h as i32;

                                let pos = offset + IVec3::new(x, y, z);

                                let texture_index = block_library[voxel].textures[signed_axis];

                                let quad = VoxelQuad::new(pos, w, h + 1, signed_axis, texture_index);

                                self.quads.push(quad);
                            }
                        },
                        PosZ | NegZ => {
                            let upward_column = visible_mask[area_yz + STRIDE_0];

                            while column != 0 {
                                let x = column.trailing_zeros() as usize;

                                let vol_x = x << SHIFT_0;
                                let vol_xyz = vol_x | vol_yz;

                                let voxel = voxels[vol_xyz];

                                if (upward_column >> x) & 1 != 0 && voxel == voxels[vol_xyz + STRIDE_1] {
                                    self.upward_merged[x] += 1;
                                    column &= column - 1;
                                    continue;
                                }

                                let mut right_merged = 1;
                                for right in (x + 1)..LEN - 1 {
                                    if (column >> right) & 1 == 0
                                        || self.upward_merged[x] != self.upward_merged[right]
                                        || voxel != {
                                            let vol_x = right << SHIFT_0;
                                            let vol_xyz = vol_x | vol_yz;
                                            voxels[vol_xyz]
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
                                let h = self.upward_merged[x] as u32;

                                self.upward_merged[x] = 0;

                                let x = x as i32;
                                let y = y as i32 - h as i32;
                                let z = z as i32;

                                let pos = offset + IVec3::new(x, y, z);

                                let texture_index = block_library[voxel].textures[signed_axis];

                                let quad = VoxelQuad::new(pos, w, h + 1, signed_axis, texture_index);

                                self.quads.push(quad);
                            }
                        }
                    }
                }
            }
            offsets[index + 1] = self.quads.len() as u32;
        }

        ChunkMesh { offsets }
    }

    pub fn mesh(
        &mut self,
        Chunk {
            voxels,
            opaque_mask,
            transparent_mask,
        }: &Chunk,
        chunk_pos: ChunkPos,
        block_library: &BlockLibrary,
    ) -> (&[VoxelQuad], ChunkMesh) {
        self.face_culling(voxels, opaque_mask, transparent_mask);
        let chunk_mesh = self.face_merging(voxels, chunk_pos.as_voxel(), block_library);
        (&self.quads, chunk_mesh)
    }
}

// pub fn compute_opaque_mask(
//     voxels: &[Voxel; PADDED_CHUNK_VOLUME],
//     transparents: &BTreeSet<Voxel>,
// ) -> [u64; PADDED_CHUNK_AREA] {
//     let mut opaque_mask = [0; PADDED_CHUNK_AREA];

//     for (i, voxel) in voxels.iter().enumerate() {
//         if voxel.is_sentinel() || transparents.contains(voxel) {
//             continue;
//         }

//         let row = i / PADDED_CHUNK_LENGTH;
//         let bit = i % PADDED_CHUNK_LENGTH;

//         opaque_mask[row] |= 1 << bit;
//     }

//     opaque_mask
// }

// pub fn compute_transparent_mask(
//     voxels: &[Voxel; PADDED_CHUNK_VOLUME],
//     transparents: &BTreeSet<Voxel>,
// ) -> [u64; PADDED_CHUNK_AREA] {
//     let mut transparent_mask = [0; PADDED_CHUNK_AREA];

//     for (i, voxel) in voxels.iter().enumerate() {
//         if voxel.is_sentinel() || !transparents.contains(voxel) {
//             continue;
//         }

//         let row = i / PADDED_CHUNK_LENGTH;
//         let bit = i % PADDED_CHUNK_LENGTH;

//         transparent_mask[row] |= 1 << bit;
//     }

//     transparent_mask
// }