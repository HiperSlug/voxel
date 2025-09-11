//! see bevy [`MeshAllocator`](https://github.com/bevyengine/bevy/blob/da126fa9647d7c2506126cc499c64861294a2ef7/crates/bevy_render/src/mesh/allocator.rs#L56)

mod slab {
    use offset_allocator::{Allocation, Allocator};
    use std::mem::replace;

    pub struct QueueElement {
        pub offset: u64,
        pub data: Vec<u8>,
    }

    impl QueueElement {
        fn end(&self) -> u64 {
            self.offset + self.data.len() as u64
        }
        
        fn len(&self) -> usize {
            self.data.len()
        }
    }

    pub struct Slab {
        allocator: Allocator,
        write_queue: Vec<QueueElement>,
        free_queue: Vec<Allocation>,
        size: u32,
    }

    impl Slab {
        pub fn new(size: u32) -> Self {
            Self {
                allocator: Allocator::new(size),
                write_queue: Vec::new(),
                free_queue: Vec::new(),
                size,
            }
        }

        pub fn try_store(&mut self, data: &[u8]) -> Option<Allocation> {
            let allocation = self.allocator.allocate(data.len() as u32)?;

            let offset = allocation.offset as u64;
            let len = self.allocator.allocation_size(allocation) as usize;

            let queue_index = self
                .write_queue
                .binary_search_by_key(&offset, |e| e.offset)
                .err()
                .unwrap();

            let (below_slice, above_slice) = self.write_queue.split_at_mut(queue_index);
            let below_opt = below_slice.last_mut();
            let above_opt = above_slice.first_mut();

            // merges adjacent writes
            match (
                below_opt.filter(|b| offset == b.end()),
                above_opt.filter(|a| a.offset == offset + len as u64),
            ) {
                (None, None) => {
                    self.write_queue.insert(
                        queue_index,
                        QueueElement {
                            offset,
                            data: {
                                let mut vec = Vec::with_capacity(len);
                                vec.extend(data);
                                vec.resize(len, 0);
                                vec
                            },
                        },
                    );
                }
                (Some(below), None) => {
                    below.data.reserve(len);
                    below.data.extend(data);
                    below.data.resize(below.len() + len, 0);
                }
                (None, Some(above)) => {
                    let mut new_data = Vec::with_capacity(len + above.len());
                    new_data.extend(data);
                    new_data.resize(len, 0);
                    new_data.extend(&above.data);

                    above.data = new_data;
                    above.offset = offset;
                }
                (Some(below), Some(above)) => {
                    below.data.reserve_exact(len + above.len());
                    below.data.extend(data);
                    below.data.resize(below.len() + len, 0);
                    below.data.extend(&above.data);

                    self.write_queue.remove(queue_index);
                }
            }

            Some(allocation)
        }

        pub fn free(&mut self, allocation: Allocation) {
            self.free_queue.push(allocation);
        }

        pub fn is_empty(&self) -> bool {
            self.allocator.storage_report().total_free_space == self.size
        }

        pub fn flush_queues(&mut self) -> Option<Vec<QueueElement>> {
            for allocation in self.free_queue.drain(..) {
                self.allocator.free(allocation);
            }

            if !self.write_queue.is_empty() {
                let capacity = self.write_queue.capacity();
                let queue = replace(&mut self.write_queue, Vec::with_capacity(capacity));
                Some(queue)
            } else {
                None
            }
        }
    }
}

use arc_swap::ArcSwap;
use bevy::{
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        render_resource::{BufferUsages, COPY_BUFFER_ALIGNMENT},
        renderer::{RenderDevice, RenderQueue},
    },
};
use bytemuck::{Pod, cast_slice};
use parking_lot::Mutex;
use slab::{Slab, SlabAllocation};
use std::{
    marker::PhantomData,
    sync::{Arc, LazyLock},
};

