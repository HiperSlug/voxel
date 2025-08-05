use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
};

use crate::chunk::ChunkFlag;

use super::texture_array::{SharedTextureArrayMaterial, TextureArrayMaterial};

use block_mesh::SignedAxis;

#[derive(Debug, Component)]
pub struct MeshedFlag;

#[derive(Debug, Component)]
pub struct ChunkMesherTask(Task<Mesh>);

impl ChunkMesherTask {
    pub fn new<M>(mesher: M) -> Self
    where
        M: Fn() -> Mesh + Send + 'static,
    {
        let thread_pool = AsyncComputeTaskPool::get();
        let task = thread_pool.spawn(async move { mesher() });
        Self(task)
    }
}

pub fn poll_chunk_meshers(
    mut commands: Commands,
    query: Query<(Entity, &mut ChunkMesherTask)>,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<SharedTextureArrayMaterial>,
) {
    for (entity, mut task) in query {
        if let Some(mesh) = block_on(poll_once(&mut task.0)) {
            commands
                .entity(entity)
                .remove::<ChunkMesherTask>()
                .insert(Mesh3d(meshes.add(mesh)))
                .entry::<MeshMaterial3d<TextureArrayMaterial>>()
                .or_insert(MeshMaterial3d(material.0.clone()));
        }
    }
}

pub fn handle_chunk_meshing(
    mut commands: Commands,
    query: Query<(Entity, &ChunkData), (With<ChunkFlag>, Without<MeshedFlag>)>,
    shared_block_lib: Res<SharedBlockLibrary>,
) {
    for (entity, chunk_data) in query {
        match &chunk_data.0 {
            Chunk::Uniform(_) => {
                commands.entity(entity).insert(ChunkMesh);
            }
            Chunk::Mixed(voxels) => {
                let vox_guard = voxels.load();
                let lib_guard = shared_block_lib.0.load();
                commands.entity(entity).insert(ChunkMesh);
                commands
                    .entity(entity)
                    .insert(ChunkMesherTask::new(move || {
                        mesh(&**vox_guard, &**lib_guard)
                    }));
            }
        };
    }
}

pub fn mesh(voxels: &[Voxel], block_lib: &BlockLibrary) -> Vec<(Handle<GenericMaterial>, Mesh)> {
    // TODO: prealloc buffer b/c why not
    let mut buffer = GreedyQuadsBuffer::new(chunk::PADDED_VOLUME_IN_VOXELS);
    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

    greedy_quads(
        voxels,
        &chunk::Shape {},
        [0; 3],
        [chunk::PADDED_LENGTH_IN_VOXELS - 1; 3],
        &faces,
        &mut buffer,
        block_lib,
    );

    let mut mesh_map: HashMap<_, MeshData> = HashMap::new();

    for (unoriented_group, face) in buffer.quads.groups.into_iter().zip(&faces) {
        for quad in unoriented_group {
            let index = chunk::linearize(quad.minimum.into());
            let voxel = voxels[index];
            let block_variant = &block_lib.variants[voxel.0 as usize];
            match &block_variant.block_model {
                BlockModel::Cube(c) => {
                    let material_index = c.material_index;
                    let material = &block_lib.materials[material_index];

                    use SignedAxis::*;
                    let texture_uv = match face.signed_axis() {
                        PosX => c.texture_coords.pos_x,
                        NegX => c.texture_coords.neg_x,
                        PosY => c.texture_coords.pos_y,
                        NegY => c.texture_coords.neg_y,
                        PosZ => c.texture_coords.pos_z,
                        NegZ => c.texture_coords.neg_z,
                    };

                    let scaled_uv_origin = texture_uv.as_vec2() / material.size.as_vec2();
                    let uv_size = 1.0 / material.size.as_vec2();

                    let uv = face
                        .tex_coords(RIGHT_HANDED_Y_UP_CONFIG.u_flip_face, true, &quad)
                        .map(|[u, v]| {
                            [
                                scaled_uv_origin.x + (u * uv_size.x),
                                scaled_uv_origin.y + (v * uv_size.y),
                            ]
                        });

                    let entry = mesh_map.entry(material.handle.clone()).or_default();

                    entry
                        .indices
                        .extend(face.quad_mesh_indices(entry.index_offset));
                    entry.index_offset += 4;
                    entry
                        .positions
                        .extend(face.quad_mesh_positions(&quad, voxel::LENGTH));
                    entry.uvs.extend(uv);
                    entry.normals.extend(face.quad_mesh_normals());
                }
            };
        }
    }

    mesh_map
        .into_iter()
        .map(|(handle, builder)| (handle, builder.build()))
        .collect()
}

#[derive(Debug, Default)]
struct MeshData {
    index_offset: u32,
    indices: Vec<u32>,
    positions: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    normals: Vec<[f32; 3]>,
}

impl MeshData {
    fn build(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_indices(Indices::U32(self.indices));

        mesh
    }
}
