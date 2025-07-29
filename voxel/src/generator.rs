use crate::data::{
    raw_chunk::{self, RawChunk},
    voxel::Voxel,
};
use arc_swap::ArcSwap;
use bevy::math::{I64Vec3, IVec3};
use fastnoise_lite::{self, FastNoiseLite};
use std::{array, sync::Arc};

pub fn chunk_pos_to_voxel_pos(chunk_pos: IVec3) -> I64Vec3 {
    chunk_pos.as_i64vec3() * raw_chunk::LENGTH_IN_VOXELS as i64
}

pub fn flat(chunk_pos: IVec3) -> RawChunk {
    let voxel_pos = chunk_pos_to_voxel_pos(chunk_pos);

    if voxel_pos.y < -(raw_chunk::LENGTH_IN_VOXELS as i64) {
        RawChunk::Uniform(Voxel(1))
    } else if voxel_pos.y >= 0 {
        RawChunk::Uniform(Voxel(0))
    } else {
        RawChunk::Mixed(ArcSwap::new(Arc::new(array::from_fn(|i| {
            let global_position = raw_chunk::index_to_pos(i).as_i64vec3() + voxel_pos;
            if global_position.y >= 0 {
                Voxel(0)
            } else {
                Voxel(1)
            }
        }))))
    }
}

static NOISE: std::sync::LazyLock<FastNoiseLite> =
    std::sync::LazyLock::new(|| FastNoiseLite::default());

pub fn noise(chunk_pos: IVec3) -> RawChunk {
    let voxel_pos = chunk_pos_to_voxel_pos(chunk_pos);

    let mut c = RawChunk::Mixed(ArcSwap::new(Arc::new(array::from_fn(|i| {
        let global_position = raw_chunk::index_to_pos(i).as_i64vec3() + voxel_pos;
        let y_cutoff =
            (NOISE.get_noise_2d(global_position.x as f32, global_position.z as f32) * 100.0) as i64;
        if y_cutoff > global_position.y {
            Voxel(0)
        } else {
            Voxel(1)
        }
    }))));
    c.attempt_collapse();
    c
}
