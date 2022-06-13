use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};

use spinning_top::Spinlock;

use super::buffer::Buffer;
use crate::utils::round_up;

const BUFFER_SIZE: u64 = 10485760;

struct BufferPoolItem {
    buffer: Arc<wgpu::Buffer>,
    allocated: u64,
    allocations: BTreeMap<u64, u64>,
}

impl BufferPoolItem {
    pub fn new(device: &wgpu::Device, usage: wgpu::BufferUsages) -> Self {
        let buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            size: BUFFER_SIZE as u64,
            usage,
            label: None,
            mapped_at_creation: false,
        }));

        let mut allocations = BTreeMap::new();
        allocations.insert(BUFFER_SIZE, 0);

        Self {
            buffer,
            allocated: 0,
            allocations,
        }
    }

    pub fn alloc(&mut self, size: u64) -> Option<(Arc<wgpu::Buffer>, u64)> {
        let alignment = 4096; // TODO limits
        let rounded_size = round_up(size, alignment);

        let offset = self.find_offset(rounded_size)?;

        self.allocated += rounded_size;
        self.allocations.insert(offset, rounded_size);

        Some((self.buffer.clone(), offset))
    }

    pub fn free(&mut self, offset: u64, size: u64) {
        self.allocated -= size;
        self.allocations.remove(&offset);
    }

    // simple allocator. may fragment a lot.
    fn find_offset(&self, size: u64) -> Option<u64> {
        let mut cursor = 0;
        for (allocation_offset, allocation_size) in self.allocations.iter() {
            if allocation_offset - cursor >= size {
                return Some(cursor);
            } else {
                cursor = allocation_offset + allocation_size;
            }
        }
        None
    }
}

pub struct BufferPool {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    // WebGL requires separate index buffer (https://www.khronos.org/registry/webgl/specs/latest/2.0/#5.1)
    buffers: Spinlock<Vec<Arc<Spinlock<BufferPoolItem>>>>,
    index_buffers: Spinlock<Vec<Arc<Spinlock<BufferPoolItem>>>>,
}

impl BufferPool {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            device,
            queue,
            index_buffers: Spinlock::new(Vec::new()),
            buffers: Spinlock::new(Vec::new()),
        }
    }

    pub fn alloc_index(&self, size: u64) -> Buffer {
        self.do_alloc(size, true)
    }

    pub fn alloc(&self, size: u64) -> Buffer {
        self.do_alloc(size, false)
    }

    fn do_alloc(&self, size: u64, is_index: bool) -> Buffer {
        let buffers = if is_index { &self.index_buffers } else { &self.buffers };
        let mut buffers = buffers.lock();

        for item in &*buffers {
            let result = self.try_alloc(item, size);
            if let Some(x) = result {
                return x;
            }
        }
        buffers.push(Arc::new(Spinlock::new(BufferPoolItem::new(&self.device, Self::convert_usage(is_index)))));
        self.try_alloc(buffers.last().unwrap(), size).unwrap()
    }

    fn try_alloc(&self, buffers: &Arc<Spinlock<BufferPoolItem>>, size: u64) -> Option<Buffer> {
        let (buffer, offset) = buffers.lock().alloc(size)?;

        let buffer_item = buffers.clone();
        Some(Buffer::new(self.queue.clone(), buffer, offset, size, move || {
            buffer_item.lock().free(offset, size)
        }))
    }

    fn convert_usage(is_index: bool) -> wgpu::BufferUsages {
        if is_index {
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST
        } else {
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        }
    }
}
