use wgpu::{Backends, util::DeviceExt};

// Minimal launcher: run async main synchronously for brevity
fn main() {
    pollster::block_on(run());
}

async fn run() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: Backends::VULKAN,
        ..Default::default()
    });
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    const REQUIRED_FEATURES: wgpu::Features =
        wgpu::Features::MULTI_DRAW_INDIRECT.union(wgpu::Features::SPIRV_SHADER_PASSTHROUGH);

    let adapter_features = adapter.features();

    assert!(adapter_features.contains(REQUIRED_FEATURES));

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            required_features: REQUIRED_FEATURES,
            ..Default::default()
        })
        .await
        .unwrap();

    let vs_desc = wgpu::include_spirv_raw!("drawid.vert.spv");
    let fs_desc = wgpu::include_spirv_raw!("drawid.frag.spv");

    let vs_module = unsafe { device.create_shader_module_passthrough(vs_desc) };
    let fs_module = unsafe { device.create_shader_module_passthrough(fs_desc) };

    let vertex_buffer_layout = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: 0,
            shader_location: 0,
        }],
    };

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("pipeline_layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vs_module,
            entry_point: Some("main"),
            buffers: &[vertex_buffer_layout],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &fs_module,
            entry_point: Some("main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    const VERTICES: &[f32] = &[-0.1, -0.3, 0.1, -0.3, 0.0, 0.2];

    let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("vertex"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    const DRAW_CMDS: &[[u32; 4]] = &[[3, 1, 0, 0], [3, 1, 0, 0]];
    let indirect_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("indirect"),
        contents: bytemuck::cast_slice(DRAW_CMDS),
        usage: wgpu::BufferUsages::INDIRECT,
    });

    let size = wgpu::Extent3d {
        width: 512,
        height: 256,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("target"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    });

    let view = texture.create_view(&Default::default());

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("ce") });
    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("rp"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        rpass.set_pipeline(&render_pipeline);
        rpass.set_vertex_buffer(0, vertex_buf.slice(..));
        rpass.multi_draw_indirect(&indirect_buf, 0, 2);
    }

    let bytes_per_pixel = 4u32;
    let padded_bytes_per_row =
        wgpu::util::align_to(4 * size.width, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) as u32;
    let readback_buffer_size = (padded_bytes_per_row as u64) * (size.height as u64);

    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: readback_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: None,
            },
        },
        size,
    );

    queue.submit(Some(encoder.finish()));
    device.poll(wgpu::PollType::Wait).unwrap();

    // 10) Map readback buffer and inspect pixels (simple check)
    let buffer_slice = readback.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, |_| ());
    device.poll(wgpu::PollType::Wait).unwrap();
    let data = buffer_slice.get_mapped_range();

    // Extract byte at some coordinates to verify color:
    // sample left triangle pixel at (64, 128) and right triangle pixel at (448, 128)
    let sample = |x: u32, y: u32| -> [u8; 4] {
        let row_start = (y as u64) * (padded_bytes_per_row as u64);
        let offset = row_start + (x as u64) * (bytes_per_pixel as u64);
        let b0 = data[offset as usize];
        let b1 = data[offset as usize + 1];
        let b2 = data[offset as usize + 2];
        let b3 = data[offset as usize + 3];
        [b0, b1, b2, b3]
    };

    let left = sample(64, 128);
    let right = sample(448, 128);

    println!("left pixel = {:?}, right pixel = {:?}", left, right);
    // Expect left to be red-ish, right to be green-ish if gl_DrawID reached the shader correctly.

    // cleanup mapping
    drop(data);
    readback.unmap();
}
