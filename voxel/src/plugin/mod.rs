use bevy::prelude::*;

use crate::{
    data::{Chunk, voxel::Voxel},
    mesher::ChunkMeshBuilder,
};

pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut chunk = Chunk::uniform(Voxel { id: 1 });
    chunk.set((0, 0, 0), Voxel { id: 0 });

    let mesher = ChunkMeshBuilder { chunk };

    commands.spawn((
        Mesh3d(meshes.add(mesher.build())),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
        Transform::default(),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::default().looking_at(Vec3::NEG_Y, Vec3::Y),
    ));

    commands.spawn((PointLight::default(), Transform::from_xyz(9.0, 9.0, 9.0)));
}
