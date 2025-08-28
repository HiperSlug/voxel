use bevy::{
    core_pipeline::core_3d::CORE_3D_DEPTH_FORMAT,
    pbr::{MeshPipeline, MeshPipelineKey, MeshPipelineViewLayoutKey},
    prelude::*,
    render::{
        mesh::{MeshVertexBufferLayoutRef, PrimitiveTopology, VertexBufferLayout, VertexFormat},
        render_resource::{
            ColorTargetState, ColorWrites, CompareFunction, DepthStencilState, Face, FragmentState,
            MultisampleState, PrimitiveState, RenderPipelineDescriptor, SpecializedMeshPipeline,
            SpecializedMeshPipelineError, TextureFormat, VertexState, VertexStepMode,
        },
    },
};

const SHADER_PATH: &str = "shaders/voxel_quad.wgsl";

#[derive(Resource)]
pub struct VoxelQuadPipeline {
    mesh_pipeline: MeshPipeline,
    shader_handle: Handle<Shader>,
}

pub fn init_voxel_quad_pipeline(
    mut commands: Commands,
    mesh_pipeline: Res<MeshPipeline>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(VoxelQuadPipeline {
        mesh_pipeline: mesh_pipeline.clone(),
        shader_handle: asset_server.load(SHADER_PATH),
    });
}

impl SpecializedMeshPipeline for VoxelQuadPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let vertex_buffer_layout = layout.0.layout().clone();

        let instance_buffer_layout = VertexBufferLayout::from_vertex_formats(
            VertexStepMode::Instance,
            vec![VertexFormat::Uint32x2],
        );

        let view_layout = self
            .mesh_pipeline
            .get_view_layout(MeshPipelineViewLayoutKey::from(key));

        Ok(RenderPipelineDescriptor {
            label: Some("VoxelQuadPipeline".into()),
            vertex: VertexState {
                shader: self.shader_handle.clone(),
                entry_point: "vertex".into(),
                shader_defs: vec![],
                buffers: vec![vertex_buffer_layout, instance_buffer_layout],
            },
            fragment: Some(FragmentState {
                shader: self.shader_handle.clone(),
                entry_point: "fragment".into(),
                shader_defs: vec![],
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout: vec![view_layout.clone()],
            push_constant_ranges: vec![],
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                cull_mode: Some(Face::Back),
                ..default()
            },
            depth_stencil: Some(DepthStencilState {
                format: CORE_3D_DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::LessEqual,
                stencil: default(),
                bias: default(),
            }),
            multisample: MultisampleState {
                count: key.msaa_samples(),
                ..default()
            },
            zero_initialize_workgroup_memory: true,
        })
    }
}
