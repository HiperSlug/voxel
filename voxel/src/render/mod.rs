pub mod buffer;

use std::{cell::LazyCell, cmp::Ordering};

use bevy::{
    ecs::{
        query::ROQueryItem,
        system::{
            SystemParamItem,
            lifetimeless::{Read, SRes},
        },
    },
    prelude::*,
    render::{
        batching::gpu_preprocessing::{IndirectParametersBuffers, IndirectParametersNonIndexed},
        render_phase::{
            PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{Buffer, BufferInitDescriptor, BufferUsages, IndexFormat},
        renderer::RenderDevice,
    },
};
use bytemuck::{Pod, Zeroable, cast_slice};
use math::prelude::*;

use crate::{
    chunk::{ChunkMesh, ChunkPos, VoxelQuad},
    render::buffer::GpuBufferAllocator,
    terrain::Terrain,
};

const SHADER_PATH: &str = "shaders/chunk.wgsl";

#[derive(Resource)]
struct BaseQuadBuffers {
    vertex: Buffer,
    index: Buffer,
}

impl FromWorld for BaseQuadBuffers {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();

        #[repr(C)]
        #[derive(Clone, Copy, Zeroable, Pod)]
        struct Vertex {
            position: [f32; 3],
            normal: [f32; 3],
            uvs: [f32; 2],
        }

        const VERTICES: [Vertex; 4] = [
            Vertex {
                position: [-0.5, 0.0, -0.5],
                normal: [0.0, 1.0, 0.0],
                uvs: [0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.0, 0.5],
                normal: [0.0, 1.0, 0.0],
                uvs: [0.0, 1.0],
            },
            Vertex {
                position: [0.5, 0.0, 0.5],
                normal: [0.0, 1.0, 0.0],
                uvs: [1.0, 1.0],
            },
            Vertex {
                position: [0.5, 0.0, -0.5],
                normal: [0.0, 1.0, 0.0],
                uvs: [1.0, 0.0],
            },
        ];

        const INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let vertex = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("BaseQuad instance"),
            usage: BufferUsages::VERTEX,
            contents: cast_slice(&VERTICES),
        });

        let index = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("BaseQuad instance"),
            usage: BufferUsages::INDEX,
            contents: cast_slice(&INDICES),
        });

        Self { vertex, index }
    }
}

struct DrawQuads;

impl<P: PhaseItem> RenderCommand<P> for DrawQuads {
    type Param = (SRes<GpuBufferAllocator<VoxelQuad>>, SRes<BaseQuadBuffers>);
    type ItemQuery = (Read<Terrain>, Read<GlobalTransform>);
    type ViewQuery = Read<GlobalTransform>;

    fn render<'w>(
        _item: &P,
        view_transform: ROQueryItem<'w, Self::ViewQuery>,
        item_query: Option<ROQueryItem<'w, Self::IteQuery>>,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (terrain, terrain_transform) = item_query.unwrap();
        let view_chunk_pos =
            ChunkPos::from_world(view_transform.translation() - terrain_transform.translation());

        let (gpu_buffer_allocator, base_quad_buffers) = param;
        let gpu_buffer_allocator = gpu_buffer_allocator.into_inner();
        let base_quad_buffers = base_quad_buffers.into_inner();

        let mut indirect_args_slab_map = Vec::new();
        indirect_args_slab_map.resize_with(gpu_buffer_allocator.slabs().len(), Vec::new);

        for (chunk_pos, chunk_mesh_opt) in terrain.chunk_mesh_map.iter().map(|r| r.pair()) {
            if let Some(chunk_mesh) = chunk_mesh_opt {
                let visible = |signed_axis: &SignedAxis| match signed_axis {
                    PosX => view_chunk_pos.x >= chunk_pos.x,
                    NegX => view_chunk_pos.x <= chunk_pos.x,
                    PosY => view_chunk_pos.y >= chunk_pos.y,
                    NegY => view_chunk_pos.y <= chunk_pos.y,
                    PosZ => view_chunk_pos.z >= chunk_pos.z,
                    NegZ => view_chunk_pos.z <= chunk_pos.z,
                };

                for signed_axis in SignedAxis::ALL.iter().copied().filter(visible) {
                    let Some(offset) = chunk_mesh.offsets[signed_axis].map(|o| o.get()) else {
                        continue;
                    };
                    let size = chunk_mesh
                        .offsets
                        .iter()
                        .skip(signed_axis.into_usize())
                        .find_map(|(_, o)| o.as_ref().map(|o| o.get() - offset))
                        .unwrap_or_else(|| chunk_mesh.allocation.size());

                    let args = DrawIndirect {
                        vertex_count: 4,
                        first_vertex: 0,
                        first_instance: offset,
                        instance_count: size,
                    };

                    indirect_args_slab_map[chunk_mesh.allocation.slab_index()].push(args);
                }
            }
        }

        pass.set_vertex_buffer(0, base_quad_buffers.vertex.slice(..));
        pass.set_index_buffer(base_quad_buffers.index.slice(..), 0, IndexFormat::Uint16);

        for (args, gpu_slab_opt) in indirect_args_slab_map
            .into_iter()
            .zip(gpu_buffer_allocator.slabs())
        {
            if args.is_empty() {
                continue;
            }
            let buffer = &gpu_slab_opt.as_ref().unwrap().buffer;

            // indirect 
            pass.set_vertex_buffer(1, buffer.slice(..));

            pass.multi_draw_indirect(indirect_buffer, indirect_offset, count);
        }

        RenderCommandResult::Success

        // {

        //     pass.set_vertex_buffer(1, instances.slice(..));
        //     pass.multi_draw_indirect(indirect, *offset, *count);

        //
        // } else {
        //     RenderCommandResult::Failure("QuadBuffers query failure")
        // }
    }
}

#[repr(C)]
struct DrawIndirect {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
    // TODO: enable that feature
    // has to be 0, unless [`Features::INDIRECT_FIRST_INSTANCE`] is enabled.
}

type DrawQuadsCommands = (SetItemPipeline, DrawQuads);

// struct QuadsPipeline {
//     variants: Vari,
// }

struct QuadPlugin;

impl Plugin for QuadPlugin {
    fn build(&self, app: &mut App) {
        // app.sub_app_mut(RenderApp).init_resource();
    }
}
