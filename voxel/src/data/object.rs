use crate::data::{octree::Octree, voxel::Voxel};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

const CHUNKS_BITS: u8 = 4;
const CHUNK_LEN: u8 = 1 << CHUNKS_BITS;
const MAX: u8 = CHUNK_LEN - 1;

pub type Chunk = Octree<Voxel, CHUNKS_BITS>;

#[derive(Component)]
pub struct VolumetricObject {
    pub data: HashMap<(i32, i32, i32), Chunk>,
}
