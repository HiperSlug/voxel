//! see bevy [`MeshAllocator`](https://github.com/bevyengine/bevy/blob/da126fa9647d7c2506126cc499c64861294a2ef7/crates/bevy_render/src/mesh/allocator.rs#L56)

mod slab {
    use bevy::render::{
        render_resource::{Buffer, BufferDescriptor, BufferUsages},
        renderer::{RenderDevice, RenderQueue},
    };
    use offset_allocator::{Allocation, Allocator};

    pub struct Slab {
        pub buffer: Buffer,
        pub allocator: Allocator,
        pub size: u32,
    }

    impl Slab {
        pub fn new(
            device: &RenderDevice,
            label: &str,
            size: u32,
            buffer_usages: BufferUsages,
        ) -> Self {
            let buffer = device.create_buffer(&BufferDescriptor {
                label: Some(label),
                size: size as u64,
                usage: BufferUsages::COPY_DST | buffer_usages,
                mapped_at_creation: false,
            });

            let allocator = Allocator::new(size);

            Self {
                buffer,
                allocator,
                size,
            }
        }

        pub fn try_store(&mut self, queue: &RenderQueue, data: &[u8]) -> Option<SlabAllocation> {
            let size = data.len() as u32;

            let allocation = self.allocator.allocate(size)?;
            queue.write_buffer(&self.buffer, allocation.offset as u64, data);

            Some(SlabAllocation { allocation, size })
        }

        pub fn free(&mut self, slab_allocation: &SlabAllocation) {
            self.allocator.free(slab_allocation.allocation);
        }

        pub fn is_empty(&self) -> bool {
            self.allocator.storage_report().total_free_space == self.size
        }
    }

    pub struct SlabAllocation {
        pub allocation: Allocation,
        pub size: u32,
    }
}

use bytemuck::{Pod, cast_slice};
use slab::{Slab, SlabAllocation};

use bevy::{
    prelude::*,
    render::{
        render_resource::{BufferUsages, COPY_BUFFER_ALIGNMENT},
        renderer::{RenderDevice, RenderQueue},
    },
};

#[derive(Resource)]
pub struct BufferAllocator {
    slabs: Vec<Option<Slab>>,
    recycle: Vec<usize>,
    buffer_usages: BufferUsages,
}

impl BufferAllocator {
    pub fn new(buffer_usages: BufferUsages) -> Self {
        Self {
            slabs: Vec::new(),
            recycle: Vec::new(),
            buffer_usages,
        }
    }

    pub fn store<T>(
        &mut self,
        data: &[T],
        settings: &BufferAllocatorSettings,
        device: &RenderDevice,
        queue: &RenderQueue,
    ) -> BufferAllocation
    where
        T: Pod,
    {
        let data = cast_slice(data);

        for (slab_index, slab) in self
            .slabs
            .iter_mut()
            .enumerate()
            .filter_map(|(i, s)| s.as_mut().map(|s| (i, s)))
        {
            if let Some(slab_allocation) = slab.try_store(queue, data) {
                return BufferAllocation {
                    slab_index,
                    slab_allocation,
                };
            }
        }

        let size = u32::max(
            settings.slab_size,
            (data.len() as u32).next_multiple_of(COPY_BUFFER_ALIGNMENT as u32),
        );

        let (is_recycled, slab_index) = self.next_slab_index();

        let mut slab = Slab::new(
            device,
            &format!("Slab {{ index: {slab_index}, size: {size} }}"),
            size,
            self.buffer_usages,
        );
        let slab_allocation = slab.try_store(queue, data).unwrap();

        if is_recycled {
            self.slabs[slab_index] = Some(slab);
        } else {
            self.slabs.push(Some(slab));
        }

        BufferAllocation {
            slab_index,
            slab_allocation,
        }
    }

    pub fn next_slab_index(&mut self) -> (bool, usize) {
        match self.recycle.pop() {
            Some(i) => (true, i),
            None => (false, self.slabs.len()),
        }
    }

    pub fn free(&mut self, buffer_allocation: &BufferAllocation) -> Result<(), ()> {
        let slab = self.slabs[buffer_allocation.slab_index]
            .as_mut()
            .ok_or(())?;

        slab.free(&buffer_allocation.slab_allocation);

        if slab.is_empty() {
            self.slabs[buffer_allocation.slab_index] = None;
            self.recycle.push(buffer_allocation.slab_index);
        }

        Ok(())
    }
}

pub struct BufferAllocation {
    slab_index: usize,
    slab_allocation: SlabAllocation,
}

impl BufferAllocation {
    pub fn slab_index(&self) -> usize {
        self.slab_index
    }

    pub fn offset(&self) -> u32 {
        self.slab_allocation.allocation.offset
    }

    pub fn size(&self) -> u32 {
        self.slab_allocation.size
    }
}

impl Into<DropEvent> for BufferAllocation {
    fn into(self) -> DropEvent {
        DropEvent {
            buffer_allocation: self,
        }
    }
}

#[derive(Event)]
struct DropEvent {
    buffer_allocation: BufferAllocation,
}

pub fn free_allocations(
    mut buffer_allocator: ResMut<BufferAllocator>,
    mut events: EventReader<DropEvent>,
) {
    for event in events.read() {
        // invariant: only this module can construct `BufferAllocation`
        buffer_allocator.free(&event.buffer_allocation).unwrap();
    }
}

#[derive(Resource)]
pub struct BufferAllocatorSettings {
    pub slab_size: u32,
}

impl Default for BufferAllocatorSettings {
    fn default() -> Self {
        const MIB: u32 = 2u32.pow(20);
        Self { slab_size: 8 * MIB }
    }
}
