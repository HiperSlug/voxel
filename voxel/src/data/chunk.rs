use super::brick::{self, Brick};
use crate::{data::voxel::VoxelVisibility, utils::subdivide_index};
use bevy::math::{I8Vec3, U8Vec3, Vec3};
use std::array;

const BITS: u8 = 1;

pub const LENGTH_IN_BRICKS: u8 = 1 << BITS;

pub const VOLUME_IN_BRICKS: usize = (LENGTH_IN_BRICKS as usize).pow(3);

pub const LENGTH_IN_VOXELS: u8 = LENGTH_IN_BRICKS * brick::LENGTH_IN_VOXELS;

pub const LENGTH: f32 = LENGTH_IN_BRICKS as f32 * brick::LENGTH;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    pub bricks: [Brick; VOLUME_IN_BRICKS],
}

impl Chunk {
    pub fn from_fn_positions<F>(function: F) -> Self
    where
        F: Fn(U8Vec3) -> Brick,
    {
        let bricks = array::from_fn(|index| {
            let position = subdivide_index::<BITS>(index);
            function(position)
        });
        Self { bricks }
    }

    pub fn bake_visibility(&mut self, adjacent_chunks: [Option<&Chunk>; 6]) {
        for (pos, brick) in self.bricks.iter_mut().enumerate().map(|(i, b)| (subdivide_index::<BITS>(i), b)) {
            let adjacent_bricks = adjacent_positions(pos).map(|pos| {
                
            });
        }
        
        let brick = self.bricks.iter().map(|brick| {
            match brick {
                Brick::NonUniform(voxels) => {
                    let visiblity = voxels.iter().enumerate().map(|(i, v)| {
                        let pos = brick::index_to_voxel_position(i);
                        let visiblity = surrounding_positions(pos).into_iter().enumerate().map(|(i, pos)| {
                            if brick::pos_in_bounds(pos) {
                                brick.get(pos.as_u8vec3()).is_empty()
                            } else {
                                // which direction did we exit from.
                                // which brick did we exit into
                                // where is the brick that owns this voxel
                                // get the voxel
                                // is empty
                                todo!()
                            }
                        }).collect::<Vec<_>>();
                        
                        VoxelVisibility {
                            pos_x: visiblity[0],
                            neg_x: visiblity[1],
                            pos_y: visiblity[2],
                            neg_y: visiblity[3],
                            pos_z: visiblity[4],
                            neg_z: visiblity[5],
                        }
                    }).collect::<Vec<_>>();

                    for (voxel, visibility) in voxels.iter_mut().zip(visiblity) {
                        voxel.set_visibility(visibility);
                    }
                },
                Brick::Uniform(of) => todo!(),
            }
        }).collect();
    }
}

pub fn index_to_global_position(index: usize) -> Vec3 {
    brick::LENGTH * subdivide_index::<BITS>(index).as_vec3()
}

fn adjacent_positions(pos: U8Vec3) -> [I8Vec3; 6] {
    debug_assert!(pos.x < 128 && pos.y < 128 && pos.z < 128);
    [
        pos.as_i8vec3() + I8Vec3::X,
        pos.as_i8vec3() + I8Vec3::NEG_X,
        pos.as_i8vec3() + I8Vec3::Y,
        pos.as_i8vec3() + I8Vec3::NEG_Y,
        pos.as_i8vec3() + I8Vec3::Z,
        pos.as_i8vec3() + I8Vec3::NEG_Z,
    ]
}
