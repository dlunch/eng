use alloc::{boxed::Box, sync::Arc, vec};
use core::ops::Drop;

use super::resource::Resource;

pub struct Buffer {
    queue: Arc<wgpu::Queue>,
    pub(crate) buffer: Arc<wgpu::Buffer>,
    pub(crate) offset: u64,
    size: u64,
    free: Box<dyn Fn() + Sync + Send + 'static>,
}

impl Buffer {
    pub(crate) fn new<F>(queue: Arc<wgpu::Queue>, buffer: Arc<wgpu::Buffer>, offset: u64, size: u64, free: F) -> Self
    where
        F: Fn() + Sync + Send + 'static,
    {
        Self {
            queue,
            buffer,
            offset,
            size,
            free: Box::new(free),
        }
    }

    pub fn write(&self, offset: u64, data: &[u8]) {
        assert!(offset < self.size);

        // TODO raise error or warn
        if data.len() % wgpu::COPY_BUFFER_ALIGNMENT as usize != 0 {
            let count = data.len() % wgpu::COPY_BUFFER_ALIGNMENT as usize;
            let mut new_buf = vec![0; data.len() + count];
            new_buf[..data.len()].copy_from_slice(data);

            self.queue.write_buffer(&self.buffer, self.offset + offset, &new_buf)
        } else {
            self.queue.write_buffer(&self.buffer, self.offset + offset, data)
        }
    }

    pub(crate) fn as_slice(&self) -> wgpu::BufferSlice {
        self.buffer.slice(self.offset as u64..self.offset as u64 + self.size as u64)
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        (self.free)()
    }
}

impl Resource for Buffer {
    fn wgpu_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &self.buffer,
            offset: self.offset as wgpu::BufferAddress,
            size: wgpu::BufferSize::new(self.size as u64),
        })
    }
}
