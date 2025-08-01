use crate::data::{
    chunk::{self, Chunk},
    voxel::Voxel,
};
use arc_swap::ArcSwap;
use bevy::math::{I64Vec3, IVec3};
use fastnoise_lite::{self, FastNoiseLite};
use std::{array, sync::Arc};

pub fn chunk_pos_to_voxel_pos(chunk_pos: IVec3) -> I64Vec3 {
    chunk_pos.as_i64vec3() * chunk::LENGTH_IN_VOXELS as i64
}

static NOISE: std::sync::LazyLock<FastNoiseLite> =
    std::sync::LazyLock::new(|| FastNoiseLite::default());

pub fn temp(chunk_pos: IVec3) -> Chunk {
    let voxel_pos = chunk_pos_to_voxel_pos(chunk_pos);

    let mut c: Chunk = Chunk::Mixed(ArcSwap::new(Arc::new(array::from_fn(|i| {
        let global_position = chunk::delinearize(i).as_i64vec3() + voxel_pos;
        let y_cutoff =
            (NOISE.get_noise_2d(global_position.x as f32, global_position.z as f32) * 100.0) as i64;
        if global_position.y > y_cutoff {
            Voxel(0)
        } else {
            Voxel(1)
        }
    }))));
    c.attempt_collapse();
    c
}
