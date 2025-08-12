use super::{CHUNK_LENGTH, CHUNK_AREA, CHUNK_VOLUME};

pub struct Mesher {
    pub quads: [Vec<VoxelQuad>; 6],

    face_masks: Box<[u64; CHUNK_AREA * 6]>,

    upward_merged: Box<[u8; CHUNK_AREA]>,
    right_merged: Box<[u8; CHUNK_AREA]>,
}

impl Mesher {
    pub fn new() -> Self {
        Self {
            quads: array::from_fn(|_| Vec::new()),
            face_masks: Box::new([0; CHUNK_AREA * 6]),
            upward_merged: Box::new([0; CHUNK_AREA]),
            right_merged: Box::new([0; CHUNK_AREA]),
        }
    }

    pub fn clear(&mut self) {
        self.face_masks.fill(0);
        self.upward_merged.fill(0);
        self.right_merged.fill(0);
        for face in &mut self.quads {
            face.clear();
        }
    }

    fn face_culling(
        &mut self,
        voxels: &[Voxel; CHUNK_VOLUME],
        transparents: &BTreeSet<Voxel>,
    ) {
        for layer_pos in UNRANGE {
            let layer_index = layer_pos * CHUNK_LENGTH;
            let unpadded_layer_index = (layer_pos - 1) * CHUNK_LENGTH;

            for column_pos in UNRANGE {
                let column_index = layer_index + column_pos;
                let unpadded_index = unpadded_layer_index + (column_pos - 1);

                for bit_pos in UNRANGE {
                    let cubic_index = column_index + bit_pos;
                    let voxel = voxels[cubic_index];
                    if voxel.is_sentinel() {
                        continue;
                    }

                    for (index, adj_index) in (0..6)
                        .map(|i| unpadded_index + i * CHUNK_AREA)
                        .zip(CUBIC_OFFSETS.map(|offset| ((cubic_index as isize) + offset) as usize))
                    {
                        let neighbor = voxels[adj_index];

                        self.face_masks[index] |=
                            (is_visible(voxel, neighbor, transparents) as u64) << (bit_pos - 1);
                    }
                }
            }
        }
    }

    fn fast_face_culling(
        &mut self,
        voxels: &[Voxel; CHUNK_VOLUME],
        opaque_mask: &[u64; CHUNK_AREA],
        transparent_mask: &[u64; CHUNK_AREA],
    ) {
        for layer_pos in UNRANGE {
            let layer_index = layer_pos * CHUNK_LENGTH;

            for column_pos in UNRANGE {
                let column_index = layer_index + column_pos;
                let column_indices: [usize; 6] = array::from_fn(|i| layer_index + i * CHUNK_AREA);

                let padded_column = opaque_mask[column_index];
                let unpadded_column = padded_column & UNMASK;

                if unpadded_column != 0 {
                    let adjacent_columns = [
                        opaque_mask[column_index + CHUNK_LENGTH],
                        opaque_mask[column_index - CHUNK_LENGTH],
                        opaque_mask[column_index + 1],
                        opaque_mask[column_index - 1],
                        padded_column << 1,
                        padded_column >> 1,
                    ]
                    .into_iter();

                    for (column_index, adj_col) in
                        column_indices.iter().copied().zip(adjacent_columns)
                    {
                        self.face_masks[column_index] = (unpadded_column & !adj_col) >> 1;
                    }
                }

                let cubic_base = column_index * CHUNK_LENGTH;

                let mut unpadded_transparent = transparent_mask[layer_index] & UNMASK;
                while unpadded_transparent != 0 {
                    let bit_pos = unpadded_transparent.trailing_zeros() as usize;
                    unpadded_transparent &= !(1 << bit_pos);

                    let cubic_index = cubic_base + bit_pos;

                    let voxel = voxels[cubic_index];

                    for (column_index, cubic_idx) in column_indices.iter().copied().zip(
                        CUBIC_OFFSETS
                            .iter()
                            .copied()
                            .map(|offset| (cubic_index as isize + offset) as usize),
                    ) {
                        let neighbor = voxels[cubic_idx];
                        self.face_masks[column_index] |=
                            ((voxel != neighbor) as u64) << (bit_pos - 1);
                    }
                }
            }
        }
    }

    fn face_merging(&mut self, voxels: &[Voxel; CHUNK_VOLUME]) {
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
                            let up_voxel_index = permutation
                                .linearize_cubic::<CHUNK_LENGTH>(
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
    neighbor == 0 || (voxel != neighbor && transparents.contains(&neighbor))
}
