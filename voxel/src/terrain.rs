use bevy::prelude::*;
use dashmap::DashMap;
use std::sync::Arc;

use crate::{
    chunk::{Chunk, ChunkMap, ChunkPos},
    viewer::Viewer,
};

#[derive(Component)]
pub struct Terrain {
    pub chunk_map: Arc<ChunkMap>,
}

pub fn visibile(
    terrains: Query<&Terrain>,
    viewers: Query<(&Transform, &Viewer), Changed<Transform>>,
) {
    for (transform, viewer) in viewers {
        let chunk_pos = ChunkPos::from_world(transform.translation);
        let visible_positions = viewer.visible_positions(chunk_pos);

        for position in visible_positions {
            for terrain in terrains {
                let chunk = terrain.chunk_map.get(&position);
            }
        }
    }
}
