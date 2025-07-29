use bevy::prelude::*;

use crate::chunk;

pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, testing_setup)
            .add_systems(Update, (
                chunk::poll_chunk_constructors,
                chunk::poll_chunk_meshers
            ));
    }
}

