use bevy::prelude::*;
use fastnoise_lite::FastNoiseLite;
use nonmax::NonMaxU16;
use std::sync::LazyLock;

use crate::{block_library::BlockLibrary, voxel::Voxel};

use super::{
    Chunk, chunk_origin,
    pad::{LEN, linearize},
};

const AMPLITUDE: f32 = 32.0;

static NOISE: LazyLock<FastNoiseLite> = LazyLock::new(FastNoiseLite::default);

pub fn generate(chunk_pos: IVec3, block_library: &BlockLibrary) -> Chunk {
    let mut chunk = Chunk::EMPTY;

    let chunk_origin = chunk_origin(chunk_pos);

    for offset_z in 0..LEN as u32 {
        for offset_y in 0..LEN as u32 {
            for offset_x in 0..LEN as u32 {
                let voxel_pos = chunk_origin + UVec3::new(offset_x, offset_y, offset_z).as_ivec3();

                let h = NOISE.get_noise_2d(voxel_pos.x as f32, voxel_pos.y as f32) * AMPLITUDE;

                let index = linearize([offset_x, offset_y, offset_z]);

                chunk.voxels[index] = if voxel_pos.y > h as i32 {
                    None
                } else {
                    Some(Voxel(NonMaxU16::new(1).unwrap()))
                }
            }
        }
    }

    chunk.build_masks(block_library);

    chunk
}
