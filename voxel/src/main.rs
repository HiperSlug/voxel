use bevy::{app::{App, Startup}, DefaultPlugins};
use bevy_flycam::NoCameraPlayerPlugin;

use crate::{alloc_buffer::AllocBufferPlugin, chunk::VoxelQuad, instancing::InstancingPlugin};

mod alloc_buffer;
mod chunk;
mod instancing;
mod signed_axis;
mod viewer;


fn main() {
	let app = App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins(NoCameraPlayerPlugin)
		.add_plugins(AllocBufferPlugin::<VoxelQuad>::default())
		.add_plugins(InstancingPlugin)
		.add_systems(Startup, setup)
		.run();
}

fn setup() {

}