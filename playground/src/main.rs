//! A shader that renders a mesh multiple times in one draw call.
//!
//! Bevy will automatically batch and instance your meshes assuming you use the same
//! `Handle<Material>` and `Handle<Mesh>` for all of your instances.
//!
//! This example is intended for advanced users and shows how to make a custom instancing
//! implementation using bevy's low level rendering api.
//! It's generally recommended to try the built-in instancing before going with this approach.

use std::{ops::Range, u32};

use bevy::{
    asset::embedded_asset,
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        query::ROQueryItem,
        system::{SystemParamItem, lifetimeless::*},
    },
    pbr::{
        MeshPipeline, MeshPipelineKey, RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup,
    },
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::{MeshVertexBufferLayoutRef, RenderMesh, allocator::MeshAllocator},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::*,
        renderer::RenderDevice,
        sync_world::MainEntity,
        view::{ExtractedView, NoFrustumCulling, NoIndirectDrawing},
    },
};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bytemuck::{NoUninit, Pod, Zeroable, bytes_of, cast, cast_mut, cast_slice};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InstanceData([u32; 2]);

impl InstanceData {
    pub fn new(x: u32, y: u32, z: u32, w: u32, h: u32, id: u32) -> Self {
        Self([x | y << 6 | z << 12 | w << 18 | h << 24, id])
    }
}

#[derive(Clone, Component, ExtractComponent)]
struct InstanceDataVec(Vec<InstanceData>);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct ChunkData {
    position: IVec3,
    /// Each window is a slice representing a Face
    slices: [u32; 7],
}

#[derive(Clone, Component, ExtractComponent)]
struct ChunkDataVec(Vec<ChunkData>);

#[derive(Component, Deref)]
struct InstanceBuffer(Buffer);

#[derive(Component)]
struct IndirectCountBuffers {
    // filled with [`DrawIndirect`]
    indirect_buffer: Buffer,
    // filled with a u32 determining the number of [`DrawIndirect`]
    count_buffer: Buffer,
    max_count: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct DrawIndirect {
    vertex_count: u32,   // The number of vertices to draw.
    instance_count: u32, // The number of instances to draw.
    first_vertex: u32,   // The Index of the first vertex to draw.
    first_instance: u32, // The instance ID of the first instance to draw.
                         // has to be 0, unless [`Features::INDIRECT_FIRST_INSTANCE`] is enabled.
}

#[derive(Component)]
struct IndirectLookupBuffer(Buffer);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CustomMaterialPlugin, NoCameraPlayerPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let instance_data_vec = {
        let mut vec = Vec::new();
        for x in 0..16 {
            for y in 0..16 {
                let instance_data = InstanceData::new(x, y, 0, 1, 1, 0);
                vec.push(instance_data);
            }
        }
        vec
    };

    let chunk_data_vec = vec![ChunkData {
        position: IVec3::default(),
        slices: [0, 30, 92, 150, 171, 222, 255],
    }];

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(1.0, 1.0))),
        InstanceDataVec(instance_data_vec),
        ChunkDataVec(chunk_data_vec),
        // NoFrustumCulling,
    ));

    commands.spawn((
        FlyCam,
        Camera3d::default(),
        Transform::default(),
        // I dont know if this is true and I dont know what to do about it:
        // "We need this component because we use `draw_indexed` and `draw`
        // instead of `draw_indirect_indexed` and `draw_indirect` in
        // `DrawMeshInstanced::render`."
        NoIndirectDrawing,
    ));
}

struct CustomMaterialPlugin;

impl Plugin for CustomMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<InstanceDataVec>::default());
        app.add_plugins(ExtractComponentPlugin::<ChunkDataVec>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<SpecializedMeshPipelines<InstancePipeline>>()
            .add_systems(
                Render,
                (
                    queue_custom.in_set(RenderSet::QueueMeshes),
                    prepare_instance_buffers.in_set(RenderSet::PrepareResources),
                ),
            );

        embedded_asset!(app, "instancing.wgsl");
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<InstancePipeline>();
    }
}

fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<InstancePipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<InstancePipeline>>,
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

fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &InstanceDataVec)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in &query {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance"),
            contents: bytemuck::cast_slice(&instance_data.0),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer(buffer));
    }
}

