use crate::data::Chunk;
use bevy::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    render::mesh::{Indices, PrimitiveTopology},
};

const NORMALS_AND_CORNERS: [(Vec3, [Vec3; 4]); 6] = [
    (
        Vec3::X,
        [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
        ],
    ),
    (
        Vec3::NEG_X,
        [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 1.0),
        ],
    ),
    (
        Vec3::Y,
        [
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
        ],
    ),
    (
        Vec3::NEG_Y,
        [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 1.0),
        ],
    ),
    (
        Vec3::Z,
        [
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
        ],
    ),
    (
        Vec3::NEG_Z,
        [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
        ],
    ),
];

const FACE_UVS: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];

#[derive(Default)]
struct MeshData {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
    uvs: Vec<[f32; 2]>,
    index_offset: u32,
}

impl MeshData {
    pub fn add_cube(&mut self, position: Vec3, size: f32) {
        for (normal, corners) in NORMALS_AND_CORNERS {
            for (corner, uv) in corners.into_iter().zip(FACE_UVS) {
                let corner_position = position + (corner * size);
                self.positions.push(corner_position.into());
                self.normals.push(normal.into());
                self.uvs.push(uv);
            }

            let base_index = self.index_offset;

            self.indices.extend([
                base_index,
                base_index + 1,
                base_index + 2,
                base_index,
                base_index + 2,
                base_index + 3,
            ]);

            self.index_offset += 4;
        }
    }
}

#[derive(Default)]
pub struct ChunkMeshBuilder {
    pub chunk: Chunk,
}

impl MeshBuilder for ChunkMeshBuilder {
    fn build(&self) -> Mesh {
        let mut data = MeshData::default();

        for (position, size, voxel) in self.chunk.leaf_iter() {
            if voxel.is_solid() {
                data.add_cube(position, size);
            }
        }

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, data.uvs);
        mesh.insert_indices(Indices::U32(data.indices));
        mesh
    }
}
