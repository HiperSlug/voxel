use std::{marker::PhantomData, num::NonZeroUsize};

use bevy::{
    ecs::query::QueryItem,
    pbr::MaterialExtension,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::{AsBindGroup, Buffer, BufferDescriptor, BufferUsages, ShaderRef},
        renderer::{RenderDevice, RenderQueue},
        storage::ShaderStorageBuffer,
    },
};
use bytemuck::{Pod, Zeroable, checked::cast_slice};

use crate::render::freelist::Allocation;

use super::freelist::{FreeList, Slice};

pub struct InstancePlugin;

impl Plugin for InstancePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<InstanceDataMaterial>::default());
        // TODO
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Deref, DerefMut, Pod, Zeroable)]
pub struct InstanceData(pub u64);

#[derive(Debug, Component, Deref)]
pub struct InstanceDataMaterial(pub Vec<InstanceData>);

impl ExtractComponent for InstanceDataMaterial {
    type QueryData = &'static Self;
    type Out = Self;
    type QueryFilter = ();

    fn extract_component(item: QueryItem<'_, Self::QueryData>) -> Option<Self::Out> {
        Some(Self(item.0.clone()))
    }
}

#[derive(Debug, Asset, TypePath, AsBindGroup, Clone)]
pub struct VoxelMaterial {
    #[storage(20, read_only)]
    chunk_data: Handle<ShaderStorageBuffer>,

    // TODO: look into bindless
    #[texture(21, dimension = "2d_array")]
    #[sampler(22)]
    textures: Handle<Image>,
}

const SHADER: &str = "shaders/chunk.wgsl";

impl MaterialExtension for VoxelMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }

    fn vertex_shader() -> ShaderRef {
        SHADER.into()
    }
}

// TODO: merge with or fragment block_lib
#[derive(Debug, Resource, Deref, DerefMut)]
pub struct SharedVoxelMaterial(pub Handle<VoxelMaterial>);

pub struct InstanceStorageBuffer<T> {
    buffer: Buffer,
    freelist: FreeList,
    size_of_t: usize,
    _phantom: PhantomData<T>,
}

impl<T> InstanceStorageBuffer<T>
where
    T: Pod,
{
    fn new(device: &RenderDevice, len: NonZeroUsize) -> Self {
        let size_of_t = size_of::<T>();
        let buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: (size_of_t * len.get()) as u64,
            mapped_at_creation: false,
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
        });

        let freelist = FreeList::new(len);

        Self {
            buffer,
            freelist,
            size_of_t,
            _phantom: PhantomData,
        }
    }

    fn write(&mut self, queue: &RenderQueue, data: &[T]) -> Option<BufferAllocation> {
        let len = data.len().try_into().unwrap();
        let allocation = self.freelist.allocate(len)?;
        let offset = (allocation.slice().start * self.size_of_t) as u64;

        let data = cast_slice(data);
        queue.write_buffer(&self.buffer, offset, data);

        Some(BufferAllocation(allocation))
    }
}

#[derive(Deref)]
pub struct BufferAllocation(pub Allocation);

pub struct ChunkMesh([Option<BufferAllocation>; 6]);
