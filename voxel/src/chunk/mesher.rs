use ndshape::{ConstShape, ConstShape3u32};
use std::{array, collections::BTreeSet};

use crate::{math::{Axis, Sign, SignedAxis}, voxel::Voxel};

use super::{
    CHUNK_AREA, CHUNK_LENGTH, CHUNK_VOLUME, PADDED_CHUNK_AREA, PADDED_CHUNK_LENGTH,
    PADDED_CHUNK_VOLUME, X_SHIFT, X_STRIDE, Y_SHIFT, Y_STRIDE, Z_SHIFT, Z_STRIDE
};

type LayerShape = ConstShape3u32<6, PADDED_CHUNK_LENGTH, PADDED_CHUNK_LENGTH>;
const LAYER_SHAPE: LayerShape = LayerShape {};

const LAYER_L_STRIDE: u32 = LayerShape::STRIDES[0];
const LAYER_X_STRIDE: u32 = LayerShape::STRIDES[1];
const LAYER_Y_STRIDE: u32 = LayerShape::STRIDES[2];

const UNPADDED_MASK: u64 = 0x7FFF_FFFF_FFFF_FFFE;

impl SignedAxis {
    const fn stride_offset(&self, base: u32) -> u32 {
        let (sign, axis) = self.split();
        let unsigned_stride = match axis {
            Axis::X => X_STRIDE,
            Axis::Y => Y_STRIDE,
            Axis::Z => Z_STRIDE,
        };

        match sign {
            Sign::Pos => base + unsigned_stride,
            Sign::Neg => base - unsigned_stride,
        }
    }
}

pub struct Mesher {
    pub quads: [Vec<VoxelQuad>; 6],
    face_masks: Box<[u64; LayerShape::SIZE as usize]>,
    upward_merged: Box<[u8; PADDED_CHUNK_AREA as usize]>,
    right_merged: Box<[u8; PADDED_CHUNK_AREA as usize]>,
}

impl Mesher {
    pub fn new() -> Self {
        Self {
            quads: array::from_fn(|_| Vec::new()),
            face_masks: Box::new([0; LayerShape::SIZE as usize]),
            upward_merged: Box::new([0; PADDED_CHUNK_AREA as usize]),
            right_merged: Box::new([0; PADDED_CHUNK_AREA as usize]),
        }
    }

    pub fn clear(&mut self) {
        self.face_masks.fill(0);
        self.upward_merged.fill(0);
        self.right_merged.fill(0);
        for vec in &mut self.quads {
            vec.clear();
        }
    }

    fn face_culling(
        &mut self,
        voxels: &[Voxel; PADDED_CHUNK_VOLUME as usize],
        transparents: &BTreeSet<Voxel>,
    ) {
        for z in 1..(PADDED_CHUNK_LENGTH - 1) {
            let z_stride = z << Z_SHIFT;
            for y in 1..(PADDED_CHUNK_LENGTH - 1) {
                let y_stride = y << Y_SHIFT;
                let zy_stride = z_stride + y_stride;

                let layer_y_stride = y * LAYER_Y_STRIDE;

                for x in 1..(PADDED_CHUNK_LENGTH - 1) {
                    let x_stride = x << X_SHIFT;
                    let zyx_stride = zy_stride + x_stride;

                    let layer_x_stride = x * LAYER_X_STRIDE;
                    let layer_yx_stride = layer_y_stride + layer_x_stride;

                    let voxel = voxels[zyx_stride as usize];
                    if voxel.is_sentinel() {
                        continue;
                    }

                    for signed_axis in SignedAxis::ALL {
                        let index = signed_axis.stride_offset(zyx_stride) as usize;
                        let neighbor = voxels[index];

                        let layer_l_stride = signed_axis.as_usize() * LAYER_L_STRIDE as usize;
                        let layer_yxl_stride = layer_yx_stride as usize + layer_l_stride;

                        self.face_masks[layer_yxl_stride] |=
                            (is_visible(voxel, neighbor, transparents) as u64) << z;
                    }
                }
            }
        }
    }

