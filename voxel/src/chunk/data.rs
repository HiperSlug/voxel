use arc_swap::ArcSwap;

use super::PADDED_CHUNK_VOLUME;
use crate::voxel::Voxel;

#[derive(Debug)]
pub enum Chunk {
    Uniform(Voxel),
    Mixed(ArcSwap<[Voxel; PADDED_CHUNK_VOLUME as usize]>),
}

impl Chunk {
    pub fn attempt_collapse(&mut self) -> bool {
        use Chunk::*;
        match self {
            Uniform(_) => true,
            Mixed(voxels) => {
                let guard = voxels.load();
                let base = guard[0];
                let can_collapse = guard.iter().skip(1).copied().all(|v| v == base);
                if can_collapse {
                    *self = Uniform(base);
                }
                can_collapse
            }
        }
    }
}
