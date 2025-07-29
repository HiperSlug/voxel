use bevy::prelude::*;

use crate::ecs::{
    chunk::{poll_chunk_constructors, poll_chunk_meshers},
    voxel_viewer::VoxelViewer,
    voxel_volume::{VoxelVolume, chunk_loading},
};

pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, testing_setup).add_systems(
            Update,
            (
                (poll_chunk_constructors, poll_chunk_meshers).chain(),
                chunk_loading,
            ),
        );
    }
}

pub fn testing_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((VoxelViewer { view_distance: 4 }, Transform::default()));

    commands.spawn((VoxelVolume::default(), Transform::default()));

    commands.spawn((
        DirectionalLight::default(),
        Transform::default().looking_at(Vec3::NEG_Y, Vec3::Y),
    ));

    commands.spawn((PointLight::default(), Transform::from_xyz(0.0, 1.0, 0.0)));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_length(0.5))),
        Transform::default(),
        MeshMaterial3d(materials.add(Color::srgb_u8(255, 255, 255))),
    ));
}
