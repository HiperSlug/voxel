use bevy::prelude::*;

use crate::{block_library::BlockLibraryPlugin, chunk, mesher, voxel_volume};

pub struct VoxelPlugin {
    block_library_path: String
}

impl VoxelPlugin {
    pub fn new(block_library_path: impl Into<String>) -> Self {
        Self { block_library_path: block_library_path.into() }
    }
}

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(BlockLibraryPlugin::new(&self.block_library_path))
            .add_systems(
            Update,
            ((
                (
                    voxel_volume::update_visible_chunks,
                    mesher::handle_chunk_meshing,
                ),
                (chunk::poll_chunk_constructors, chunk::poll_chunk_meshers),
            )
                .chain(),),
        );
    }
}
