use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use core::{
    cmp::{Eq, PartialEq},
    hash::{Hash, Hasher},
};

use hashbrown::HashMap;
use log::trace;
use spinning_top::Spinlock;

use super::{Shader, VertexFormat};

struct PipelineCacheKey {
    shader: Weak<Shader>,
    vertex_formats: Vec<VertexFormat>,
}

impl PipelineCacheKey {
    pub fn new(shader: &Arc<Shader>, vertex_formats: &[VertexFormat]) -> Self {
        Self {
            shader: Arc::downgrade(shader),
            vertex_formats: vertex_formats.to_vec(),
        }
    }
}

impl PartialEq for PipelineCacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.shader.ptr_eq(&other.shader) && self.vertex_formats == other.vertex_formats
    }
}

impl Eq for PipelineCacheKey {}

impl Hash for PipelineCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ptr = self.shader.as_ptr();
        let ptr_num = ptr as usize; // workaround for https://github.com/rust-lang/rust/issues/85447
        ptr_num.hash(state);

        self.vertex_formats.hash(state);
    }
}

pub struct PipelineCache {
    caches: Spinlock<HashMap<PipelineCacheKey, Arc<wgpu::RenderPipeline>>>,
}

impl PipelineCache {
    pub fn new() -> Self {
        Self {
            caches: Spinlock::new(HashMap::new()),
        }
    }

    pub fn get(
        &self,
        device: &wgpu::Device,
        shader: &Arc<Shader>,
        vertex_formats: &[VertexFormat],
        surface_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Arc<wgpu::RenderPipeline> {
        let key = PipelineCacheKey::new(shader, vertex_formats);

        let mut caches = self.caches.lock();

        if let Some(x) = caches.get(&key) {
            trace!("Pipeline Cache Hit");

            x.clone()
        } else {
            trace!("Pipeline Cache Miss");

            let pipeline = Self::create(device, shader, vertex_formats, surface_format, depth_format);
            caches.insert(key, pipeline.clone());

            pipeline
        }
    }

    fn create(
        device: &wgpu::Device,
        shader: &Arc<Shader>,
        vertex_formats: &[VertexFormat],
        surface_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Arc<wgpu::RenderPipeline> {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[&shader.bind_group_layout],
        });

        let attributes = vertex_formats.iter().map(|x| x.wgpu_attributes(&shader.inputs)).collect::<Vec<_>>();

        let vertex_buffers = attributes
            .iter()
            .zip(vertex_formats.iter())
            .map(|(attributes, vertex_format)| wgpu::VertexBufferLayout {
                array_stride: vertex_format.stride as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes,
            })
            .collect::<Vec<_>>();

        Arc::new(device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader.module,
                entry_point: &shader.vs_entry,
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader.module,
                entry_point: &shader.fs_entry,
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            operation: wgpu::BlendOperation::Add,
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        },
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: depth_format.map(|x| wgpu::DepthStencilState {
                format: x,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            label: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        }))
    }
}
