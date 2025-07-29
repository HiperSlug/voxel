pub mod voxel_viewer;

pub mod voxel_volume;

pub mod chunk;

use crate::data::chunk::LENGTH as GLOBAL_CHUNK_LENGTH;
use bevy::math::{IVec3, Vec3};

pub fn global_pos_to_chunk_pos(global: Vec3) -> IVec3 {
    (global / GLOBAL_CHUNK_LENGTH).floor().as_ivec3()
}

pub fn chunk_pos_to_global_pos(chunk_pos: IVec3) -> Vec3 {
    chunk_pos.as_vec3() * GLOBAL_CHUNK_LENGTH
}
