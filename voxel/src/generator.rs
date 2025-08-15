use crate::{
    chunk::{CHUNK_SHAPE, Chunk, WORLD_CHUNK_LENGTH},
    voxel::Voxel,
};
use bevy::math::{I64Vec3, IVec3};
use fastnoise_lite::{self, FastNoiseLite};
use ndshape::Shape;
use std::{array, sync::LazyLock, u16};

pub fn chunk_pos_to_voxel_pos(chunk_pos: IVec3) -> I64Vec3 {
    chunk_pos.as_i64vec3() * WORLD_CHUNK_LENGTH as i64
}

static NOISE: LazyLock<FastNoiseLite> = LazyLock::new(|| FastNoiseLite::default());

pub fn temp(chunk_pos: IVec3) -> Chunk {
    let voxel_pos = chunk_pos_to_voxel_pos(chunk_pos);

    let c: Chunk = Chunk::new(array::from_fn(|i| {
        let global_position =
            I64Vec3::from(CHUNK_SHAPE.delinearize(i as u32).map(|c| c as i64)) + voxel_pos;
        let y_cutoff =
            (NOISE.get_noise_2d(global_position.x as f32, global_position.z as f32) * 100.0) as i64;
        if global_position.y > y_cutoff {
            Voxel { id: u16::MAX }
        } else {
            Voxel { id: 0 }
        }
    }));
    c
}
