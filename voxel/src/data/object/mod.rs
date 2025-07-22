use crate::data::chunk::{Chunk, ChunkPos};
use std::collections::HashMap;

pub struct VolumetricObject {
    chunks: HashMap<ChunkPos, Chunk>,
}
