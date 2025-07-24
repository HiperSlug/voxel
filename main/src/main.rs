use bevy::prelude::*;
use bevy_flycam::PlayerPlugin;
use voxel::VoxelPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VoxelPlugin)
        .add_plugins(PlayerPlugin)
        .run();
}
