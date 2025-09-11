use bevy::{
    ecs::{
        query::ROQueryItem,
        system::{
            SystemParamItem,
            lifetimeless::{Read, SRes},
        },
    },
    pbr::{SetMeshBindGroup, SetMeshViewBindGroup},
    prelude::*,
    render::{
        render_phase::{
            PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::ShaderType,
    },
};
use bytemuck::{Pod, Zeroable};

use crate::{
    chunk::VoxelQuad,
    render::{BaseQuadBuffer, IndirectTerrainBuffers, buffer_allocator::GpuBufferAllocator},
};

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, ShaderType)]
struct DrawIndirect {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    // TODO: handle feature gate
    first_instance: u32,
}

struct DrawVoxelQuads;

impl<P: PhaseItem> RenderCommand<P> for DrawVoxelQuads {
    type Param = (SRes<GpuBufferAllocator<VoxelQuad>>, SRes<BaseQuadBuffer>);
    type ItemQuery = Read<IndirectTerrainBuffers>;
    type ViewQuery = ();

    fn render<'w>(
        _item: &P,
        _view_query: ROQueryItem<'w, Self::ViewQuery>,
        item_query: Option<ROQueryItem<'w, Self::ItemQuery>>,
        (gpu_slabs, base_quad_buffer): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(indirect_buffers) = item_query.map(|w| &w.buffers) else {
            return RenderCommandResult::Failure("No `IndirectTerrainBuffers`");
        };
        let gpu_slabs = gpu_slabs.into_inner().slabs();

        pass.set_vertex_buffer(0, base_quad_buffer.into_inner().buffer.slice(..));

        debug_assert_eq!(indirect_buffers.len(), gpu_slabs.len());

        for (indirect_opt, gpu_slab_opt) in indirect_buffers.iter().zip(gpu_slabs) {
            let Some((indirect_buffer, count)) = indirect_opt else {
                continue;
            };

            let Some(instance_buffer) = gpu_slab_opt.as_ref().map(|s| &s.buffer) else {
                return RenderCommandResult::Failure(
                    "`IndirectTerrainBuffers` pointed at `gpu_slab = None` ",
                );
            };

            pass.set_vertex_buffer(1, instance_buffer.slice(..));
            pass.set_push_constants(stages, offset, data);

            pass.draw_indirect(indirect_buffer, indirect_offset);

            pass.multi_draw_indirect(indirect_buffer, 0, *count);
        }

        RenderCommandResult::Success
    }
}

type DrawQuadsCommands = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    DrawVoxelQuads,
);
