use bevy::prelude::*;

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
    // let chunk = MatrixChunk::filled();

    // commands.spawn((
    //     Mesh3d(meshes.add(mesh_chunk(chunk))),
    //     MeshMaterial3d(materials.add(Color::srgb(0.7, 0.7, 0.7))),
    //     Transform::default(),
    // ));
}
