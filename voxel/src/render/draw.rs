use bevy::{
    core_pipeline::core_3d::Transparent3d, ecs::{
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes}, SystemParamItem
        },
    }, mesh::{MeshVertexBufferLayoutRef, VertexBufferLayout, VertexFormat}, pbr::{MeshPipeline, MeshPipelineKey, RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup}, prelude::*, render::{
        mesh::RenderMesh, render_asset::RenderAssets, render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases
        }, render_resource::{PipelineCache, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipeline, SpecializedMeshPipelineError, SpecializedMeshPipelines, VertexAttribute, VertexStepMode}, sync_world::MainEntity, view::ExtractedView, Render, RenderApp, RenderStartup, RenderSystems
    }
};
use bytemuck::{Pod, Zeroable};

use crate::{
    chunk::VoxelQuad,
    render::{alloc_buffer::AllocBufferPlugin, BaseQuadBuffer, IndirectTerrainBuffers},
};

const SHADER_ASSET_PATH: &str = "";

struct CustomMaterialPlugin;

impl Plugin for CustomMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AllocBufferPlugin::<VoxelQuad>::default())
            .sub_app_mut(RenderApp)
            .add_render_command()
            .add_systems(RenderStartup, todo!()) //init custom pipeline
            .add_systems(Render, (
                queue_custom.in_set(RenderSystems::QueueMeshes),
                prepare_instance_buffers.in_set(RenderSystems::PrepareResources)
            ))
            ;
            
    }
}

// ill unmod later
use pipeline::*;
mod pipeline {
    use super::*;

    #[derive(Resource)]
    pub struct CustomPipeline {
        shader: Handle<Shader>,
        mesh_pipeline: MeshPipeline,
    }

    pub fn init_custom_pipeline(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mesh_pipeline: Res<MeshPipeline>,
    ) {
        commands.insert_resource(CustomPipeline {
            shader: asset_server.load(SHADER_ASSET_PATH),
            mesh_pipeline: mesh_pipeline.clone(),
        })
    }

    impl SpecializedMeshPipeline for CustomPipeline {
        type Key = MeshPipelineKey;
        
        fn specialize(
            &self,
            key: Self::Key,
            layout: &MeshVertexBufferLayoutRef,
        ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
            let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;

            descriptor.vertex.buffers.push(VertexBufferLayout {
                array_stride: size_of::<VoxelQuad>() as u64,
                step_mode: VertexStepMode::Instance,
                attributes: vec![
                    // pos: IVec3
                    VertexAttribute {
                        format: VertexFormat::Sint32x3,
                        offset: 0,
                        shader_location: 3,
                    },
                    // data: { width: u6, height: u6, tex_index: u16, signed_axis: u3 }
                    VertexAttribute {
                        format: VertexFormat::Uint32,
                        offset: VertexFormat::Sint32x3.size(),
                        shader_location: 4,
                    },
                ]
            });

            descriptor.vertex.shader = self.shader.clone();
            descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();

            // as long as this is a specilized mesh pipeline I cannot, for example, change PrimitiveState to Strips.

            Ok(descriptor)
        }
    }
}


// enqueues all PhaseItems
fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<CustomPipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<CustomPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<RenderMesh>>,
    render_mesh_instances: Res<RenderMeshInstances>,
    material_meshes: Query<(Entity, &MainEntity), With<InstanceMaterialData>>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    views: Query<(&ExtractedView, &Msaa)>,
) {
    let draw_custom = transparent_3d_draw_functions.read().id::<DrawCustom>();

    for (view, msaa) in &views {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view.retained_view_entity)
        else {
            continue;
        };

        let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());

        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        let rangefinder = view.rangefinder3d();
        for (entity, main_entity) in &material_meshes {
            let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(*main_entity)
            else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };
            let key =
                view_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology());
            let pipeline = pipelines
                .specialize(&pipeline_cache, &custom_pipeline, key, &mesh.layout)
                .unwrap();
            transparent_phase.add(Transparent3d {
                entity: (entity, *main_entity),
                pipeline,
                draw_function: draw_custom,
                distance: rangefinder.distance_translation(&mesh_instance.translation),
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed: true,
            });
        }
    }
}

// dont need to do this
pub fn prepare_instance_buffers() {}

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
