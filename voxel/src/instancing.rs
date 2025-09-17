use std::sync::Arc;

use bevy::{
    core_pipeline::core_3d::Transparent3d, ecs::{query::ROQueryItem, system::SystemParamItem}, mesh::MeshVertexBufferLayoutRef, pbr::{MeshPipeline, MeshPipelineKey, RenderMeshInstances}, prelude::*, render::{
        mesh::RenderMesh, render_asset::RenderAssets, render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand, RenderCommandResult, TrackedRenderPass, ViewSortedRenderPhases
        }, render_resource::{
            PipelineCache, RenderPipelineDescriptor, SpecializedMeshPipeline, SpecializedMeshPipelineError, SpecializedMeshPipelines
        }, sync_world::MainEntity, view::ExtractedView, Render, RenderApp, RenderStartup, RenderSystems
    }
};
use dashmap::DashMap;

use crate::chunk::{ChunkMesh, ChunkPos};

// TODO
// NoFrustumCulling, InstanceMaterialData

// *TODO
// Opaque3d,
// Deferring Frees

const SHADER_ASSET_PATH: &str = "instancing.wgsl";

pub struct InstancingPlugin;

impl Plugin for InstancingPlugin {
    fn build(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            // .add_render_command::<Opaque3d, DrawCustom>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .add_systems(RenderStartup, init_custom_pipeline)
            .add_systems(
                Render,
                (
                    queue_custom.in_set(RenderSystems::QueueMeshes),
                    prepare_instance_buffers.in_set(RenderSystems::PrepareResources),
                ),
            );
    }
}

#[derive(Component, Deref)]
struct InstanceMaterialData(Arc<DashMap<ChunkPos, ChunkMesh>>);

#[derive(Resource)]
struct CustomPipeline {
	shader: Handle<Shader>,
	mesh_pipeline: MeshPipeline,
}

impl SpecializedMeshPipeline for CustomPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        todo!();
    }
}

fn init_custom_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mesh_pipeline: Res<MeshPipeline>,
) {
    commands.insert_resource(CustomPipeline {
        shader: asset_server.load(SHADER_ASSET_PATH),
        mesh_pipeline: mesh_pipeline.clone(),
    });
}

struct DrawCustom;

impl<P: PhaseItem> RenderCommand<P> for DrawCustom {
    type ItemQuery = (); // TODO
    type Param = (); // TODO
    type ViewQuery = (); // TODO

    fn render<'w>(
        item: &P,
        view: ROQueryItem<'w, '_, Self::ViewQuery>,
        entity: Option<ROQueryItem<'w, '_, Self::ItemQuery>>,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        todo!();
    }
}

// This doesnt seem to queue each instance individually but queues the entire instance handler thingy.
// I dont know why in the transparent phase. I dont think transparency will work because its ordered based on the origin of the original mesh.

// I think what I eventually want to do is split each chunk into 2 phase items per axis and then each slice merging or whatever.
fn queue_custom(
	transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<CustomPipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<CustomPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<RenderMesh>>,
    render_mesh_instances: Res<RenderMeshInstances>,
    material_meshes: Query<(Entity, &MainEntity), With<InstanceMaterialData>>, // TODO
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
            let key: MeshPipelineKey =
                view_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology());
            let pipeline = pipelines
                .specialize(&pipeline_cache, &custom_pipeline, key, &mesh.layout)
                .unwrap();

			// TODO HERE
            transparent_phase.add(Transparent3d {
                entity: (entity, *main_entity),
                pipeline,
                draw_function: draw_custom,
                distance: rangefinder.distance_translation(&mesh_instance.translation), // I dont think this is useful at all.
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed: true,
            });
        }
    }
}

fn prepare_instance_buffers() {}
