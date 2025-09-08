use bevy::{
    ecs::{query::QueryItem, system::lifetimeless::Read},
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
    // todo: stop cloning this whole thing every frame
    pub visible_quad_ranges: Vec<Vec<(u32, u32)>>,
}

#[derive(Component)]
pub struct ExtractTerrain {
    pub visible_quad_ranges: Vec<Vec<(u32, u32)>>,
}

impl ExtractComponent for Terrain {
    type Out = ExtractTerrain;
    type QueryData = Read<Terrain>;
    type QueryFilter = ();

    fn extract_component(terrain: QueryItem<'_, Self::QueryData>) -> Option<Self::Out> {
        Some(ExtractTerrain {
            visible_quad_ranges: terrain.visible_quad_ranges.clone(),
        })
    }
}

// for (chunk_pos, chunk_mesh_opt) in terrain.chunk_mesh_map.iter().map(|r| r.pair()) {
//     if let Some(chunk_mesh) = chunk_mesh_opt {
//         let visible = |signed_axis: &SignedAxis| match signed_axis {
//             PosX => view_chunk_pos.x >= chunk_pos.x, // ORDERING
//             NegX => view_chunk_pos.x <= chunk_pos.x,
//             PosY => view_chunk_pos.y >= chunk_pos.y,
//             NegY => view_chunk_pos.y <= chunk_pos.y,
//             PosZ => view_chunk_pos.z >= chunk_pos.z,
//             NegZ => view_chunk_pos.z <= chunk_pos.z,
//         };

//         for signed_axis in SignedAxis::ALL.iter().copied().filter(visible) {
//             let Some(offset) = chunk_mesh.offsets[signed_axis].map(|o| o.get()) else {
//                 continue;
//             };
//             let size = chunk_mesh
//                 .offsets
//                 .iter()
//                 .skip(signed_axis.into_usize())
//                 .find_map(|(_, o)| o.as_ref().map(|o| o.get() - offset))
//                 .unwrap_or_else(|| chunk_mesh.allocation.size());

//             let args = DrawIndirect {
//                 vertex_count: 4,
//                 first_vertex: 0,
//                 first_instance: offset,
//                 instance_count: size,
//             };

//             indirect_args_slab_map[chunk_mesh.allocation.slab_index()].push(args);
//         }
//     }
// }

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