pub static BUFFER_ALLOCATOR_SETTINGS: LazyLock<ArcSwap<BufferAllocatorSettings>> =
    LazyLock::new(|| ArcSwap::from_pointee(BufferAllocatorSettings::default()));

pub struct BufferAllocatorSettings {
    pub slab_size: u64,
    pub large_threshold: u64,
}

impl Default for BufferAllocatorSettings {
    fn default() -> Self {
        const MIB: u64 = 2u64.pow(20);
        Self {
            slab_size: 16 * MIB,
            large_threshold: 8 * MIB,
        }
    }
}

#[derive(Resource, Default)]
pub struct BufferAllocator<T> {
    slabs: Arc<Mutex<Vec<(bool, Option<Slab>)>>>,
    recycle: Vec<usize>,
    _phantom: PhantomData<T>,
}

impl<T: Pod> BufferAllocator<T> {
    pub fn store(&mut self, data: &[T]) -> BufferAllocation<T> {
        let data = cast_slice(data);

        let mut guard = self.slabs.lock();

        for (slab_index, dirty, slab) in guard
            .iter_mut()
            .enumerate()
            .filter_map(|(i, (d, s))| s.as_mut().map(|s| (i, d, s)))
        {
            if let Some(slab_allocation) = slab.try_store(data) {
                *dirty = true;
                return BufferAllocation {
                    slab_index,
                    slab_allocation,
                    _phantom: PhantomData,
                };
            }
        }

        let slab_size = u64::max(
            BUFFER_ALLOCATOR_SETTINGS.load().slab_size,
            (data.len() as u64).next_multiple_of(COPY_BUFFER_ALIGNMENT),
        );

        let mut slab = Slab::new(slab_size);
        let slab_allocation = slab.try_store(data).unwrap();

        let slab_index = if let Some(slab_index) = self.recycle.pop() {
            guard[slab_index] = (true, Some(slab));
            slab_index
        } else {
            guard.push((true, Some(slab)));
            guard.len() - 1
        };

        BufferAllocation {
            slab_index,
            slab_allocation,
            _phantom: PhantomData,
        }
    }

    pub fn free(&mut self, buffer_allocation: &BufferAllocation<T>) -> Result<(), ()> {
        let slab = self.slabs[buffer_allocation.slab_index]
            .1
            .as_mut()
            .ok_or(())?;

        slab.free(&buffer_allocation.slab_allocation);

        if slab.is_empty() {
            self.slabs[buffer_allocation.slab_index] = (true, None);
            self.recycle.push(buffer_allocation.slab_index);
        }

        Ok(())
    }

