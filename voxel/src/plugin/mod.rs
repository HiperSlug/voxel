use bevy::{math::U8Vec3, prelude::*};

use crate::{
    data::{
        Chunk, VoxelVolume,
        octree::Octree,
        voxel::Voxel,
        voxel_volume::{VoxelViewer, load_queue, update_voxel_volumes},
    },
    mesher,
};

pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(First, (update_voxel_volumes, load_queue).chain());
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut chunk = Chunk {
        data: Octree::uniform(Voxel { id: 1 }),
    };
    chunk.data.set(U8Vec3::new(0, 0, 0), Voxel { id: 0 });

    let mesh = mesher::build(&chunk);

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
        Transform::default(),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::default().looking_at(Vec3::NEG_Y, Vec3::Y),
    ));

    commands.spawn((VoxelViewer { load_distance: 20 }, Transform::default()));

    commands.spawn((VoxelVolume::new(), Transform::default()));

    commands.spawn((PointLight::default(), Transform::from_xyz(9.0, 9.0, 9.0)));
}