    fn fast_face_culling(
        &mut self,
        voxels: &[Voxel; PADDED_CHUNK_VOLUME as usize],
        opaque_mask: &[u64; PADDED_CHUNK_AREA as usize],
        transparent_mask: &[u64; PADDED_CHUNK_AREA as usize],
    ) {
        for y in 1..(PADDED_CHUNK_LENGTH - 1) {
            let y_stride = y << Y_SHIFT;

            let layer_y_stride = y * LAYER_Y_STRIDE;

            for x in 1..(PADDED_CHUNK_LENGTH - 1) {
                let x_stride = x << X_SHIFT;
                let yx_stride = y_stride + x_stride;

                let layer_x_stride = x * LAYER_X_STRIDE; 
                let layer_yx_stride = layer_y_stride + layer_x_stride;

                let column_index = yx_stride as usize;

                let column = opaque_mask[column_index];
                let unpadded_column = column & UNPADDED_MASK;
                if unpadded_column == 0 {
                    continue;
                }

                for signed_axis in SignedAxis::ALL {
                    let (sign, axis) = signed_axis.split();
                    let adjacent_column = match axis {
                        Axis::X => {
                            let index = match sign {
                                Sign::Pos => yx_stride + X_STRIDE,
                                Sign::Neg => yx_stride - X_STRIDE,
                            };

                            opaque_mask[index as usize]
                        },
                        Axis::Y => {
                            let index = match sign {
                                Sign::Pos => yx_stride + Y_STRIDE,
                                Sign::Neg => yx_stride - Y_STRIDE,
                            };

                            opaque_mask[index as usize]
                        },
                        Axis::Z => {
                            match sign {
                                Sign::Pos => column << 1,
                                Sign::Neg => column >> 1,
                            }
                        },
                    };

                    let layer_l_stride = signed_axis.as_usize() * LAYER_L_STRIDE as usize;
                    let layer_yxl_stride = layer_l_stride + layer_yx_stride as usize;

                    self.face_masks[layer_yxl_stride] = column & adjacent_column; 
                }

                let mut transparent = transparent_mask[column_index] & UNPADDED_MASK;
                while transparent != 0 {
                    let z = transparent.trailing_zeros();
                    transparent &= !(1 << z);

                    let z_stride = z << Z_SHIFT;
                    let zyx_stride = yx_stride + z_stride;
                    let voxel = voxels[zyx_stride as usize];

                    for signed_axis in SignedAxis::ALL {
                        let (sign, axis) = signed_axis.split();
                        let unsigned_stride = match axis {
                            Axis::X => X_STRIDE,
                            Axis::Y => Y_STRIDE,
                            Axis::Z => Z_STRIDE,
                        };

                        let neighbor_index = match sign {
                            Sign::Pos => zyx_stride + unsigned_stride,
                            Sign::Neg => zyx_stride - unsigned_stride,
                        };
                        let neighbor = voxels[neighbor_index as usize];

                        let layer_l_stride = signed_axis.as_usize() * LAYER_L_STRIDE as usize;
                        let layer_yxl_stride = layer_l_stride + layer_yx_stride as usize;

                        self.face_masks[layer_yxl_stride] |=
                            ((voxel != neighbor) as u64) << z;
                    }
                }
            }
        }
    }

