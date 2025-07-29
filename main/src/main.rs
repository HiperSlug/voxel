use bevy::prelude::*;
use bevy_flycam::FlyCam;
use voxel::{VoxelPlugin, VoxelViewer, VoxelVolume};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VoxelPlugin)
        .add_systems(Startup, testing)
        .run();
}

pub fn testing(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::default(),
        FlyCam,
        VoxelViewer::new(4),
    ));

    commands.spawn((VoxelVolume::default(), Transform::default()));

    commands.spawn((
        DirectionalLight::default(),
        Transform::default().looking_at(Vec3::NEG_Y, Vec3::Y),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_length(0.5))),
        Transform::default(),
        MeshMaterial3d(materials.add(Color::srgb_u8(255, 255, 255))),
    ));
}