fn prepare_indirect_count_buffers_and_per_draw_indices(
    mut commands: Commands,
    query: Query<(Entity, &ChunkDataVec)>,
    render_device: Res<RenderDevice>,
) {
    // no culling is done here, later I want a compute shader to be doing this.
    for (entity, chunk_data) in &query {
        let count_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("count"),
            contents: cast_slice(&[chunk_data.0.len() as u32 * 6]),
            usage: BufferUsages::STORAGE,
        });
        let indirect_contents = chunk_data
            .0
            .iter()
            .flat_map(|c| {
                c.slices.windows(2).map(|slice| DrawIndirect {
                    first_vertex: 0,
                    vertex_count: 4,
                    first_instance: slice[0],
                    instance_count: slice[1],
                })
            })
            .collect::<Vec<_>>();
        let indirect_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("indirect"),
            contents: cast_slice(&indirect_contents),
            usage: BufferUsages::STORAGE,
        });
        commands.entity(entity).insert(IndirectCountBuffers {
            indirect_buffer,
            count_buffer,
            max_count: u32::MAX,
        });
    }
}

#[derive(Resource)]
struct InstancePipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
}

impl FromWorld for InstancePipeline {
    fn from_world(world: &mut World) -> Self {
        let shader = world
            .resource::<AssetServer>()
            .load("embedded://playground/instancing.wgsl");
        let mesh_pipeline = world.resource::<MeshPipeline>().clone();

        Self {
            shader,
            mesh_pipeline,
        }
    }
}

impl SpecializedMeshPipeline for InstancePipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;

        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: size_of::<InstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![VertexAttribute {
                format: VertexFormat::Uint32x2,
                offset: 0,
                shader_location: 3, // shader locations 0-2 are taken up by Position, Normal and UV attributes
            }],
        });
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        Ok(descriptor)
    }
}

type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    DrawInstances,
);

impl FromWorld for IndirectCountBuffers {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();

        // this should probably be higher
        let max_count = 255;

        let indirect_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("indirect"),
            size: max_count as u64 * size_of::<DrawIndirect>() as u64,
            usage: BufferUsages::INDIRECT,
            mapped_at_creation: false,
        });

        let count_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("count"),
            size: size_of::<u32>() as u64,
            usage: BufferUsages::empty(),
            mapped_at_creation: false,
        });

        Self {
            indirect_buffer,
            count_buffer,
            max_count,
        }
    }
}

struct DrawInstances;

impl<P: PhaseItem> RenderCommand<P> for DrawInstances {
    type Param = (
        SRes<RenderMeshInstances>,
        SRes<MeshAllocator>,
        SRes<IndirectCountBuffers>,
    );
    type ViewQuery = ();
    type ItemQuery = Read<InstanceBuffer>;

    fn render<'w>(
        item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        instance_buffer_opt: Option<ROQueryItem<'w, Self::ItemQuery>>,
        (render_mesh_instances, mesh_allocator, indirect_count_buffers): SystemParamItem<
            'w,
            '_,
            Self::Param,
        >,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_allocator = mesh_allocator.into_inner();

        let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(item.main_entity())
        else {
            return RenderCommandResult::Skip;
        };

        let vertex_buffer_slice = if let Some(mesh_buffer_slice) =
            mesh_allocator.mesh_vertex_slice(&mesh_instance.mesh_asset_id)
        {
            mesh_buffer_slice
                .buffer
                .slice((mesh_buffer_slice.range.start as u64)..(mesh_buffer_slice.range.end as u64))
        } else {
            return RenderCommandResult::Skip;
        };

        let instance_buffer_slice = if let Some(instance_buffer) = instance_buffer_opt {
            instance_buffer.slice(..)
        } else {
            return RenderCommandResult::Skip;
        };

        let IndirectCountBuffers {
            indirect_buffer,
            count_buffer,
            max_count,
        } = indirect_count_buffers.into_inner();

        pass.set_vertex_buffer(0, vertex_buffer_slice);
        pass.set_vertex_buffer(1, instance_buffer_slice);

        pass.multi_draw_indirect_count(indirect_buffer, 0, count_buffer, 0, *max_count);

        RenderCommandResult::Success
    }
}
