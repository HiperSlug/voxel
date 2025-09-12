use bevy::{
    platform::collections::HashMap,
    prelude::*,
    render::{
        RenderApp, define_atomic_id,
        render_resource::{
            Buffer, BufferAddress, BufferDescriptor, BufferUsages, COPY_BUFFER_ALIGNMENT,
        },
        renderer::{RenderDevice, RenderQueue},
    },
};
use bytemuck::{Pod, cast_slice};
use core::slice;
use offset_allocator as alloc;
use parking_lot::Mutex;
use std::{marker::PhantomData, sync::Arc};

define_atomic_id!(SlabId);

#[derive(Resource, Default, Clone, Deref)]
pub struct AllocBuffer(pub Arc<Mutex<InnerAllocBuffer>>);

#[derive(Default)]
pub struct InnerAllocBuffer {
    slabs: HashMap<SlabId, Slab>,
    layouts: HashMap<SlabLayout, Vec<SlabId>>,
}

impl InnerAllocBuffer {
    pub fn store<T: Pod>(
        &mut self,
        data: &[T],
        queue: &RenderQueue,
        device: &RenderDevice,
        settings: &AllocBufferSettings,
    ) -> Allocation<T> {
        let bytes = {
            let mut bytes = cast_slice::<_, u8>(data);
            if !size_of::<T>().is_multiple_of(COPY_BUFFER_ALIGNMENT as usize) {
                bytes = unsafe {
                    slice::from_raw_parts(
                        bytes.as_ptr(),
                        bytes.len().next_multiple_of(COPY_BUFFER_ALIGNMENT as usize),
                    )
                }
            }
            bytes
        };

        let slab_layout = SlabLayout::new::<T>();
        let slab_ids = self.layouts.entry(slab_layout).or_default();

        if settings.large_threshold >= bytes.len() {
            for slab_id in slab_ids.iter() {
                let slab = self.slabs.get_mut(slab_id).unwrap();

                if let Some(allocation) = slab.try_store(bytes, queue) {
                    return Allocation {
                        allocation,
                        slab_id: *slab_id,
                        _marker: PhantomData,
                    };
                };
            }
        }

        let slab_id = SlabId::new();

        #[cfg(debug_assertions)]
        let label = Some(&*format!("Slab {slab_id:?}"));
        #[cfg(not(debug_assertions))]
        let label = None;

        let size = settings.slab_size.max(bytes.len() as u32);

        let mut slab = Slab::new(device, label, size, settings.usage);

        let allocation = slab.try_store(bytes, queue).unwrap();

        self.slabs.insert(slab_id, slab);
        slab_ids.push(slab_id);

        Allocation {
            allocation,
            slab_id,
            _marker: PhantomData,
        }
    }

    pub fn free<T>(
        &mut self,
        Allocation {
            allocation,
            slab_id,
            ..
        }: Allocation<T>,
    ) {
        let slab = self.slabs.get_mut(&slab_id).unwrap();
        slab.free(allocation);

        if slab.is_empty() {
            self.slabs.remove(&slab_id);

            let slab_layout = SlabLayout::new::<T>();
            let layout = self.layouts.get_mut(&slab_layout).unwrap();
            layout.retain(|s| *s != slab_id);
        }
    }

    pub fn slabs(&self) -> &HashMap<SlabId, Slab> {
        &self.slabs
    }
}

pub struct Slab {
    allocator: alloc::Allocator,
    buffer: Buffer,
}

impl Slab {
    pub fn new(device: &RenderDevice, label: Option<&str>, size: u32, usage: BufferUsages) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label,
            size: size as u64,
            usage: usage | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let allocator = alloc::Allocator::new(size);

        Self { allocator, buffer }
    }

    pub fn try_store(&mut self, bytes: &[u8], queue: &RenderQueue) -> Option<alloc::Allocation> {
        let allocation = self.allocator.allocate(bytes.len() as u32)?;

        queue.write_buffer(&self.buffer, allocation.offset as BufferAddress, bytes);

        Some(allocation)
    }

    pub fn free(&mut self, allocation: alloc::Allocation) {
        self.allocator.free(allocation);
    }

    pub fn reset(&mut self) {
        self.allocator.reset();
    }

    pub fn is_empty(&self) -> bool {
        self.allocator.storage_report().total_free_space as BufferAddress == self.buffer.size()
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}

pub struct Allocation<T> {
    allocation: alloc::Allocation,
    slab_id: SlabId,
    _marker: PhantomData<T>,
}

impl<T> Allocation<T> {
    pub fn byte_offset(&self) -> u32 {
        self.allocation.offset
    }

    pub fn offset(&self) -> u32 {
        self.byte_offset() / size_of::<T>() as u32
    }

    pub fn slab_id(&self) -> SlabId {
        self.slab_id
    }
}

#[derive(Resource, Default, Clone, Deref)]
pub struct AllocBufferSettings(pub Arc<InnerAllocBufferSettings>);

pub struct InnerAllocBufferSettings {
    pub slab_size: u32,
    pub large_threshold: usize,
    pub usage: BufferUsages,
}

impl Default for InnerAllocBufferSettings {
    fn default() -> Self {
        const MIB: u32 = 2u32.pow(20);
        Self {
            slab_size: 16 * MIB,
            large_threshold: 8 * MIB as usize,
            usage: BufferUsages::empty(),
        }
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
struct SlabLayout {
    size_of: u16,
}

impl SlabLayout {
    const fn new<T>() -> Self {
        Self {
            size_of: size_of::<T>() as u16,
        }
    }
}

pub struct AllocBufferPlugin;

impl Plugin for AllocBufferPlugin {
    fn build(&self, app: &mut App) {
        let alloc_buffer = AllocBuffer::default();
        let alloc_buffer_settings = AllocBufferSettings::default();

        app.insert_resource(alloc_buffer.clone())
            .insert_resource(alloc_buffer_settings.clone())
            .sub_app_mut(RenderApp)
            .insert_resource(alloc_buffer)
            .insert_resource(alloc_buffer_settings);
    }
}
