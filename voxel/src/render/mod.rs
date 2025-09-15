pub mod alloc_buffer;
mod draw;
mod pipeline;

use bevy::{
    prelude::*,
    render::{
        render_phase::PhaseItem,
        render_resource::{Buffer, BufferDescriptor, BufferInitDescriptor, BufferUsages},
        renderer::{RenderDevice, RenderQueue},
    },
};
use bytemuck::{Pod, Zeroable, cast_slice, cast_slice_mut};
use std::num::NonZero;

use crate::terrain::ExtractTerrain;

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uvs: [f32; 2],
}

#[derive(Resource)]
struct BaseQuadBuffer {
    buffer: Buffer,
}

impl FromWorld for BaseQuadBuffer {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();

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
                position: [0.5, 0.0, -0.5],
                normal: [0.0, 1.0, 0.0],
                uvs: [1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.0, 0.5],
                normal: [0.0, 1.0, 0.0],
                uvs: [1.0, 1.0],
            },
        ];

        let buffer = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("BaseQuad"),
            usage: BufferUsages::VERTEX,
            contents: cast_slice(&VERTICES),
        });

        Self { buffer }
    }
}

#[derive(Component)]
pub struct IndirectTerrainBuffers {
    buffers: Vec<Option<(Buffer, u32)>>,
}

pub fn a(
    mut commands: Commands,
    query: Query<(Entity, &ExtractTerrain)>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    for (entity, extract_terrain) in query {
        let buffers = extract_terrain
            .visible_quad_ranges
            .iter()
            .map(|ranges| {
                let size = (ranges.len() * size_of::<DrawIndirect>()) as u64;
                let nz_size = NonZero::new(size)?;

                let buffer = device.create_buffer(&BufferDescriptor {
                    label: Some("IndirectBuffer"),
                    usage: BufferUsages::INDIRECT,
                    mapped_at_creation: false,
                    size,
                });

                {
                    let mut view = queue.write_buffer_with(&buffer, 0, nz_size).unwrap();
                    let indirect_args: &mut [DrawIndirect] = cast_slice_mut(&mut view);

                    for (indirect, range) in indirect_args.iter_mut().zip(ranges) {
                        *indirect = DrawIndirect {
                            first_vertex: 0,
                            vertex_count: 4,
                            first_instance: range.0,
                            instance_count: range.1,
                        };
                    }
                }

                Some((buffer, ranges.len() as u32))
            })
            .collect::<Vec<_>>();

        let buffers = IndirectTerrainBuffers { buffers };

        commands.entity(entity).insert(buffers);
    }
}

struct QuadPlugin;

impl Plugin for QuadPlugin {
    fn build(&self, _app: &mut App) {}
}
