use bevy::prelude::*;
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use voxel::{VoxelPlugin, VoxelViewer, VoxelVolume};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VoxelPlugin)
        .add_plugins(NoCameraPlayerPlugin)
        .add_systems(Startup, testing)
        .add_systems(Update, moving)
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
        VoxelViewer::new(32),
    ));

    commands.spawn((
        VoxelVolume::default(),
        Transform::default(),
        Visibility::Visible,
    ));

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

pub fn moving(query: Query<&mut Transform, With<VoxelVolume>>, time: Res<Time>) {
    for mut t in query {
        t.translation += Vec3::new(1.0, 0.0, 0.0) * time.delta_secs()
    }
}
