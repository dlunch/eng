use core::{marker::PhantomData, mem::size_of};

use zerocopy::AsBytes;

use crate::utils::round_up;

use super::{buffer_pool::BufferPool, resource::Resource, Buffer};

pub struct DynamicUniformBuffer<T>
where
    T: AsBytes,
{
    buffer: Buffer,
    item_size: u32,
    _phantom: PhantomData<T>,
}

impl<T> DynamicUniformBuffer<T>
where
    T: AsBytes,
{
    pub(crate) fn with_buffer_pool(buffer_pool: &BufferPool, count: usize) -> Self {
        let alignment = 256; // TODO limts
        let item_size = round_up(size_of::<T>(), alignment) as u32;

        let buffer = buffer_pool.alloc(item_size as u64 * count as u64);

        Self {
            buffer,
            item_size,
            _phantom: PhantomData::default(),
        }
    }

    pub fn write(&mut self, index: usize, data: &T) {
        let offset = self.item_size * index as u32;

        self.buffer.write(offset as u64, data.as_bytes());
    }

    pub fn offset_for_index(&self, index: usize) -> u32 {
        self.item_size as u32 * index as u32
    }
}

impl<T> Resource for DynamicUniformBuffer<T>
where
    T: AsBytes,
{
    fn wgpu_resource(&self) -> wgpu::BindingResource {
        self.buffer.wgpu_resource()
    }
}
