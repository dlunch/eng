use alloc::{boxed::Box, sync::Arc, vec};
use core::ops::Drop;

pub struct Buffer {
    queue: Arc<wgpu::Queue>,
    pub(crate) buffer: Arc<wgpu::Buffer>,
    pub(crate) offset: usize,
    size: usize,
    free: Box<dyn Fn() + Sync + Send + 'static>,
}

impl Buffer {
    pub(crate) fn new<F>(queue: Arc<wgpu::Queue>, buffer: Arc<wgpu::Buffer>, offset: usize, size: usize, free: F) -> Self
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

    pub fn write(&self, data: &[u8]) {
        // TODO raise error or warn
        if data.len() % wgpu::COPY_BUFFER_ALIGNMENT as usize != 0 {
            let count = data.len() % wgpu::COPY_BUFFER_ALIGNMENT as usize;
            let mut new_buf = vec![0; data.len() + count];
            new_buf[..data.len()].copy_from_slice(data);

            self.queue.write_buffer(&self.buffer, self.offset as u64, &new_buf)
        } else {
            self.queue.write_buffer(&self.buffer, self.offset as u64, data)
        }
    }

    pub(crate) fn binding_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::Buffer(self.as_slice())
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