    pub fn send_sync_info(&mut self) {
        let msg = Synchronize(
            self.slabs
                .iter_mut()
                .enumerate()
                .filter_map(|(index, (dirty, slab_opt))| {
                    if *dirty {
                        *dirty = false;
                        Some((index, slab_opt.as_mut().and_then(|slab| slab.slab_info())))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
        );

        self.sender.send(msg).unwrap();
    }
}

#[derive(Resource)]
pub struct GpuBufferAllocator<T> {
    slabs: Vec<Option<GpuSlab>>,
    receiver: Receiver<Synchronize>,
    _phantom: PhantomData<T>,
}

impl<T> GpuBufferAllocator<T> {
    pub fn new(receiver: Receiver<Synchronize>) -> Self {
        Self {
            slabs: Vec::new(),
            _phantom: PhantomData,
            receiver,
        }
    }

    pub fn slabs(&self) -> &Vec<Option<GpuSlab>> {
        &self.slabs
    }

    pub fn receive_sync_info(
        &mut self,
        device: &RenderDevice,
        queue: &RenderQueue,
        settings: &GpuBufferAllocatorSettings<T>,
    ) {
        for msg in self.receiver.try_iter() {
            for (index, slab_info_opt) in msg.0.into_iter() {
                if index >= self.slabs.len() {
                    self.slabs.resize_with(index + 1, Default::default);
                }

                let gpu_slab_opt = &mut self.slabs[index];

                if let Some(slab_info) = slab_info_opt {
                    let gpu_slab = gpu_slab_opt.get_or_insert_with(|| {
                        let size = slab_info.size;
                        let label = format!("GpuSlab {{ index: {index}, size: {size} }}");

                        GpuSlab::new(device, &label, size, settings.buffer_usages)
                    });

                    for queue_element in &slab_info.queue {
                        gpu_slab.store(queue, queue_element);
                    }
                } else {
                    *gpu_slab_opt = None;
                }
            }
        }
    }
}

struct Synchronize(Vec<(usize, Option<SlabInfo>)>);

pub struct BufferAllocation<T> {
    slab_index: usize,
    slab_allocation: SlabAllocation,
    _phantom: PhantomData<T>,
}

impl<T> BufferAllocation<T> {
    pub fn slab_index(&self) -> usize {
        self.slab_index
    }

    pub fn byte_offset(&self) -> u32 {
        self.slab_allocation.allocation.offset
    }

    pub fn byte_size(&self) -> u32 {
        self.slab_allocation.size
    }

    pub fn offset(&self) -> u32 {
        debug_assert_eq!(self.byte_offset() % size_of::<T>() as u32, 0);
        self.byte_offset() / size_of::<T>() as u32
    }

    pub fn size(&self) -> u32 {
        debug_assert_eq!(self.byte_size() % size_of::<T>() as u32, 0);
        self.byte_size() / size_of::<T>() as u32
    }
}

#[derive(Event)]
pub struct FreeEvent<T> {
    buffer_allocation: BufferAllocation<T>,
}

impl<T> Into<FreeEvent<T>> for BufferAllocation<T> {
    fn into(self) -> FreeEvent<T> {
        FreeEvent {
            buffer_allocation: self,
        }
    }
}

pub fn free_allocations<T: Pod + Send + Sync + 'static>(
    mut buffer_allocator: ResMut<BufferAllocator<T>>,
    mut events: EventReader<FreeEvent<T>>,
) {
    for event in events.read() {
        // invariant: `BufferAllocation` is consumed when dropped (turned into an event)
        buffer_allocator.free(&event.buffer_allocation).unwrap();
    }
}

pub fn send_sync_info<T: Pod + Send + Sync + 'static>(
    mut buffer_allocator: ResMut<BufferAllocator<T>>,
) {
    buffer_allocator.send_sync_info();
}

pub fn receive_sync_info<T: Pod + Send + Sync + 'static>(
    mut gpu_buffer_allocator: ResMut<GpuBufferAllocator<T>>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    settings: Res<GpuBufferAllocatorSettings<T>>,
) {
    gpu_buffer_allocator.receive_sync_info(&device, &queue, &settings);
}

#[derive(Resource)]
pub struct GpuBufferAllocatorSettings<T> {
    pub buffer_usages: BufferUsages,
    _phantom: PhantomData<T>,
}

impl<T> Default for GpuBufferAllocatorSettings<T> {
    fn default() -> Self {
        Self {
            buffer_usages: BufferUsages::empty(),
            _phantom: PhantomData,
        }
    }
}

pub struct BufferAllocatorPlugin<T>(PhantomData<T>);

impl<T: Pod + Default + Send + Sync + 'static> Plugin for BufferAllocatorPlugin<T> {
    fn build(&self, app: &mut App) {
        let (sender, receiver) = bounded(8);

        app.insert_resource(BufferAllocator::<T>::new(sender))
            .init_resource::<BufferAllocatorSettings<T>>()
            .add_systems(Update, (free_allocations::<T>, send_sync_info::<T>));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .insert_resource(GpuBufferAllocator::<T>::new(receiver))
            .init_resource::<GpuBufferAllocatorSettings<T>>()
            .add_systems(Render, receive_sync_info::<T>.in_set(RenderSet::Prepare));
    }
}
