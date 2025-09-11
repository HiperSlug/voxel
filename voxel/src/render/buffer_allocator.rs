//! see bevy [`MeshAllocator`](https://github.com/bevyengine/bevy/blob/da126fa9647d7c2506126cc499c64861294a2ef7/crates/bevy_render/src/mesh/allocator.rs#L56)

mod slab {
    use bevy::render::render_resource::BufferAddress;
    use offset_allocator::{Allocation, Allocator};
    use std::mem::take;

    pub struct WriteBufferArgs {
        pub offset: BufferAddress,
        pub data: Vec<u8>,
    }

    impl WriteBufferArgs {
        fn new(offset: BufferAddress, d: &[u8], len: usize) -> Self {
            let mut data = Vec::with_capacity(len);
            data.resize(len, 0);

            let range = ..d.len();
            data[range].copy_from_slice(d);

            Self { offset, data }
        }

        fn join_below(&mut self, d: &[u8], len: usize) {
            let old_len = self.data.len();

            self.data.resize(old_len + len, 0);

            let range = old_len..old_len + d.len();
            self.data[range].copy_from_slice(d);
        }

        fn join_above(&mut self, offset: u64, d: &[u8], len: usize) {
            let old_len = self.data.len();

            self.data.resize(len + old_len, 0);

            let range = ..old_len;
            self.data.copy_within(range, len);

            let range = ..d.len();
            self.data[range].copy_from_slice(d);

            self.offset = offset;
        }

        fn join_between(&mut self, d: &[u8], len: usize, above: &Self) {
            let old_len = self.data.len();

            self.data.resize(old_len + len + above.data.len(), 0);

            let range = old_len..old_len + d.len();
            self.data[range].copy_from_slice(d);

            let range = old_len + len..old_len + len + above.data.len();
            self.data[range].copy_from_slice(&above.data);
        }

        fn end(&self) -> BufferAddress {
            self.offset + self.data.len() as BufferAddress
        }
    }

    pub struct Slab {
        allocator: Allocator,
        write_queue: Vec<WriteBufferArgs>,
        free_queue: Vec<Allocation>,
        flush_free: bool,
        size: u32,
    }

    impl Slab {
        pub fn new(size: u32) -> Self {
            Self {
                allocator: Allocator::new(size),
                write_queue: Vec::new(),
                free_queue: Vec::new(),
                flush_free: false,
                size,
            }
        }

        pub fn try_store(&mut self, data: &[u8]) -> Option<Allocation> {
            if self.flush_free {
                self.flush_free = false;
                for allocation in self.free_queue.drain(..) {
                    self.allocator.free(allocation);
                }
            }

            let allocation = self.allocator.allocate(data.len() as u32)?;

            let offset = allocation.offset as BufferAddress;
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
                    let element = WriteBufferArgs::new(offset, data, len);
                    self.write_queue.insert(queue_index, element);
                }
                (Some(below), None) => {
                    below.join_below(data, len);
                }
                (None, Some(above)) => {
                    above.join_above(offset, data, len);
                }
                (Some(below), Some(above)) => {
                    below.join_between(data, len, above);

                    self.write_queue.remove(queue_index);
                }
            }

            Some(allocation)
        }

        pub fn is_empty(&self) -> bool {
            self.allocator.storage_report().total_free_space == self.size
        }

        pub fn free(&mut self, allocation: Allocation) {
            self.free_queue.push(allocation);
        }

        pub fn flush(&mut self) -> Vec<WriteBufferArgs> {
            // flag tells next allocation that its safe to flush the frees.
            // this is doubly deferred because this runs on the extract 
            // schedule which is supposed to be very fast.
            self.flush_free = true;

            take(&mut self.write_queue)
        }
    }
}

use arc_swap::ArcSwap;
use bevy::{
    prelude::*,
    render::{
        render_resource::{Buffer, BufferDescriptor, BufferUsages}, renderer::{RenderDevice, RenderQueue}, Extract, Render, RenderApp, RenderSet
    },
};
use bytemuck::{Pod, cast_slice};
use offset_allocator::Allocation;
use parking_lot::Mutex;
use slab::{Slab, WriteBufferArgs};
use std::{
    marker::PhantomData,
    sync::{Arc, LazyLock},
};

pub static BUFFER_ALLOCATOR_SETTINGS: LazyLock<ArcSwap<BufferAllocatorSettings>> =
    LazyLock::new(|| ArcSwap::from_pointee(BufferAllocatorSettings::default()));

pub struct BufferAllocatorSettings {
    pub slab_size: u32,
    pub large_threshold: usize,
    pub buffer_usages: BufferUsages,
}

impl Default for BufferAllocatorSettings {
    fn default() -> Self {
        const MIB: usize = 2usize.pow(20);
        Self {
            slab_size: 16 * MIB as u32,
            large_threshold: 8 * MIB,
            buffer_usages: BufferUsages::empty(),
        }
    }
}

#[derive(Resource, Default)]
pub struct AsyncBufferAllocator<T>(pub Arc<Mutex<BufferAllocator<T>>>);

#[derive(Default)]
pub struct BufferAllocator<T> {
    slabs: Vec<Option<Slab>>,
    recycle: Vec<usize>,
    _phantom: PhantomData<T>,
}

