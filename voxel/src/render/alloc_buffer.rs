use bevy::{
    prelude::*,
    render::{
        RenderApp,
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
use slotmap::{SlotMap, new_key_type};
use std::{marker::PhantomData, mem::MaybeUninit, sync::Arc};

new_key_type! {
    struct SlabKey;
}

#[derive(Resource, Deref)]
pub struct AllocBuffer<T>(pub Arc<Mutex<InnerAllocBuffer<T>>>);

impl<T> Clone for AllocBuffer<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> AllocBuffer<T> {
    pub fn new(settings: AllocBufferSettings) -> Self {
        Self(Arc::new(Mutex::new(InnerAllocBuffer::new(settings))))
    }
}

pub struct InnerAllocBuffer<T> {
    slabs: SlotMap<SlabKey, Slab>,
    pub settings: AllocBufferSettings,

    _marker: PhantomData<T>,
}

impl<T> InnerAllocBuffer<T> {
    pub fn new(settings: AllocBufferSettings) -> Self {
        Self {
            settings,
            slabs: default(),
            _marker: default(),
        }
    }
}

impl<T: Pod> InnerAllocBuffer<T> {
    pub fn store(
        &mut self,
        data: &[T],
        queue: &RenderQueue,
        device: &RenderDevice,
    ) -> Allocation<T> {
        let bytes = {
            let mut bytes = cast_slice(data);
            if !size_of::<T>().is_multiple_of(COPY_BUFFER_ALIGNMENT as usize) {
                bytes = unsafe {
                    slice::from_raw_parts(
                        bytes.as_ptr(),
                        bytes.len().next_multiple_of(COPY_BUFFER_ALIGNMENT as usize),
                    )
                };
            }
            bytes
        };

        if self.settings.large_threshold >= bytes.len() {
            for (slab_key, slab) in &mut self.slabs {
                if let Some(allocation) = slab.try_store(bytes, queue) {
                    return Allocation {
                        allocation,
                        slab_key,
                        _marker: default(),
                    };
                };
            }
        }

        let mut allocation = MaybeUninit::uninit();

        let slab_key = self.slabs.insert_with_key(|k| {
            #[cfg(debug_assertions)]
            let label = Some(&*format!("Slab {k:?}"));
            #[cfg(not(debug_assertions))]
            let label = None;

            let size = self.settings.slab_size.max(bytes.len() as u32);

            let mut slab = Slab::new(device, label, size, self.settings.usage);

            allocation = MaybeUninit::new(slab.try_store(bytes, queue).unwrap());

            slab
        });

        let allocation = unsafe { allocation.assume_init() };

        Allocation {
            allocation,
            slab_key,
            _marker: default(),
        }
    }

    pub fn free(
        &mut self,
        Allocation {
            allocation,
            slab_key,
            ..
        }: Allocation<T>,
    ) {
        let slab = &mut self.slabs[slab_key];
        slab.free(allocation);

        if slab.is_empty() {
            self.slabs.remove(slab_key);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (SlabKey, &Slab)> {
        self.slabs.iter()
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

    pub fn is_empty(&self) -> bool {
        self.allocator.storage_report().total_free_space as BufferAddress == self.buffer.size()
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}

pub struct Allocation<T> {
    allocation: alloc::Allocation,
    slab_key: SlabKey,
    _marker: PhantomData<T>,
}

impl<T> Allocation<T> {
    pub fn byte_offset(&self) -> u32 {
        self.allocation.offset
    }

    pub fn offset(&self) -> u32 {
        self.byte_offset() / size_of::<T>() as u32
    }

    pub fn slab_key(&self) -> SlabKey {
        self.slab_key
    }
}

#[derive(Clone, Copy)]
pub struct AllocBufferSettings {
    pub slab_size: u32,
    pub large_threshold: usize,
    pub usage: BufferUsages,
}

impl Default for AllocBufferSettings {
    fn default() -> Self {
        const MIB: u32 = 2u32.pow(20);
        Self {
            slab_size: 16 * MIB,
            large_threshold: 8 * MIB as usize,
            usage: BufferUsages::empty(),
        }
    }
}

#[derive(Default)]
pub struct AllocBufferPlugin<T> {
    pub settings: AllocBufferSettings,
    _marker: PhantomData<T>,
}

impl<T> AllocBufferPlugin<T> {
    pub fn new(settings: AllocBufferSettings) -> Self {
        Self {
            settings,
            _marker: default(),
        }
    }
}

impl<T: Send + Sync + 'static> Plugin for AllocBufferPlugin<T> {
    fn build(&self, app: &mut App) {
        let alloc_buffer = AllocBuffer::<T>::new(self.settings);

        app.insert_resource(alloc_buffer.clone());

        app.sub_app_mut(RenderApp).insert_resource(alloc_buffer);
    }
}
