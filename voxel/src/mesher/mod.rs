use bevy::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    render::mesh::{Indices, PrimitiveTopology},
};

use crate::data::chunk::Chunk;
use crate::data::utils::subdivide_index;
use crate::data::{brick, chunk, voxel};

pub fn mesh(chunk: &Chunk) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    let mut uvs = Vec::new();

    let mut index_offset = 0;

    let iterator = chunk
        .bricks
        .iter()
        .enumerate()
        .filter_map(|(i, b)| {
            let pos = brick::LENGTH * subdivide_index::<{ chunk::BITS }>(i).as_vec3();
            b.voxels.as_ref().map(|v| (pos, v))
        })
        .flat_map(|(brick_pos, v)| {
            v.iter().enumerate().map(move |(i, v)| {
                let pos = voxel::LENGTH * subdivide_index::<{ brick::BITS }>(i).as_vec3();
                if (14..17).contains(&i) {
                    println!("Voxel pos: {pos}");
                }
                (pos + brick_pos, v)
            })
        });

    let vec = iterator.clone().collect::<Vec<_>>();
    println!("VOXELS IN CHUNK: {}", vec.len()); // 2^14

    for (position, voxel) in iterator {
        if voxel.is_empty() {
            continue;
        }

        let min = position;
        let max = position + Vec3::splat(voxel::LENGTH);

        let data = &[
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

        positions.extend(data.iter().map(|(p, _, _)| *p));
        normals.extend(data.iter().map(|(_, n, _)| *n));
        uvs.extend(data.iter().map(|(_, _, uv)| *uv));

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

    println!("{}", positions.len());

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
