use crate::data::volume::{self, VoxelVolume, voxel_viewer::VoxelViewer};
use bevy::prelude::*;
pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (
                volume::active_chunks,
                (volume::poll_chunk_constructor, volume::poll_chunk_mesher),
            )
                .chain(),
        );
    }
}

pub fn setup(mut commands: Commands) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::default().looking_at(Vec3::NEG_Y, Vec3::Y),
    ));

    commands.spawn((VoxelViewer { view_distance: 3 }, Transform::default()));

    commands.spawn((VoxelVolume::default(), Transform::default()));

    commands.spawn((PointLight::default(), Transform::from_xyz(8.0, 0.0, 8.0)));
}
