use bevy::{
    prelude::*,
    render::{
        render_resource::{Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor},
        renderer::{RenderDevice, RenderQueue},
    },
};
use bytemuck::{Pod, checked::cast_slice};
use freelist::{FreeList, Slice, oom::OomStrategy, search::LastFit};
use parking_lot::Mutex;
use std::{marker::PhantomData, num::NonZeroUsize, sync::Arc};

pub struct AllocatedBuffer<T> {
    label: &'static str,
    // We can only write when we know data is not being read
    write_queue: Vec<T>,
    
    buffer: Buffer,
    freelist: Arc<Mutex<FreeList>>,
}

impl<T> AllocatedBuffer<T>
where
    T: Pod,
{
    pub fn new(device: &RenderDevice, len: NonZeroUsize, label: String) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some(&label),
            size: (size_of::<T>() * len.get()) as u64,
            mapped_at_creation: false,
            usage: BufferUsages::COPY_SRC | BufferUsages::COPY_DST | BufferUsages::STORAGE,
        });

        let extent = Slice::new_from_zero(len);

        // Arbitrary number I chose
        const STARTING_CAPACITY: usize = 1024;

        let freelist = Arc::new(Mutex::new(FreeList::with_internal_capacity(
            extent,
            STARTING_CAPACITY,
        )));

        Self {
            label,
            buffer,
            freelist,
            _phantom: PhantomData,
        }
    }

    pub fn write(
        &mut self,
        device: &RenderDevice,
        queue: &RenderQueue,
        data: &[T],
    ) -> Option<BufferAllocation<T>> {
        let len = data.len().try_into().ok()?;

        let mut guard = self.freelist.lock();

        // Also an arbitrary number I chose
        let slice = match guard.alloc_first::<ConjureMore<256>>(len) {
            Ok(slice) => slice,
            Err((slice, by)) => {
                drop(guard);
                let new_size = self.expand_buffer(device, queue, by);
                info!("Expanded buffer to {new_size}");
                slice
            }
        };

        let offset = (slice.start * size_of::<T>()) as u64;
        let size = ((slice.len() * size_of::<T>()) as u64).try_into().unwrap();

        let mut view = queue.write_buffer_with(&self.buffer, offset, size).unwrap();
        view.copy_from_slice(cast_slice(data));

        Some(BufferAllocation {
            slice,
            freelist: self.freelist.clone(),
            _phantom: PhantomData,
        })
    }

    // If instead I provide already contiguous data with precomputed indices it would cut down on cost.
    pub fn multi_write_contiguous<const N: usize>(
        &mut self,
        device: &RenderDevice,
        queue: &RenderQueue,
        data: [&[T]; N],
    ) -> Option<MultiBufferAllocation<T, N>> {
        let flat_data = data
            .iter()
            .flat_map(|s| s.iter())
            .copied()
            .collect::<Vec<_>>();
        let total_len = flat_data.len().try_into().ok()?;

        let mut guard = self.freelist.lock();

        // Same arbitrary number as before
        let cumulative = match guard.alloc_first::<ConjureMore<256>>(total_len) {
            Ok(slice) => slice,
            Err((slice, by)) => {
                drop(guard);
                let new_size = self.expand_buffer(device, queue, by);
                info!("Expanded buffer to {new_size}");
                slice
            }
        };

        let offset = (cumulative.start * size_of::<T>()) as u64;
        let size = ((cumulative.len() * size_of::<T>()) as u64)
            .try_into()
            .unwrap();

        let mut view = queue.write_buffer_with(&self.buffer, offset, size).unwrap();
        view.copy_from_slice(cast_slice(&flat_data));

        let mut splitten = cumulative;

        let slices = data.map(|s| {
            let (before, after) = splitten.split(s.len());
            if let Some(after) = after {
                splitten = after
            }
            before
        });

        Some(MultiBufferAllocation {
            slices,
            cumulative,
            freelist: self.freelist.clone(),
            _phantom: PhantomData,
        })
    }

    pub fn expand_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue, by: usize) -> u64 {
        let new_size = self.buffer.size() + (by * size_of::<T>()) as u64;

        let new_buffer = device.create_buffer(&BufferDescriptor {
            label: self.label.as_ref().map(|s| s.as_str()),
            size: new_size,
            mapped_at_creation: false,
            usage: BufferUsages::COPY_SRC | BufferUsages::COPY_DST | BufferUsages::STORAGE,
        });

        let mut command_encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("AllocatedBuffer copy reallocation"),
        });
        command_encoder.copy_buffer_to_buffer(&self.buffer, 0, &new_buffer, 0, self.buffer.size());

        queue.submit([command_encoder.finish()]);

        self.buffer = new_buffer;

        new_size
    }
}

struct ConjureMore<const N: usize>;

impl<const N: usize> OomStrategy for ConjureMore<N> {
    type Output = (Slice, usize);

    fn strategy(freelist: &mut FreeList, failed_len: NonZeroUsize) -> Self::Output {
        let end = freelist.extent().end();

        let trailing_len = failed_len.get().next_multiple_of(N);

        let trailing_slice = Slice {
            start: end,
            len: trailing_len.try_into().unwrap(),
        };

        unsafe {
            freelist.dealloc(&trailing_slice).unwrap();
        }

        let slice = freelist.alloc_infallible::<LastFit>(failed_len);

        (slice, trailing_len)
    }
}

#[derive(Debug, Deref)]
pub struct BufferAllocation<T> {
    #[deref]
    slice: Slice,
    freelist: Arc<Mutex<FreeList>>,
    _phantom: PhantomData<T>,
}

impl<T> Drop for BufferAllocation<T> {
    fn drop(&mut self) {
        let mut guard = self.freelist.lock();
        unsafe {
            guard.dealloc(&self.slice).unwrap();
        }
    }
}

#[derive(Debug, Deref)]
pub struct MultiBufferAllocation<T, const N: usize> {
    #[deref]
    slices: [Option<Slice>; N],
    cumulative: Slice,
    freelist: Arc<Mutex<FreeList>>,
    _phantom: PhantomData<T>,
}

impl<T, const N: usize> Drop for MultiBufferAllocation<T, N> {
    fn drop(&mut self) {
        let mut guard = self.freelist.lock();
        unsafe {
            guard.dealloc(&self.cumulative).unwrap();
        }
    }
}
