use crate::{
    chunk::{ChunkFlag, ChunkConstructorTask, ChunkData, ChunkMesherTask},
    data::{
        chunk::{self, Chunk},
        voxel::{self, Voxel},
    },
};
use bevy::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    render::mesh::{Indices, PrimitiveTopology},
};
use block_mesh::{GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG, greedy_quads};

#[derive(Debug, Component)]
pub struct NullMesh;

pub fn handle_chunk_meshing(
    mut commands: Commands,
    query: Query<
        (Entity, &ChunkData),
        (
            With<ChunkFlag>,
            Without<Mesh3d>,
            Without<ChunkConstructorTask>,
            Without<ChunkMesherTask>,
            Without<NullMesh>,
        ),
    >,
) {
    for (entity, chunk_data) in query {
        match &chunk_data.0 {
            Chunk::Uniform(_) => {
                commands.entity(entity).insert(NullMesh);
            }
            Chunk::Mixed(voxels) => {
                let guard = voxels.load();
                commands
                    .entity(entity)
                    .insert(ChunkMesherTask::new(move || mesh(&**guard)));
            }
        };
    }
}

pub fn mesh(voxels: &[Voxel]) -> Mesh {
    let mut buffer = GreedyQuadsBuffer::new(chunk::PADDED_VOLUME_IN_VOXELS);
    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;
    greedy_quads(
        voxels,
        &chunk::Shape {},
        [0; 3],
        [chunk::PADDED_LENGTH_IN_VOXELS - 1; 3],
        &faces,
        &mut buffer,
    );

    let quad_count = buffer.quads.num_quads();
    let index_count = quad_count * 6;
    let vertex_count = quad_count * 4;

    let mut indices = Vec::with_capacity(index_count);
    let mut positions = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);

    let mut index_offset = 0;

    for (unoriented_group, face) in buffer.quads.groups.into_iter().zip(faces) {
        for quad in unoriented_group.into_iter() {
            indices.extend(face.quad_mesh_indices(index_offset));
            index_offset += 4;
            positions.extend(face.quad_mesh_positions(&quad, voxel::LENGTH));
            uvs.extend(face.tex_coords(RIGHT_HANDED_Y_UP_CONFIG.u_flip_face, true, &quad));
            normals.extend(face.quad_mesh_normals());
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}
