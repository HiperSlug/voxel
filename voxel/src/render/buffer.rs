use bevy::{
    prelude::*,
    render::{
        render_resource::{Buffer, BufferDescriptor, BufferUsages},
        renderer::{RenderDevice, RenderQueue},
    },
};
use bytemuck::{Pod, checked::cast_slice};
use freelist::{Allocation, FreeList};
use std::{marker::PhantomData, num::NonZeroUsize};

pub struct AllocatedStorageBuffer<T> {
    buffer: Buffer,
    freelist: FreeList,
    _phantom: PhantomData<T>,
}

impl<T> AllocatedStorageBuffer<T>
where
    T: Pod,
{
    pub fn new(device: &RenderDevice, len: NonZeroUsize) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: (size_of::<T>() * len.get()) as u64,
            mapped_at_creation: false,
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
        });

        let freelist = FreeList::new(len);

        Self {
            buffer,
            freelist,
            _phantom: PhantomData,
        }
    }

    pub fn write(&self, queue: &RenderQueue, data: &[T]) -> Option<BufferAllocation> {
        let len = data.len().try_into().expect("cannot write zero data");
        let allocation = self.freelist.allocate(len)?;

        let offset = (allocation.slice().start * size_of::<T>()) as u64;
        let size = ((allocation.slice().len() * size_of::<T>()) as u64)
            .try_into()
            .unwrap();

        let mut view = queue
            .write_buffer_with(&self.buffer, offset, size)
            .expect("already resolved size");
        view.copy_from_slice(cast_slice(data));

        Some(BufferAllocation(allocation))
    }
}

#[derive(Deref)]
pub struct BufferAllocation(Allocation);

pub struct ChunkMesh([Option<BufferAllocation>; 6]);
