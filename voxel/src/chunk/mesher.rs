use ndshape::ConstPow2Shape3u32;
use std::{array, collections::BTreeSet};

use crate::voxel::Voxel;

use super::{
    BITS, ChunkShape, PADDED_CHUNK_AREA, PADDED_CHUNK_LENGTH, PADDED_CHUNK_VOLUME, VoxelQuad,
    X_SHIFT, X_STRIDE, Y_SHIFT, Y_STRIDE, Z_SHIFT, Z_STRIDE,
};

type LayerShape = ConstPow2Shape3u32<BITS, BITS, 3>;

const LAYER_Y_SHIFT: usize = LayerShape::SHIFTS[0] as usize;
const LAYER_Z_SHIFT: usize = LayerShape::SHIFTS[1] as usize;
const LAYER_L_SHIFT: usize = LayerShape::SHIFTS[2] as usize;

const LAYER_Y_STRIDE: usize = 1 << LAYER_Y_SHIFT;
const LAYER_Z_STRIDE: usize = 1 << LAYER_Z_SHIFT;

const UNPADDED_MASK: u64 = 0x7FFF_FFFF_FFFF_FFFE;

pub struct Mesher {
    // TODO: use one continuous buffer with offset indices.
    pub mesh: [Vec<VoxelQuad>; 6],
    face_masks: Box<[u64; PADDED_CHUNK_AREA * 6]>,
    upward_merged: Box<[u8; PADDED_CHUNK_AREA]>,
    forward_merged: Box<[u8; PADDED_CHUNK_LENGTH]>,
}

impl Mesher {
    pub fn new() -> Self {
        Self {
            mesh: array::from_fn(|_| Vec::new()),
            face_masks: Box::new([0; PADDED_CHUNK_AREA * 6]),
            forward_merged: Box::new([0; PADDED_CHUNK_LENGTH]),
            upward_merged: Box::new([0; PADDED_CHUNK_AREA]),
        }
    }

    pub fn clear(&mut self) {
        self.face_masks.fill(0);
        self.forward_merged.fill(0);
        self.upward_merged.fill(0);
        for face in self.mesh.iter_mut() {
            face.clear();
        }
    }

    fn face_culling(
        &mut self,
        voxels: &[Voxel; PADDED_CHUNK_VOLUME],
        transparents: &BTreeSet<Voxel>, // TODO: FIGURE OUT BEST STRUCT FOR THIS
    ) {
        for face in SignedAxis::ALL {
            let layer_l_stride = face.into_usize() << LAYER_L_SHIFT;

            for z in 1..(PADDED_CHUNK_LENGTH - 1) {
                let z_stride = z << Z_SHIFT;

                let layer_z_stride = z << LAYER_Z_SHIFT;
                let layer_zl_stride = layer_z_stride + layer_l_stride;

                for y in 1..(PADDED_CHUNK_LENGTH - 1) {
                    let y_stride = y << Y_SHIFT;
                    let yz_stride = z_stride + y_stride;

                    let layer_y_stride = y << LAYER_Y_SHIFT;
                    let layer_yzl_stride = layer_y_stride + layer_zl_stride;

                    for x in 1..(PADDED_CHUNK_LENGTH - 1) {
                        let x_stride = x << X_SHIFT;
                        let xyz_stride = yz_stride + x_stride;

                        let voxel = voxels[xyz_stride];
                        if voxel.is_sentinel() {
                            continue;
                        }

                        let neighbor_index = offset_stride(face, xyz_stride);
                        let neighbor = voxels[neighbor_index];

                        self.face_masks[layer_yzl_stride] |=
                            is_visible_as_u64(voxel, neighbor, transparents) << z;
                    }
                }
            }
        }
    }

