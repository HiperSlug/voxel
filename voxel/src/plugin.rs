use bevy::prelude::*;

use crate::{chunk, mesher, voxel_volume};

pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TexturePlugin).add_systems(
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
