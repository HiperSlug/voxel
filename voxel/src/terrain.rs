use bevy::{
    prelude::*,
    render::extract_component::{ExtractComponent, ExtractComponentPlugin},
};
use std::{collections::HashSet, sync::Arc};

use crate::{
    chunk::{Chunk, ChunkMap, ChunkMeshMap, ChunkPos},
    viewer::Viewer,
};

#[derive(Component)]
pub struct Terrain {
    pub chunk_map: Arc<ChunkMap>,
    pub chunk_mesh_map: Arc<ChunkMeshMap>,
    pub visible_quad_slices: Vec<Vec<(u32, u32)>>,
}

pub struct ExtractTerrainMesh {

}

impl ExtractComponent for 

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

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<TerrainMesh>::default());
    }
}
