pub mod buffer;

use bevy::{
    ecs::{query::ROQueryItem, system::SystemParamItem},
    prelude::*,
    render::{
        render_phase::{
            PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::Buffer,
    },
};

const SHADER_PATH: &str = "shaders/chunk.wgsl";

struct DrawQuads;

impl<P: PhaseItem> RenderCommand<P> for DrawQuads {
    type Param = ();
    type ItemQuery = &'static QuadBuffers;
    type ViewQuery = ();

    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(QuadBuffers {
            base,
            instances,
            indirect,
            offset,
            count,
        }) = entity
        {
            pass.set_vertex_buffer(0, base.slice(..));
            pass.set_vertex_buffer(1, instances.slice(..));
            pass.multi_draw_indirect(indirect, *offset, *count);

            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure("QuadBuffers query failure")
        }
    }
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
