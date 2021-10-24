use alloc::{vec, vec::Vec};
use core::mem::size_of;

use zerocopy::AsBytes;

use crate::{buffer::Buffer, buffer_pool::BufferPool, Renderer, VertexFormat, VertexFormatItem, VertexItemType};

#[repr(C)]
#[derive(AsBytes)]
pub struct SimpleVertex {
    pub pos: [f32; 4],
    pub tex_coord: [f32; 2],
}

impl SimpleVertex {
    pub fn new(pos: [f32; 4], tex_coord: [f32; 2]) -> Self {
        Self { pos, tex_coord }
    }
}

pub struct Mesh {
    pub(crate) vertex_buffers: Vec<Buffer>,
    pub(crate) index_buffer: Buffer,
    pub(crate) index_count: usize,
    pub(crate) vertex_formats: Vec<VertexFormat>,
}

impl Mesh {
    pub fn new(renderer: &Renderer, vertex_data: &[&[u8]], indices: &[u16], vertex_formats: Vec<VertexFormat>) -> Self {
        Self::with_buffer_pool(&renderer.buffer_pool, vertex_data, indices, vertex_formats)
    }

    pub fn with_simple_vertex(renderer: &Renderer, vertices: &[SimpleVertex], indices: &[u16]) -> Self {
        let vertex_data = vertices.as_bytes();

        Self::with_buffer_pool(
            &renderer.buffer_pool,
            &[vertex_data],
            indices,
            vec![VertexFormat::new(
                vec![
                    VertexFormatItem::new("Position", VertexItemType::Float4, 0),
                    VertexFormatItem::new("TexCoord", VertexItemType::Float2, size_of::<f32>() * 4),
                ],
                size_of::<SimpleVertex>(),
            )],
        )
    }

    pub(crate) fn with_buffer_pool(buffer_pool: &BufferPool, vertex_data: &[&[u8]], indices: &[u16], vertex_formats: Vec<VertexFormat>) -> Self {
        let mut vertex_buffers = Vec::with_capacity(vertex_data.len());
        for vertex_datum in vertex_data {
            let buffer = buffer_pool.alloc(vertex_datum.len());
            buffer.write(vertex_datum);

            vertex_buffers.push(buffer);
        }

        let index_data = indices.as_bytes();
        let index_buffer = buffer_pool.alloc_index(index_data.len());
        index_buffer.write(index_data);

        Self {
            vertex_buffers,
            index_buffer,
            index_count: indices.len(),
            vertex_formats,
        }
    }
}
