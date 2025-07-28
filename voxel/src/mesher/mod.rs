use crate::data::{
    brick::{self, Brick},
    chunk::{self, Chunk},
    voxel,
};
use bevy::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    render::mesh::{Indices, PrimitiveTopology},
};

pub fn mesh(chunk: &Chunk) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    let mut uvs = Vec::new(); // todo

    let mut index_offset = 0;

    for (b_offset, brick) in chunk
        .bricks
        .iter()
        .enumerate()
        .map(|(i, b)| (chunk::index_to_position(i), b))
    {
        match brick {
            Brick::Uniform(of) => {
                let visibility = of.visibility();
                let pos = b_offset;
                let end = b_offset + Vec3::splat(brick::LENGTH);
                if visibility.pos_x {
                    positions.extend([
                        [end.x, pos.y, pos.z],
                        [end.x, end.y, pos.z],
                        [end.x, end.y, end.z],
                        [end.x, pos.y, end.z],
                    ]);
                    normals.extend([[1.0, 0.0, 0.0]; 4]);
                    uvs.extend([[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]]);
                    indices.extend([0, 1, 2, 2, 3, 0].into_iter().map(|i| index_offset + i));
                    index_offset += 4;
                }
                if visibility.neg_x {
                    positions.extend([
                        [pos.x, pos.y, end.z],
                        [pos.x, end.y, end.z],
                        [pos.x, end.y, pos.z],
                        [pos.x, pos.y, pos.z],
                    ]);
                    normals.extend([[-1.0, 0.0, 0.0]; 4]);
                    uvs.extend([[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]]);
                    const INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];
                    indices.extend(INDICES.into_iter().map(|i| index_offset + i));
                    index_offset += 4;
                }
                if visibility.pos_y {
                    positions.extend([
                        [end.x, end.y, pos.z],
                        [pos.x, end.y, pos.z],
                        [pos.x, end.y, end.z],
                        [end.x, end.y, end.z],
                    ]);
                    normals.extend([[0.0, 1.0, 0.0]; 4]);
                    uvs.extend([[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]]);
                    indices.extend([0, 1, 2, 2, 3, 0].into_iter().map(|i| index_offset + i));
                    index_offset += 4;
                }
                if visibility.neg_y {
                    positions.extend([
                        [end.x, pos.y, end.z],
                        [pos.x, pos.y, end.z],
                        [pos.x, pos.y, pos.z],
                        [end.x, pos.y, pos.z],
                    ]);
                    normals.extend([[0.0, -1.0, 0.0]; 4]);
                    uvs.extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
                    indices.extend([0, 1, 2, 2, 3, 0].into_iter().map(|i| index_offset + i));
                    index_offset += 4;
                }
                if visibility.pos_z {
                    positions.extend([
                        [pos.x, pos.y, end.z],
                        [end.x, pos.y, end.z],
                        [end.x, end.y, end.z],
                        [pos.x, end.y, end.z],
                    ]);
                    normals.extend([[0.0, 0.0, 1.0]; 4]);
                    uvs.extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
                    indices.extend([0, 1, 2, 2, 3, 0].into_iter().map(|i| index_offset + i));
                    index_offset += 4;
                }
                if visibility.neg_z {
                    positions.extend([
                        [pos.x, end.y, pos.z],
                        [end.x, end.y, pos.z],
                        [end.x, pos.y, pos.z],
                        [pos.x, pos.y, pos.z],
                    ]);
                    normals.extend([[0.0, 0.0, -1.0]; 4]);
                    uvs.extend([[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]]);
                    indices.extend([0, 1, 2, 2, 3, 0].into_iter().map(|i| index_offset + i));
                    index_offset += 4;
                }
            }
            Brick::NonUniform(voxels) => {
                // todo!() texture data
                for (v_offset, visibility) in voxels
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (brick::index_to_position(i), v))
                    .map(|(p, v)| (p, v.visibility()))
                {
                    // todo!() greedy meshing
                    let pos = v_offset + b_offset;
                    let end = pos + Vec3::splat(voxel::LENGTH);
                    if visibility.pos_x {
                        positions.extend([
                            [end.x, pos.y, pos.z],
                            [end.x, end.y, pos.z],
                            [end.x, end.y, end.z],
                            [end.x, pos.y, end.z],
                        ]);
                        normals.extend([[1.0, 0.0, 0.0]; 4]);
                        uvs.extend([[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]]);
                        indices.extend([0, 1, 2, 2, 3, 0].into_iter().map(|i| index_offset + i));
                        index_offset += 4;
                    }
                    if visibility.neg_x {
                        positions.extend([
                            [pos.x, pos.y, end.z],
                            [pos.x, end.y, end.z],
                            [pos.x, end.y, pos.z],
                            [pos.x, pos.y, pos.z],
                        ]);
                        normals.extend([[-1.0, 0.0, 0.0]; 4]);
                        uvs.extend([[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]]);
                        const INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];
                        indices.extend(INDICES.into_iter().map(|i| index_offset + i));
                        index_offset += 4;
                    }
                    if visibility.pos_y {
                        positions.extend([
                            [end.x, end.y, pos.z],
                            [pos.x, end.y, pos.z],
                            [pos.x, end.y, end.z],
                            [end.x, end.y, end.z],
                        ]);
                        normals.extend([[0.0, 1.0, 0.0]; 4]);
                        uvs.extend([[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]]);
                        indices.extend([0, 1, 2, 2, 3, 0].into_iter().map(|i| index_offset + i));
                        index_offset += 4;
                    }
                    if visibility.neg_y {
                        positions.extend([
                            [end.x, pos.y, end.z],
                            [pos.x, pos.y, end.z],
                            [pos.x, pos.y, pos.z],
                            [end.x, pos.y, pos.z],
                        ]);
                        normals.extend([[0.0, -1.0, 0.0]; 4]);
                        uvs.extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
                        indices.extend([0, 1, 2, 2, 3, 0].into_iter().map(|i| index_offset + i));
                        index_offset += 4;
                    }
                    if visibility.pos_z {
                        positions.extend([
                            [pos.x, pos.y, end.z],
                            [end.x, pos.y, end.z],
                            [end.x, end.y, end.z],
                            [pos.x, end.y, end.z],
                        ]);
                        normals.extend([[0.0, 0.0, 1.0]; 4]);
                        uvs.extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
                        indices.extend([0, 1, 2, 2, 3, 0].into_iter().map(|i| index_offset + i));
                        index_offset += 4;
                    }
                    if visibility.neg_z {
                        positions.extend([
                            [pos.x, end.y, pos.z],
                            [end.x, end.y, pos.z],
                            [end.x, pos.y, pos.z],
                            [pos.x, pos.y, pos.z],
                        ]);
                        normals.extend([[0.0, 0.0, -1.0]; 4]);
                        uvs.extend([[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]]);
                        indices.extend([0, 1, 2, 2, 3, 0].into_iter().map(|i| index_offset + i));
                        index_offset += 4;
                    }
                }
            }
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

