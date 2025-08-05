#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    mesh_view_bindings::{view, globals},
    pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND, PbrInput, pbr_input_new},
    pbr_functions as fns,
    mesh_functions::{get_world_from_local, mesh_position_local_to_clip, mesh_position_local_to_world},
}
#import bevy_core_pipeline::tonemapping::tone_mapping

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

@group(2) @binding(100) var textures: texture_2d_array<f32>;
@group(2) @binding(101) var texture_sampler: sampler;

struct VertexInput {
	@builtin(instance_index) instance_index: u32,
    @location(0) packed_data: vec2<u32>,
};

struct CustomVertexOutput {
    @builtin(position) position: vec4<f32>,
	@location(0) world_position: vec4<f32>,
	@location(1) world_normal: vec3<f32>,
	@location(2) uv: vec2<f32>,
	@location(3) color: vec4<f32>,
	@location(4) texture_layer: u32,
};

const MASK3: u32 = (1 << 3) - 1;
const MASK6: u32 = (1 << 6) - 1;
const MASK9: u32 = (1 << 9) - 1;

fn normal_from_id(id: u32) -> vec3<f32> {
	switch id {
		case 0u {
			return vec3(0.0, 1.0, 0.0);
		}
		case 1u {
			return vec3(0.0, -1.0, 0.0);
		}
		case 2u {
			return vec3(1.0, 0.0, 0.0);
		}
		case 3u {
			return vec3(-1.0, 0.0, 0.0);
		}
		case 4u {
			return vec3(0.0, 0.0, 1.0);
		}
		case 5u {
			return vec3(0.0, 0.0, -1.0);
		}
		default {
			return vec3(0.0);
		}
	}
}

fn color_from_id(id: u32) -> vec4<f32> {
	let r = f32(id & MASK3) / 7.0;
	let g = f32((id >> 3) & MASK3) / 7.0;
	let b = f32((id >> 6) & MASK3) / 7.0;
	return vec4(r, g, b, 1.0);
}

@vertex
fn vertex(in: VertexInput) -> CustomVertexOutput {
	let vertex_info = in.packed_data.x;
	let x = f32(vertex_info & MASK6);
	let y = f32((vertex_info >> 6) & MASK6);
	let z = f32((vertex_info >> 12) & MASK6);
	
	let position = vec4(x, y, z, 1.0);

	let u = f32((vertex_info >> 18) & MASK6) / 63.0;
	let v = f32((vertex_info >> 24) & MASK6) / 63.0;

	let uv = vec2(u, v);


	let quad_info = in.packed_data.y;

	let normal_id = quad_info & MASK3;
	let normal = normal_from_id(normal_id);

	let color_id = (quad_info >> 3) & MASK9;
	let color = color_from_id(color_id);

	let texture_layer = quad_info >> 12;

	var out: CustomVertexOutput;
	out.position = mesh_position_local_to_clip(
		get_world_from_local(in.instance_index),
		position,
	);
	out.world_position = mesh_position_local_to_world(
		get_world_from_local(in.instance_index),
		position,
	);
	out.world_normal = normal;
	out.uv = uv;
	out.color = color;
	out.texture_layer = texture_layer;

    return out;
}

@fragment
fn fragment(
	in: CustomVertexOutput,
	@builtin(front_facing) is_front: bool,
) -> FragmentOutput {
	var std_output: VertexOutput;

	std_output.position = in.position;
	std_output.world_position = in.world_position;
	std_output.world_normal = in.world_normal;
#ifdef VERTEX_UVS
	std_output.uv = in.uv;
#endif
#ifdef VERTEX_UVS_B
	std_output.uv_b = in.uv;
#endif
#ifdef VERTEX_COLORS
	std_output.color = in.color;
#endif

	var pbr_input = pbr_input_from_standard_material(std_output, is_front);
	pbr_input.material.base_color = in.color * textureSampleBias(textures, texture_sampler, in.uv, in.texture_layer, view.mip_bias);
	pbr_input.material.base_color = fns::alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
	let out = deferred_output(in, pbr_input);
#else
	var out: FragmentOutput;
	out.color = apply_pbr_lighting(pbr_input);
	out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

	return out;
}