    fn fast_face_culling(
        &mut self,
        voxels: &[Voxel; PADDED_CHUNK_VOLUME],
        opaque_mask: &[u64; PADDED_CHUNK_AREA],
        transparent_mask: &[u64; PADDED_CHUNK_AREA],
    ) {
        for face in SignedAxis::ALL {
            let layer_l_stride = face.into_usize() << LAYER_L_SHIFT;

            let (sign, axis) = face.split();

            for z in 1..(PADDED_CHUNK_LENGTH - 1) {
                let z_stride = z << Z_SHIFT;

                let layer_z_stride = z << LAYER_Z_SHIFT;
                let layer_zl_stride = layer_z_stride + layer_l_stride;

                for y in 1..(PADDED_CHUNK_LENGTH - 1) {
                    let y_stride = y << Y_SHIFT;
                    let yz_stride = y_stride + z_stride;

                    let layer_y_stride = y << LAYER_Y_SHIFT;
                    let layer_yzl_stride = layer_y_stride + layer_zl_stride;

                    let column = opaque_mask[yz_stride];
                    let unpadded_column = column & UNPADDED_MASK;
                    let unpadded_transparent = transparent_mask[yz_stride] & UNPADDED_MASK;

                    if unpadded_column == 0 && unpadded_transparent == 0 {
                        continue;
                    }

                    let adjacent_column = match axis {
                        Axis::X => {
                            let index = match sign {
                                Sign::Pos => yz_stride + X_STRIDE,
                                Sign::Neg => yz_stride - X_STRIDE,
                            };

                            opaque_mask[index]
                        }
                        Axis::Y => {
                            let index = match sign {
                                Sign::Pos => yz_stride + Y_STRIDE,
                                Sign::Neg => yz_stride - Y_STRIDE,
                            };

                            opaque_mask[index]
                        }
                        Axis::Z => match sign {
                            Sign::Pos => column << 1,
                            Sign::Neg => column >> 1,
                        },
                    };

                    self.face_masks[layer_yzl_stride] = unpadded_column & !adjacent_column;

                    let mut visible_transparent = unpadded_transparent & !adjacent_column;
                    while visible_transparent != 0 {
                        let x = visible_transparent.trailing_zeros() as usize;
                        visible_transparent &= !(1 << x);

                        let x_stride = x << X_SHIFT;
                        let xyz_stride = yz_stride + x_stride;

                        let voxel = voxels[xyz_stride];

                        let neighbor_index = offset_stride(face, xyz_stride);
                        let neighbor = voxels[neighbor_index];

                        self.face_masks[layer_yzl_stride] |= ((voxel != neighbor) as u64) << x;
                    }
                }
            }
        }
    }