    fn face_merging(&mut self, voxels: &[Voxel; PADDED_CHUNK_VOLUME as usize]) {
        for signed_axis in SignedAxis::ALL {
            
        }
        
        for signed_axis in [
            SignedAxis::PosX,
            SignedAxis::NegX,
            SignedAxis::PosY,
            SignedAxis::NegY,
        ] {
            let permutation = AxisPermutation::even(signed_axis.as_unsigned());
            let axis_offset = signed_axis.as_index() * CHUNK_AREA;

            for layer_pos in 0..CHUNK_LENGTH {
                let layer_index = layer_pos * CHUNK_LENGTH + axis_offset;

                for column_pos in 0..CHUNK_LENGTH {
                    let column_index = column_pos + layer_index;

                    let mut column = self.face_masks[column_index];
                    if column == 0 {
                        continue;
                    }

                    let up_column = if column_pos + 1 < CHUNK_LENGTH {
                        self.face_masks[column_index + 1]
                    } else {
                        0
                    };

                    let mut right_merged = 1;
                    while column != 0 {
                        let bit_pos = column.trailing_zeros() as usize;

                        let voxel_index = permutation.linearize_cubic::<CHUNK_LENGTH>(
                            layer_pos + 1,
                            column_pos + 1,
                            bit_pos + 1,
                        );
                        let voxel = voxels[voxel_index];

                        if (up_column >> bit_pos) != 0 {
                            let up_voxel_index = permutation.linearize_cubic::<CHUNK_LENGTH>(
                                layer_pos + 1,
                                column_pos + 2,
                                bit_pos + 1,
                            );
                            let up_voxel = voxels[up_voxel_index];

                            if up_voxel == voxel {
                                self.upward_merged[bit_pos] += 1;
                                column &= !(1 << bit_pos);
                                continue;
                            }
                        }

                        let right_voxel_index = permutation.linearize_cubic::<CHUNK_LENGTH>(
                            layer_pos + 2,
                            column_pos + 1,
                            bit_pos + 1,
                        );
                        let right_voxel = voxels[right_voxel_index];
                        for right in bit_pos..CHUNK_LENGTH {
                            if (column >> right) & 1 == 0
                                || self.upward_merged[bit_pos] != self.upward_merged[right]
                                || voxel != right_voxel
                            {
                                break;
                            }
                            self.upward_merged[right] = 0;
                            right_merged += 1;
                        }
                        column &= !((1 << (bit_pos + right_merged)) - 1);

                        let mesh_x = layer_pos + signed_axis.is_positive() as usize;
                        let mesh_y = column_pos - self.upward_merged[bit_pos] as usize;
                        let mesh_z = bit_pos;

                        let mesh_w = right_merged;
                        let mesh_h = (self.upward_merged[bit_pos] + 1) as usize;

                        right_merged = 1;
                        self.upward_merged[bit_pos] = 0;

                        let quad = match signed_axis {
                            SignedAxis::PosX | SignedAxis::NegY => VoxelQuad::new(
                                mesh_x,
                                mesh_y,
                                mesh_z,
                                mesh_w,
                                mesh_h,
                                voxel.id as usize,
                            ),
                            SignedAxis::NegX => VoxelQuad::new(
                                mesh_x + mesh_h,
                                mesh_y,
                                mesh_z,
                                mesh_w,
                                mesh_h,
                                voxel.id as usize,
                            ),
                            SignedAxis::PosY => VoxelQuad::new(
                                mesh_x,
                                mesh_y + mesh_h,
                                mesh_z,
                                mesh_w,
                                mesh_h,
                                voxel.id as usize,
                            ),
                            _ => unreachable!(),
                        };

                        self.quads[signed_axis.as_index()].push(quad);
                    }
                }
            }
        }

        for signed_axis in [SignedAxis::PosZ, SignedAxis::NegZ] {
            let permutation = AxisPermutation::even(signed_axis.as_unsigned());
            let axis_offset = signed_axis.as_index() * CHUNK_AREA;

            for layer_pos in 0..CHUNK_LENGTH {
                let layer_index = layer_pos * CHUNK_LENGTH + axis_offset;
                let next_layer_index = (layer_pos + 1) * CHUNK_LENGTH + axis_offset;

                for column_pos in 0..CHUNK_LENGTH {
                    let mut column = self.face_masks[column_pos + layer_index];
                    if column == 0 {
                        continue;
                    }

                    let upward_column = if layer_pos + 1 < CHUNK_LENGTH {
                        self.face_masks[column_pos + next_layer_index]
                    } else {
                        0
                    };

                    let right_column = if column_pos + 1 < CHUNK_LENGTH {
                        self.face_masks[column_pos + layer_index + 1]
                    } else {
                        0
                    };

                    let right_size = column_pos * CHUNK_LENGTH;

                    while column != 0 {
                        let bit_pos = column.trailing_zeros() as usize;

                        column &= !(1 << bit_pos);

                        let voxel_index = permutation.linearize_cubic::<CHUNK_LENGTH>(
                            column_pos + 1,
                            layer_pos + 1,
                            bit_pos,
                        );
                        let voxel = voxels[voxel_index];

                        let upward_index = right_size + (bit_pos - 1);
                        let right_merged_ref = &mut self.right_merged[bit_pos - 1];

                        let right_voxel_index = permutation.linearize_cubic::<CHUNK_LENGTH>(
                            column_pos + 1,
                            layer_pos + 2,
                            bit_pos,
                        );
                        let right_voxel = voxels[right_voxel_index];

                        if *right_merged_ref == 0
                            && (upward_column >> bit_pos) & 1 != 0
                            && voxel == right_voxel
                        {
                            self.upward_merged[upward_index] += 1;
                            continue;
                        }

                        let next_upward_index = upward_index + CHUNK_LENGTH;

                        let next_voxel_index = permutation.linearize_cubic::<CHUNK_LENGTH>(
                            column_pos + 2,
                            layer_pos + 1,
                            bit_pos,
                        );
                        let next_voxel = voxels[next_voxel_index];

                        if (right_column >> bit_pos) & 1 != 0
                            && self.upward_merged[upward_index]
                                == self.upward_merged[next_upward_index]
                            && voxel == next_voxel
                        {
                            self.upward_merged[upward_index] = 0;
                            *right_merged_ref += 1;
                            continue;
                        }

                        let mesh_y = layer_pos - self.upward_merged[upward_index] as usize;
                        let mesh_z = bit_pos - 1 + signed_axis.is_positive() as usize;

                        let mesh_w = 1 + *right_merged_ref;
                        let mesh_h = 1 + self.upward_merged[upward_index];

                        let mesh_x = column_pos - *right_merged_ref as usize
                            + match signed_axis {
                                SignedAxis::PosZ => mesh_w as usize,
                                SignedAxis::NegZ => 0,
                                _ => unreachable!(),
                            };

                        self.upward_merged[upward_index] = 0;
                        *right_merged_ref = 0;

                        let quad = VoxelQuad::new(
                            mesh_x,
                            mesh_y,
                            mesh_z,
                            mesh_w as usize,
                            mesh_h as usize,
                            voxel.id as usize,
                        );
                        self.quads[signed_axis.as_index()].push(quad);
                    }
                }
            }
        }
    }

