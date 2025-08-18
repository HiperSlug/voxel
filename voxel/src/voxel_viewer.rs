use bevy::prelude::*;

use crate::chunk::ChunkPos;

#[derive(Debug, Component)]
pub struct VoxelViewer {
    pub radius: i32,
}

impl VoxelViewer {
    pub fn new(radius: i32) -> Self {
        Self { radius }
    }

    pub fn visible_positions(&self, origin: ChunkPos) -> impl Iterator<Item = ChunkPos> {
        let radius = self.radius;
        let radius_sq = radius.pow(2);

        (-radius..=radius).flat_map(move |x| {
            (-radius..=radius).flat_map(move |y| {
                (-radius..=radius).filter_map(move |z| {
                    let offset = IVec3::new(x, y, z);
                    if offset.length_squared() <= radius_sq {
                        Some((origin.0 + offset).into())
                    } else {
                        None
                    }
                })
            })
        })
    }
}