    fn face_merging(&mut self, voxels: &[Voxel; PADDED_CHUNK_VOLUME]) {
        for face in SignedAxis::ALL {
            let permutation = AxisPermutation::even(face.abs());
            let sigificance_map = permutation.sigificance_map();

            let face_index = face.into_usize();
            let layer_l_stride = face_index << LAYER_L_SHIFT;

            for z in 1..(PADDED_CHUNK_LENGTH - 1) {
                let z_significance = sigificance_map[2];
                let z_shift = ChunkShape::SHIFTS[z_significance];
                let z_stride_length = 1usize << z_shift;

                let z_stride = z << z_shift;

                let layer_z_stride = z << LAYER_Z_SHIFT;
                let layer_zl_stride = layer_l_stride + layer_z_stride;

                for y in 1..(PADDED_CHUNK_LENGTH - 1) {
                    let y_significance = sigificance_map[1];
                    let y_shift = ChunkShape::SHIFTS[y_significance];
                    let y_stride_length = 1usize << y_shift;

                    let y_stride = y << y_shift;
                    let yz_stride = y_stride + z_stride;

                    let layer_y_stride = y << LAYER_Y_SHIFT;
                    let layer_yzl_stride = layer_zl_stride + layer_y_stride;

                    use SignedAxis::*;
                    match face {
                        PosY | NegY | PosZ | NegZ => {
                            let mut column = self.face_masks[layer_yzl_stride];
                            if column == 0 {
                                continue;
                            }

                            let upward_column = if (y + 1) < (PADDED_CHUNK_LENGTH - 1) {
                                self.face_masks[layer_yzl_stride + LAYER_Y_STRIDE]
                            } else {
                                0
                            };

                            let mut forward_merged = 1;
                            while column != 0 {
                                let x = column.trailing_zeros() as usize;

                                let x_significance = sigificance_map[0];
                                let x_stride = x << ChunkShape::SHIFTS[x_significance];
                                let xyz_stride = x_stride + yz_stride;

                                let voxel = voxels[xyz_stride];

                                if (upward_column >> x) & 1 != 0
                                    && voxel == voxels[xyz_stride + y_stride_length]
                                {
                                    self.upward_merged[x] += 1;
                                    column &= !(1 << x);
                                    continue;
                                }

                                for forward in (x + 1)..(PADDED_CHUNK_LENGTH - 1) {
                                    if (column >> forward) & 1 == 0
                                        || self.forward_merged[x] != self.forward_merged[forward]
                                        || voxel != voxels[xyz_stride + z_stride_length]
                                    {
                                        break;
                                    }
                                    self.upward_merged[forward] = 0;
                                    forward_merged += 1;
                                }
                                column &= !((1 << (x + forward_merged)) - 1);

                                let upward_merged =
                                    std::mem::take(&mut self.upward_merged[x]) as usize;
                                forward_merged = 1;

                                let quad_y = y as u32 - 1 - upward_merged as u32;
                                let quad_x = x as u32 - 1;
                                let quad_z = z as u32 - 1 + face.is_positive() as u32;
                                let quad_w = forward_merged as u32;
                                let quad_h = upward_merged as u32 + 1;

                                let quad = match face {
                                    PosY => VoxelQuad::new(
                                        quad_x,
                                        quad_y,
                                        quad_z,
                                        quad_w,
                                        quad_h,
                                        voxel.id as u32,
                                    ),
                                    NegY => VoxelQuad::new(
                                        quad_x,
                                        quad_y + quad_h,
                                        quad_z,
                                        quad_w,
                                        quad_h,
                                        voxel.id as u32,
                                    ),
                                    PosZ => VoxelQuad::new(
                                        quad_y + quad_h,
                                        quad_x,
                                        quad_z,
                                        quad_w,
                                        quad_h,
                                        voxel.id as u32,
                                    ),
                                    NegZ => VoxelQuad::new(
                                        quad_y,
                                        quad_x,
                                        quad_z,
                                        quad_w,
                                        quad_h,
                                        voxel.id as u32,
                                    ),
                                    _ => unreachable!(),
                                };

                                self.mesh[face_index].push(quad);
                            }
                        }
                        PosX | NegX => {
                            let mut column = self.face_masks[layer_yzl_stride];
                            if column == 0 {
                                continue;
                            }

                            let upward_column = if (y + 1) < (PADDED_CHUNK_LENGTH - 1) {
                                self.face_masks[layer_yzl_stride + LAYER_Y_STRIDE]
                            } else {
                                0
                            };

                            let forward_column = if (z + 1) < (PADDED_CHUNK_LENGTH - 1) {
                                self.face_masks[layer_yzl_stride + LAYER_Z_STRIDE]
                            } else {
                                0
                            };

                            let forward_y_stride = y << Y_SHIFT;

                            while column != 0 {
                                let x = column.trailing_zeros() as usize;
                                column &= !(1 << x);

                                let x_significance = sigificance_map[0];
                                let x_shift = ChunkShape::SHIFTS[x_significance];

                                let x_stride = x << x_shift;
                                let xyz_stride = x_stride + yz_stride;

                                let forward_x_stride = x << X_SHIFT;
                                let forward_xy_stride = forward_y_stride + forward_x_stride;

                                let voxel = voxels[xyz_stride];

                                let forward_merged = &mut self.forward_merged[x];

                                if *forward_merged == 0
                                    && (upward_column >> x) & 1 != 0
                                    && voxel == voxels[xyz_stride + y_stride_length]
                                {
                                    self.upward_merged[forward_xy_stride] += 1;
                                    continue;
                                }

                                if (forward_column >> x) & 1 != 0
                                    && self.upward_merged[forward_xy_stride]
                                        == self.upward_merged[forward_xy_stride + z_stride_length]
                                    && voxel == voxels[xyz_stride + z_stride_length]
                                {
                                    self.upward_merged[forward_xy_stride] = 0;
                                    *forward_merged += 1;
                                    continue;
                                }

                                let upward_merged =
                                    std::mem::take(&mut self.upward_merged[forward_xy_stride]);

                                let quad_y = y as u32 - 1 - upward_merged as u32;
                                let quad_x = x as u32 - 1;
                                let quad_z = z as u32 - 1 + face.is_positive() as u32;
                                let quad_w = *forward_merged as u32;
                                let quad_h = upward_merged as u32 + 1;

                                let quad = VoxelQuad::new(
                                    quad_x + if let PosX = face { quad_w } else { 0 },
                                    quad_y,
                                    quad_z,
                                    quad_w,
                                    quad_h,
                                    voxel.id as u32,
                                );

                                self.mesh[face_index].push(quad);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn fast_mesh(
        &mut self,
        voxels: &[Voxel; PADDED_CHUNK_VOLUME],
        opaque_mask: &[u64; PADDED_CHUNK_AREA],
        transparent_mask: &[u64; PADDED_CHUNK_AREA],
    ) {
        self.fast_face_culling(voxels, opaque_mask, transparent_mask);
        self.face_merging(voxels);
    }

    pub fn mesh(&mut self, voxels: &[Voxel; PADDED_CHUNK_VOLUME], transparents: &BTreeSet<Voxel>) {
        self.face_culling(voxels, transparents);
        self.face_merging(voxels);
    }
}

#[inline]
fn is_visible(voxel: Voxel, neighbor: Voxel, transparents: &BTreeSet<Voxel>) -> bool {
    neighbor.is_sentinel() || (voxel != neighbor && transparents.contains(&neighbor))
}

#[inline]
fn is_visible_as_u64(voxel: Voxel, neighbor: Voxel, transparents: &BTreeSet<Voxel>) -> u64 {
    is_visible(voxel, neighbor, transparents) as u64
}

fn offset_stride(axis: SignedAxis, base: usize) -> usize {
    let (sign, axis) = axis.split();
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

pub fn compute_opaque_mask(
    voxels: &[Voxel; PADDED_CHUNK_VOLUME],
    transparents: &BTreeSet<Voxel>,
) -> [u64; PADDED_CHUNK_AREA] {
    let mut opaque_mask = [0; PADDED_CHUNK_AREA];

    for (i, voxel) in voxels.iter().enumerate() {
        if voxel.is_sentinel() || transparents.contains(voxel) {
            continue;
        }

        let row = i / PADDED_CHUNK_LENGTH;
        let bit = i % PADDED_CHUNK_LENGTH;

        opaque_mask[row] |= 1 << bit;
    }

    opaque_mask
}

pub fn compute_transparent_mask(
    voxels: &[Voxel; PADDED_CHUNK_VOLUME],
    transparents: &BTreeSet<Voxel>,
) -> [u64; PADDED_CHUNK_AREA] {
    let mut transparent_mask = [0; PADDED_CHUNK_AREA];

    for (i, voxel) in voxels.iter().enumerate() {
        if voxel.is_sentinel() || !transparents.contains(voxel) {
            continue;
        }

        let row = i / PADDED_CHUNK_LENGTH;
        let bit = i % PADDED_CHUNK_LENGTH;

        transparent_mask[row] |= 1 << bit;
    }

    transparent_mask
}

#[cfg(test)]
mod tests {
    use ndshape::Shape;
    use std::collections::BTreeSet;

    use crate::{
        chunk::{CHUNK_LENGTH, CHUNK_SHAPE, PADDED_CHUNK_AREA, PADDED_CHUNK_VOLUME},
        voxel::Voxel,
    };

    use super::{Mesher, compute_opaque_mask, compute_transparent_mask};

    #[test]
    fn test_output() {
        let mut voxels = [Voxel::default(); PADDED_CHUNK_VOLUME];
        voxels[CHUNK_SHAPE.linearize([1, 1, 1]) as usize] = Voxel { id: 1 };
        voxels[CHUNK_SHAPE.linearize([1, 2, 1]) as usize] = Voxel { id: 1 };

        let mut mesher = Mesher::new();
        let opaque_mask = compute_opaque_mask(&voxels, &BTreeSet::new());
        let trans_mask = Box::new([0; PADDED_CHUNK_AREA]);
        mesher.fast_mesh(&voxels, &opaque_mask, &trans_mask);
        for (i, quads) in mesher.mesh.iter().enumerate() {
            println!("--- SignedAxis {i} ---");
            for &quad in quads {
                println!("{quad:?}");
            }
        }
    }

    #[test]
    fn same_results() {
        let voxels = test_buffer();
        let transparent_blocks = BTreeSet::from([Voxel { id: 1 }]);
        let opaque_mask = compute_opaque_mask(&voxels, &BTreeSet::new());
        let trans_mask = compute_transparent_mask(&voxels, &transparent_blocks);
        let mut mesher1 = Mesher::new();
        mesher1.mesh(&voxels, &transparent_blocks);
        let mut mesher2 = Mesher::new();
        mesher2.fast_mesh(&voxels, &opaque_mask, &trans_mask);
        assert_eq!(mesher1.mesh, mesher2.mesh);
    }

    fn test_buffer() -> Box<[Voxel; PADDED_CHUNK_VOLUME]> {
        let mut voxels = Box::new([Voxel::default(); PADDED_CHUNK_VOLUME]);
        for x in 0..CHUNK_LENGTH as u32 {
            for y in 0..CHUNK_LENGTH as u32 {
                for z in 0..CHUNK_LENGTH as u32 {
                    voxels[CHUNK_SHAPE.linearize([x + 1, y + 1, z + 1]) as usize] =
                        transparent_sphere(x, y, z);
                }
            }
        }
        voxels
    }

    fn transparent_sphere(x: u32, y: u32, z: u32) -> Voxel {
        if x == 8 {
            Voxel { id: 1 }
        } else if (x as i32 - 31).pow(2) + (y as i32 - 31).pow(2) + (z as i32 - 31).pow(2)
            < 16 as i32
        {
            Voxel { id: 0 }
        } else {
            Voxel::default()
        }
    }
}
