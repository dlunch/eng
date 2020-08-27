use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};

use spinning_top::Spinlock;

use crate::buffer::Buffer;

const BUFFER_SIZE: usize = 10485760;

struct BufferPoolItem {
    buffer: Arc<wgpu::Buffer>,
    allocated: usize,
    allocations: BTreeMap<usize, usize>,
}

impl BufferPoolItem {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            size: BUFFER_SIZE as u64,
            usage: wgpu::BufferUsage::INDEX | wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
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

    pub fn alloc(&mut self, size: usize) -> Option<(Arc<wgpu::Buffer>, usize)> {
        let alignment = wgpu::BIND_BUFFER_ALIGNMENT as usize;
        let rounded_size = Self::round_up(size, alignment);

        let offset = self.find_offset(rounded_size)?;

        self.allocated += rounded_size;
        self.allocations.insert(offset, rounded_size);

        Some((self.buffer.clone(), offset))
    }

    pub fn free(&mut self, offset: usize, size: usize) {
        self.allocated -= size;
        self.allocations.remove(&offset);
    }

    // simple allocator. may fragment a lot.
    fn find_offset(&self, size: usize) -> Option<usize> {
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

    fn round_up(num_to_round: usize, multiple: usize) -> usize {
        if multiple == 0 {
            return num_to_round;
        }

        let remainder = num_to_round % multiple;
        if remainder == 0 {
            num_to_round
        } else {
            num_to_round + multiple - remainder
        }
    }
}

pub struct BufferPool {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    items: Spinlock<Vec<Arc<Spinlock<BufferPoolItem>>>>,
}

impl BufferPool {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            device,
            queue,
            items: Spinlock::new(Vec::new()),
        }
    }

    pub fn alloc(&self, size: usize) -> Buffer {
        let mut items = self.items.lock();

        for item in &*items {
            let result = self.try_alloc(&item, size);
            if let Some(x) = result {
                return x;
            }
        }
        items.push(Arc::new(Spinlock::new(BufferPoolItem::new(&self.device))));
        self.try_alloc(items.last().unwrap(), size).unwrap()
    }

    fn try_alloc(&self, buffer_item: &Arc<Spinlock<BufferPoolItem>>, size: usize) -> Option<Buffer> {
        let (buffer, offset) = buffer_item.lock().alloc(size)?;

        let buffer_item = buffer_item.clone();
        Some(Buffer::new(self.queue.clone(), buffer, offset, size, move || {
            buffer_item.lock().free(offset, size)
        }))
    }
}
