use bevy::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    render::mesh::{Indices, PrimitiveTopology},
};

use crate::data::Chunk;

pub struct ChunkMeshBuilder {
    pub chunk: Chunk,
}

impl MeshBuilder for ChunkMeshBuilder {
    fn build(&self) -> Mesh {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();
        let mut uvs = Vec::new();

        let mut index_offset = 0;

        for (position, size, voxel) in self.chunk.leaf_vec() {
            if voxel.is_empty() {
                continue;
            }

            let min = position;
            let max = position + Vec3::splat(size);

            let vertices = &[
                // Front
                ([min.x, min.y, max.z], [0.0, 0.0, 1.0], [0.0, 0.0]),
                ([max.x, min.y, max.z], [0.0, 0.0, 1.0], [1.0, 0.0]),
                ([max.x, max.y, max.z], [0.0, 0.0, 1.0], [1.0, 1.0]),
                ([min.x, max.y, max.z], [0.0, 0.0, 1.0], [0.0, 1.0]),
                // Back
                ([min.x, max.y, min.z], [0.0, 0.0, -1.0], [1.0, 0.0]),
                ([max.x, max.y, min.z], [0.0, 0.0, -1.0], [0.0, 0.0]),
                ([max.x, min.y, min.z], [0.0, 0.0, -1.0], [0.0, 1.0]),
                ([min.x, min.y, min.z], [0.0, 0.0, -1.0], [1.0, 1.0]),
                // Right
                ([max.x, min.y, min.z], [1.0, 0.0, 0.0], [0.0, 0.0]),
                ([max.x, max.y, min.z], [1.0, 0.0, 0.0], [1.0, 0.0]),
                ([max.x, max.y, max.z], [1.0, 0.0, 0.0], [1.0, 1.0]),
                ([max.x, min.y, max.z], [1.0, 0.0, 0.0], [0.0, 1.0]),
                // Left
                ([min.x, min.y, max.z], [-1.0, 0.0, 0.0], [1.0, 0.0]),
                ([min.x, max.y, max.z], [-1.0, 0.0, 0.0], [0.0, 0.0]),
                ([min.x, max.y, min.z], [-1.0, 0.0, 0.0], [0.0, 1.0]),
                ([min.x, min.y, min.z], [-1.0, 0.0, 0.0], [1.0, 1.0]),
                // Top
                ([max.x, max.y, min.z], [0.0, 1.0, 0.0], [1.0, 0.0]),
                ([min.x, max.y, min.z], [0.0, 1.0, 0.0], [0.0, 0.0]),
                ([min.x, max.y, max.z], [0.0, 1.0, 0.0], [0.0, 1.0]),
                ([max.x, max.y, max.z], [0.0, 1.0, 0.0], [1.0, 1.0]),
                // Bottom
                ([max.x, min.y, max.z], [0.0, -1.0, 0.0], [0.0, 0.0]),
                ([min.x, min.y, max.z], [0.0, -1.0, 0.0], [1.0, 0.0]),
                ([min.x, min.y, min.z], [0.0, -1.0, 0.0], [1.0, 1.0]),
                ([max.x, min.y, min.z], [0.0, -1.0, 0.0], [0.0, 1.0]),
            ];

            positions.extend(vertices.iter().map(|(p, _, _)| *p));
            normals.extend(vertices.iter().map(|(_, n, _)| *n));
            uvs.extend(vertices.iter().map(|(_, _, uv)| *uv));

            for _ in 0..6 {
                for i in [0, 1, 2, 2, 3, 0] {
                    indices.push(i + index_offset);
                }
                index_offset += 4;
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
}
