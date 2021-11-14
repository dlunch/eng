use alloc::{string::String, vec::Vec};

use hashbrown::HashMap;

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum VertexItemType {
    UByte4,
    Float2,
    Float3,
    Float4,
    Half2,
    Half4,
}

impl VertexItemType {
    pub(crate) fn wgpu_type(&self) -> wgpu::VertexFormat {
        match self {
            VertexItemType::UByte4 => wgpu::VertexFormat::Uint8x4,
            VertexItemType::Float2 => wgpu::VertexFormat::Float32x2,
            VertexItemType::Float3 => wgpu::VertexFormat::Float32x3,
            VertexItemType::Float4 => wgpu::VertexFormat::Float32x4,
            VertexItemType::Half2 => wgpu::VertexFormat::Float16x2,
            VertexItemType::Half4 => wgpu::VertexFormat::Float16x4,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct VertexFormatItem {
    shader_name: &'static str,
    item_type: VertexItemType,
    offset: usize,
}

impl VertexFormatItem {
    pub fn new(shader_name: &'static str, item_type: VertexItemType, offset: usize) -> Self {
        Self {
            shader_name,
            item_type,
            offset,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct VertexFormat {
    pub items: Vec<VertexFormatItem>,
    pub stride: usize,
}

impl VertexFormat {
    pub fn new(items: Vec<VertexFormatItem>, stride: usize) -> Self {
        Self { items, stride }
    }

    pub(crate) fn wgpu_attributes(&self, shader_inputs: &HashMap<String, u32>) -> Vec<wgpu::VertexAttribute> {
        self.items
            .iter()
            .map(|x| wgpu::VertexAttribute {
                format: x.item_type.wgpu_type(),
                offset: x.offset as u64,
                shader_location: *shader_inputs.get(x.shader_name).unwrap(),
            })
            .collect::<Vec<_>>()
    }
}