    pub fn fast_mesh(
        &mut self,
        voxels: &[Voxel; CHUNK_VOLUME],
        opaque_mask: &[u64; CHUNK_AREA],
        transparent_mask: &[u64; CHUNK_AREA],
    ) {
        self.fast_face_culling(voxels, opaque_mask, transparent_mask);
        self.face_merging(voxels);
    }

    pub fn mesh(&mut self, voxels: &[Voxel; CHUNK_VOLUME], transparents: &BTreeSet<Voxel>) {
        self.face_culling(voxels, transparents);
        self.face_merging(voxels);
    }
}

#[derive(Debug, Clone, Copy, Deref, DerefMut)]
// TODO: switch to a single u32
pub struct VoxelQuad(u64);

impl VoxelQuad {
    #[inline]
    pub const fn new(x: usize, y: usize, z: usize, w: usize, h: usize, voxel: usize) -> Self {
        Self(
            (voxel as u64) << 32
                | (h as u64) << 24
                | (w as u64) << 18
                | (z as u64) << 12
                | (y as u64) << 6
                | x as u64,
        )
    }
}

#[inline]
fn is_visible(voxel: Voxel, neighbor: Voxel, transparents: &BTreeSet<Voxel>) -> bool {
    neighbor.is_sentinel() || (voxel != neighbor && transparents.contains(&neighbor))
}