impl<T: Pod> BufferAllocator<T> {
    pub fn store(&mut self, data: &[T]) -> BufferAllocation<T> {
        let data = cast_slice(data);

        let settings = BUFFER_ALLOCATOR_SETTINGS.load();

        if data.len() < settings.large_threshold {
            for (slab_index, slab) in self
                .slabs
                .iter_mut()
                .enumerate()
                .filter_map(|(i, s)| s.as_mut().map(|s| (i, s)))
            {
                if let Some(allocation) = slab.try_store(data) {
                    return BufferAllocation {
                        allocation,
                        slab_index,
                        _phantom: PhantomData,
                    };
                }
            }
        }

        let mut slab = Slab::new(settings.slab_size);
        let allocation = slab.try_store(data).unwrap();

        let slab_index = if let Some(slab_index) = self.recycle.pop() {
            self.slabs[slab_index] = Some(slab);
            slab_index
        } else {
            self.slabs.push(Some(slab));
            self.slabs.len() - 1
        };

        BufferAllocation {
            slab_index,
            allocation,
            _phantom: PhantomData,
        }
    }

    pub fn free(&mut self, buffer_allocations: impl Iterator<Item = BufferAllocation<T>>) {
        for BufferAllocation {
            slab_index,
            allocation,
            ..
        } in buffer_allocations
        {
            // invariant: all `BufferAllocation` are only freed once and not created from arbitrary data
            self.slabs[slab_index].as_mut().unwrap().free(allocation)
        }
    }

    pub fn flush(&mut self) -> Vec<Option<Vec<WriteBufferArgs>>> {
        let queues = self
            .slabs
            .iter_mut()
            .map(|slab_opt| slab_opt.as_mut().map(|slab| slab.flush()))
            .collect::<Vec<_>>();

        for slab_opt in self.slabs.iter_mut() {
            let Some(slab) = slab_opt else {
                continue;
            };
            if slab.is_empty() {
                *slab_opt = None
            }
        }

        queues
    }
}

#[derive(Resource, Default)]
pub struct GpuBufferAllocator<T> {
    buffers: Vec<Option<Buffer>>,
    _phantom: PhantomData<T>,
}

impl<T> GpuBufferAllocator<T> {
    pub fn write_buffers(
        &mut self,
        queues: &Vec<Option<Vec<WriteBufferArgs>>>,
        device: &RenderDevice,
        queue: &RenderQueue,
    ) {
        let settings = BUFFER_ALLOCATOR_SETTINGS.load();
        
        if queues.len() > self.buffers.len() {
            self.buffers.resize(queues.len(), None);
        }

        for (slab_opt, buffer_opt) in queues.into_iter().zip(&mut self.buffers) {
            if let Some(writes) = slab_opt {
                let buffer = buffer_opt.get_or_insert_with(|| {
                    device.create_buffer(&BufferDescriptor {
                        label: None,
                        size: settings.slab_size as u64,
                        usage: settings.buffer_usages | BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    })
                });
                for write in writes {
                    queue.write_buffer(buffer, write.offset, &write.data);
                }
            } else {
                *buffer_opt = None
            }
        }
    }
}

pub struct BufferAllocation<T> {
    slab_index: usize,
    allocation: Allocation,
    _phantom: PhantomData<T>,
}

impl<T> BufferAllocation<T> {
    pub fn slab_index(&self) -> usize {
        self.slab_index
    }

    pub fn byte_offset(&self) -> u32 {
        self.allocation.offset
    }

    pub fn offset(&self) -> u32 {
        debug_assert_eq!(self.byte_offset() % size_of::<T>() as u32, 0);
        self.byte_offset() / size_of::<T>() as u32
    }
}

#[derive(Resource, Default)]
struct ExtractedQueues<T>(Vec<Option<Vec<WriteBufferArgs>>>, PhantomData<T>);

pub fn flush<T: Pod + Send + Sync + 'static>(
    async_buffer_allocator: Extract<Res<AsyncBufferAllocator<T>>>,
    mut extracted_queues: ResMut<ExtractedQueues<T>>,
) {
    let queues = async_buffer_allocator.0.lock().flush();
    extracted_queues.0 = queues;
}

pub fn write_buffers<T: Pod + Send + Sync + 'static>(
    extracted_queues: Res<ExtractedQueues<T>>,
    mut gpu_buffer_allocator: ResMut<GpuBufferAllocator<T>>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>, 
) {
    let queues = &extracted_queues.0;
    gpu_buffer_allocator.write_buffers(queues, &device, &queue);
}

pub struct BufferAllocatorPlugin<T>(PhantomData<T>);

impl<T: Pod + Default + Send + Sync + 'static> Plugin for BufferAllocatorPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_resource::<AsyncBufferAllocator::<T>>();

        app.sub_app_mut(RenderApp)
            .init_resource::<GpuBufferAllocator::<T>>()
            .init_resource::<ExtractedQueues<T>>()
            .add_systems(ExtractSchedule, flush::<T>)
            .add_systems(Render, write_buffers::<T>.in_set(RenderSet::Prepare));
    }
}
