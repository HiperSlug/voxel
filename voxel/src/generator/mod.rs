use crate::data::{
    brick::{self, Brick},
    chunk::{self, Chunk},
};
use bevy::math::IVec3;

pub fn flat(chunk_pos: IVec3) -> Chunk {
    Chunk::from_fn_positions(|brick_pos| {
        let mut brick = Brick::from_fn_positions(|voxel_pos| {
            let global_voxel_pos = chunk_pos.as_i64vec3() * chunk::LENGTH_IN_VOXELS as i64
                + brick_pos.as_i64vec3() * brick::LENGTH_IN_VOXELS as i64
                + voxel_pos.as_i64vec3();

            if global_voxel_pos.y < 0 { 1 } else { 0 }
        });

        brick.attempt_collapse();
        brick
    })
}
