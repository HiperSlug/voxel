use enum_map::enum_map;

use crate::{
    math::{axis::*, signed_axis::*},
    voxel::Voxel,
};

use super::{
    Chunk, VoxelQuad,
    padded::{AREA, LEN, VOL},
    padded::{
        X_SHIFT as SHIFT_0, X_STRIDE as STRIDE_0, Y_SHIFT as SHIFT_1,
        Y_STRIDE as STRIDE_1, Z_SHIFT as SHIFT_2, Z_STRIDE as STRIDE_2,
    },
};

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

            let cub_adj_offset = match signed_axis {
                PosX => STRIDE_0 as isize,
                NegX => -(STRIDE_0 as isize),
                PosY => STRIDE_1 as isize,
                NegY => -(STRIDE_1 as isize),
                PosZ => STRIDE_2 as isize,
                NegZ => -(STRIDE_2 as isize),
            };

            for z in 1..LEN - 1 {
                let cub_z = z << SHIFT_2;

                let col_z = z << SHIFT_1;

                for y in 1..LEN - 1 {
                    let cub_y = y << SHIFT_1;
                    let cub_yz = cub_y | cub_z;

                    let col_y = y << SHIFT_0;
                    let col_yz = col_y | col_z;

                    let opaque = opaque_mask[col_yz];
                    let unpad_opaque = opaque & UNPADDED_MASK;
                    let unpad_transparent = transparent_mask[col_yz] & UNPADDED_MASK;

                    if unpad_opaque == 0 && unpad_transparent == 0 {
                        continue;
                    }

                    let adj_opaque = match signed_axis {
                        PosX => opaque << 1,
                        NegX => opaque >> 1,
                        PosY => opaque_mask[col_yz + STRIDE_0],
                        NegY => opaque_mask[col_yz - STRIDE_0],
                        PosZ => opaque_mask[col_yz + STRIDE_1],
                        NegZ => opaque_mask[col_yz - STRIDE_1],
                    };

                    visible_mask[col_yz] = unpad_opaque & !adj_opaque;

                    let mut visible_transparent = unpad_transparent & !adj_opaque;

                    while visible_transparent != 0 {
                        let x = visible_transparent.trailing_zeros() as usize;
                        visible_transparent &= visible_transparent - 1;

                        let cub_x = x << SHIFT_0;
                        let cub_xyz = cub_x | cub_yz;

                        let voxel = voxels[cub_xyz];

                        let adj_index = (cub_xyz as isize + cub_adj_offset) as usize;
                        let adj_voxel = voxels[adj_index];

                        visible_mask[col_yz] |= ((voxel != adj_voxel) as u64) << x;
                    }
                }
            }
        }
    }

    fn face_merging(&mut self, voxels: &[Voxel; VOL]) {
        // along Z axis, should be the same for neg and pos
        let visible_mask = &mut self.visible_masks[PosZ];
        for z in 1..LEN - 1 {
            let cub_z = z << SHIFT_2;

            let col_z = z << SHIFT_1;

            for y in 1..LEN - 1 {
                let cub_y = y << SHIFT_1;
                let cub_yz = cub_y | cub_z;

                let col_y = y << SHIFT_0;
                let col_yz = col_y | col_z;

                let mut column = visible_mask[col_yz];
                if column == 0 {
                    continue;
                }

                let upward_column = visible_mask[col_yz + STRIDE_0];

                while column != 0 {
                    let x = column.trailing_zeros() as usize;

                    let cub_x = x << SHIFT_0;
                    let cub_xyz = cub_x | cub_yz;

                    let voxel = voxels[cub_xyz];

                    if (upward_column >> x) & 1 != 0 && voxel == voxels[cub_xyz + STRIDE_1] {
                        self.upward_merged[x] += 1;
                        column &= column - 1;
                        continue;
                    }

                    let mut right_merged = 1;
                    for right in (x + 1)..LEN - 1 {
                        if (column >> right) & 1 == 0
                            || self.upward_merged[x] != self.upward_merged[right]
                            || voxel != {
                                let cub_x = right << SHIFT_0;
                                let cub_xyz = cub_x | cub_yz;
                                voxels[cub_xyz]
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

                    let x = x as u32;
                    let y = y as u32;
                    let z = z as u32;

                    let quad = VoxelQuad::new(
                        x,
                        y - h,
                        z,
                        w,
                        h + 1,
                        PosZ,
                        0, // TODO
                        0, // TODO
                    );
                }
            }
        }

        // Along Y axis.
        // keep in mind that queries are not in an optimal order
        let visible_mask = &mut self.visible_masks[PosY];
        for z in 1..LEN - 1 {
            let col_z = z << Z_SHIFT;

            let cub_z = z << Z_SHIFT;

            for y in 1..LEN - 1 {
                let col_y = y << Y_SHIFT;
                let col_yz = col_y | col_z;

                let cub_y = y << Y_SHIFT;
                let cub_yz = cub_y | cub_z;

                let mut column = visible_mask[col_yz];
                if column == 0 {
                    continue;
                }

                let forward_column = visible_mask[col_yz + Z_STRIDE]; // jump

                while column != 0 {
                    let x = column.trailing_zeros() as usize;

                    let cub_x = x << X_SHIFT;
                    let cub_xyz = cub_x | cub_yz;

                    let cub_xy = cub_x | cub_y;

                    let voxel = voxels[cub_xyz];

                    if (forward_column >> x) & 1 != 0 && voxel == voxels[cub_xyz + Z_STRIDE] {
                        self.forward_merged[cub_xy] += 1;
                        column &= column - 1;
                        continue;
                    }

                    let mut right_merged = 1;
                    for right in (x + 1)..LEN - 1 {
                        let cub_x = right << X_SHIFT;
                        if (column >> right) & 1 == 0
                            || self.forward_merged[cub_xy] != self.forward_merged[cub_x | cub_y]
                            || voxel != voxels[cub_x | cub_yz]
                        {
                            break;
                        }
                        self.forward_merged[cub_x | cub_y] = 0;
                        right_merged += 1;
                    }
                    let cleared = x + right_merged;
                    column &= !((1 << cleared) - 1);

                    let w = right_merged as u32;
                    let h = self.forward_merged[cub_xy] as u32;

                    self.forward_merged[cub_xy] = 0;

                    let x = x as u32;
                    let y = y as u32;
                    let z = z as u32;

                    let quad = VoxelQuad::new(
                        x,
                        y,
                        z - h,
                        w,
                        h + 1,
                        PosY,
                        0, // TODO
                        0, // TODO
                    );
                }
            }
        }

        for signed_axis in SignedAxis::ALL {
            let visible_mask = &mut self.visible_masks[signed_axis];

            let axis_map = AxisPermutation::even(signed_axis.axis()).axis_map();

            for z in 1..LEN - 1 {
                let cub_z_shift = SHIFTS[axis_map[Z]];
                let cub_z_stride = 1 << cub_z_shift;
                let cub_z = z << cub_z_shift;

                let col_z = z << Z_SHIFT;

                for y in 1..LEN - 1 {
                    let cub_y_shift = SHIFTS[axis_map[Y]];
                    let cub_y_stride = 1 << cub_y_shift;
                    let cub_y = y << cub_y_shift;
                    let cub_yz = cub_y | cub_z;

                    let col_y = y << Y_SHIFT;
                    let col_yz = col_y | col_z;

                    match signed_axis {
                        PosY | NegY | PosZ | NegZ => {
                            let mut column = visible_mask[col_yz];
                            if column == 0 {
                                continue;
                            }

                            let upward_column = visible_mask[col_yz + Y_STRIDE];

                            // in these cases forward is along the length of the column
                            let mut forward_merged = 1;
                            while column != 0 {
                                let x = column.trailing_zeros() as usize;

                                let cub_x_shift = SHIFTS[axis_map[X]];
                                let cub_x = x << cub_x_shift;
                                let cub_xyz = cub_x | cub_yz;

                                let voxel = voxels[cub_xyz];

                                if (upward_column >> x) & 1 != 0
                                    && voxel == voxels[cub_xyz + cub_y_stride]
                                {
                                    self.upward_merged[x] += 1;
                                    column &= !(1 << x);
                                    continue;
                                }

                                for forward in (x + 1)..(PADDED_CHUNK_LENGTH - 1) {
                                    if (column >> forward) & 1 == 0
                                        || self.forward_merged[x] != self.forward_merged[forward]
                                        || voxel != voxels[xyz + z_length]
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
                                let quad_z = z as u32 - 1 + signed_axis.is_positive() as u32;
                                let quad_w = forward_merged as u32;
                                let quad_h = upward_merged as u32 + 1;

                                let quad = match signed_axis {
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

                                self.mesh[signed_axis_index].push(quad);
                            }
                        }
                        PosX | NegX => {
                            let mut column = self.visible_masks[layer_yzl];
                            if column == 0 {
                                continue;
                            }

                            let upward_column = if (y + 1) < (PADDED_CHUNK_LENGTH - 1) {
                                self.visible_masks[layer_yzl + LAYER_Y_STRIDE]
                            } else {
                                0
                            };

                            let forward_column = if (z + 1) < (PADDED_CHUNK_LENGTH - 1) {
                                self.visible_masks[layer_yzl + LAYER_Z_STRIDE]
                            } else {
                                0
                            };

                            let forward_y = y << Y_SHIFT;

                            while column != 0 {
                                let x = column.trailing_zeros() as usize;
                                column &= !(1 << x);

                                let x_significance = sigificance_map[0];
                                let x_shift = ChunkShape::SHIFTS[x_significance];

                                let x = x << x_shift;
                                let xyz = x + yz;

                                let forward_x = x << X_SHIFT;
                                let forward_xy = forward_y + forward_x;

                                let voxel = voxels[xyz];

                                let forward_merged = &mut self.forward_merged[x];

                                if *forward_merged == 0
                                    && (upward_column >> x) & 1 != 0
                                    && voxel == voxels[xyz + y_length]
                                {
                                    self.upward_merged[forward_xy] += 1;
                                    continue;
                                }

                                if (forward_column >> x) & 1 != 0
                                    && self.upward_merged[forward_xy]
                                        == self.upward_merged[forward_xy + z_length]
                                    && voxel == voxels[xyz + z_length]
                                {
                                    self.upward_merged[forward_xy] = 0;
                                    *forward_merged += 1;
                                    continue;
                                }

                                let upward_merged =
                                    std::mem::take(&mut self.upward_merged[forward_xy]);

                                let quad_y = y as u32 - 1 - upward_merged as u32;
                                let quad_x = x as u32 - 1;
                                let quad_z = z as u32 - 1 + signed_axis.is_positive() as u32;
                                let quad_w = *forward_merged as u32;
                                let quad_h = upward_merged as u32 + 1;

                                let quad = VoxelQuad::new(
                                    quad_x + if let PosX = signed_axis { quad_w } else { 0 },
                                    quad_y,
                                    quad_z,
                                    quad_w,
                                    quad_h,
                                    voxel.id as u32,
                                );

                                self.mesh[signed_axis_index].push(quad);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn fast_mesh(
        &mut self,
        Chunk {
            voxels,
            opaque_mask,
            transparent_mask,
        }: &Chunk,
    ) {
        self.fast_face_culling(voxels, opaque_mask, transparent_mask);
        self.signed_axis_merging(voxels);
    }

    // pub fn mesh(&mut self, voxels: &[Voxel; PADDED_CHUNK_VOLUME], transparents: &BTreeSet<Voxel>) {
    //     self.signed_axis_culling(voxels, transparents);
    //     self.signed_axis_merging(voxels);
    // }
}

#[inline]
fn is_visible(voxel: Voxel, neighbor: Voxel, transparents: &BTreeSet<Voxel>) -> bool {
    neighbor.is_sentinel() || (voxel != neighbor && transparents.contains(&neighbor))
}

#[inline]
fn is_visible_as_u64(voxel: Voxel, neighbor: Voxel, transparents: &BTreeSet<Voxel>) -> u64 {
    is_visible(voxel, neighbor, transparents) as u64
}

fn offset(axis: SignedAxis, base: usize) -> usize {
    let (sign, axis) = axis.split();
    let unsigned = match axis {
        Axis::X => X_STRIDE,
        Axis::Y => Y_STRIDE,
        Axis::Z => Z_STRIDE,
    };

    match sign {
        Sign::Pos => base + unsigned,
        Sign::Neg => base - unsigned,
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
