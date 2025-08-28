//! see bevy [`MeshAllocator`](https://github.com/bevyengine/bevy/blob/da126fa9647d7c2506126cc499c64861294a2ef7/crates/bevy_render/src/mesh/allocator.rs#L56)

mod slab {
    use std::mem::replace;

    use bevy::render::{
        render_resource::{Buffer, BufferDescriptor, BufferUsages},
        renderer::{RenderDevice, RenderQueue},
    };
    use offset_allocator::{Allocation, Allocator};

    pub type WriteQueue = Vec<QueueElement>;

    pub struct QueueElement {
        pub offset: u64,
        pub data: Vec<u8>,
        alloc_len: usize,
    }

    impl QueueElement {
        fn end(&self) -> u64 {
            self.offset + self.alloc_len as u64
        }
    }

    pub struct Slab {
        pub allocator: Allocator,
        pub size: u64,
        pub queue: WriteQueue,
    }

    impl Slab {
        pub fn new(size: u64) -> Self {
            let allocator = Allocator::new(size as u32);

            Self {
                allocator,
                size,
                queue: Vec::new(),
            }
        }

        pub fn try_store(&mut self, data: &[u8]) -> Option<SlabAllocation> {
            let size = data.len() as u32;
            let allocation = self.allocator.allocate(size)?;

            let offset = allocation.offset as u64;
            // alloc_len != data.len()
            let alloc_len = self.allocator.allocation_size(allocation) as usize;

            let queue_index = self
                .queue
                .binary_search_by_key(&offset, |q| q.offset)
                .err()
                .unwrap();

            let (below, above) = self.queue.split_at_mut(queue_index);
            let below = below.last_mut();
            let above = above.first_mut();

            // merges adjacent writes
            match (
                below.and_then(|b| (offset == b.end()).then_some(b)),
                above.and_then(|b| (b.offset == (offset + alloc_len as u64)).then_some(b)),
            ) {
                (None, None) => {
                    let queue_element = QueueElement {
                        offset,
                        alloc_len,
                        data: data.to_vec(),
                    };
                    self.queue.insert(queue_index, queue_element);
                }
                (Some(below), None) => {
                    let len_diff = below.alloc_len - below.data.len();
                    below.data.reserve_exact(len_diff + data.len());
                    below.data.resize(below.alloc_len, 0);
                    below.data.extend(data);

                    below.alloc_len += alloc_len;
                }
                (None, Some(above)) => {
                    let mut new_data = Vec::with_capacity(alloc_len + above.data.len());
                    new_data.extend(data);
                    new_data.resize(alloc_len, 0);
                    new_data.extend(&above.data);
                    above.data = new_data;

                    above.offset = offset;
                    above.alloc_len += alloc_len;
                }
                (Some(below), Some(above)) => {
                    let len_diff = below.alloc_len - below.data.len();
                    below
                        .data
                        .reserve_exact(len_diff + alloc_len + above.data.len());
                    below.data.resize(below.alloc_len, 0);
                    below.data.extend(data);
                    below.data.resize(below.alloc_len + alloc_len, 0);
                    below.data.extend(&above.data);

                    below.alloc_len += alloc_len + above.alloc_len;

                    self.queue.remove(queue_index);
                }
            }

            Some(SlabAllocation { allocation, size })
        }

        pub fn free(&mut self, slab_allocation: &SlabAllocation) {
            self.allocator.free(slab_allocation.allocation);
        }

        pub fn is_empty(&self) -> bool {
            self.allocator.storage_report().total_free_space as u64 == self.size
        }

        pub fn slab_info(&mut self) -> Option<SlabInfo> {
            (!self.queue.is_empty()).then(|| {
                let capacity = self.queue.capacity();
                SlabInfo {
                    size: self.size,
                    queue: replace(&mut self.queue, Vec::with_capacity(capacity)),
                }
            })
        }
    }

    pub struct SlabAllocation {
        pub allocation: Allocation,
        pub size: u32,
    }

    pub struct SlabInfo {
        pub size: u64,
        pub queue: WriteQueue,
    }

    pub struct GpuSlab {
        pub buffer: Buffer,
    }

    impl GpuSlab {
        pub fn new(
            device: &RenderDevice,
            label: &str,
            size: u64,
            buffer_usages: BufferUsages,
        ) -> Self {
            let buffer = device.create_buffer(&BufferDescriptor {
                label: Some(label),
                size,
                usage: buffer_usages | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            Self { buffer }
        }

        pub fn store(&self, queue: &RenderQueue, queue_element: &QueueElement) {
            queue.write_buffer(&self.buffer, queue_element.offset, &queue_element.data)
        }
    }
}

use bevy::{
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        render_resource::{BufferUsages, COPY_BUFFER_ALIGNMENT},
        renderer::{RenderDevice, RenderQueue},
    },
};
use bytemuck::{Pod, cast_slice};
use crossbeam::channel::{Receiver, Sender, bounded};
use slab::{GpuSlab, Slab, SlabAllocation, SlabInfo};
use std::marker::PhantomData;

#[derive(Resource)]
pub struct BufferAllocator<T> {
    slabs: Vec<(bool, Option<Slab>)>,
    recycle: Vec<usize>,
    sender: Sender<Synchronize>,
    _phantom: PhantomData<T>,
}

impl<T: Pod> BufferAllocator<T> {
    pub fn new(sender: Sender<Synchronize>) -> Self {
        Self {
            sender,
            slabs: Vec::new(),
            recycle: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn store(
        &mut self,
        data: &[T],
        settings: &BufferAllocatorSettings<T>,
    ) -> BufferAllocation<T> {
        let data = cast_slice(data);

        for (slab_index, dirty, slab) in self
            .slabs
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
            settings.slab_size,
            (data.len() as u64).next_multiple_of(COPY_BUFFER_ALIGNMENT),
        );

        let mut slab = Slab::new(slab_size);
        let slab_allocation = slab.try_store(data).unwrap();

        let slab_index = if let Some(slab_index) = self.recycle.pop() {
            self.slabs[slab_index] = (true, Some(slab));
            slab_index
        } else {
            self.slabs.push((true, Some(slab)));
            self.slabs.len() - 1
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
pub struct BufferAllocatorSettings<T> {
    pub slab_size: u64,
    _phantom: PhantomData<T>,
}

impl<T> Default for BufferAllocatorSettings<T> {
    fn default() -> Self {
        const MIB: u64 = 2u64.pow(20);
        Self {
            slab_size: 8 * MIB,
            _phantom: PhantomData,
        }
    }
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
