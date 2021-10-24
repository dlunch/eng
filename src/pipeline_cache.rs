use alloc::{sync::Arc, vec::Vec};

use hashbrown::HashMap;
use spinning_top::Spinlock;

use crate::{Shader, VertexFormat};

static mut PIPELINE_CACHE_LAST: usize = 1;

#[derive(PartialEq, Eq, Hash)]
struct PipelineCacheKey {
    id: usize,
}

impl PipelineCacheKey {
    // TODO temporary code
    pub fn new(_: &Arc<Shader>, _: &[VertexFormat]) -> Self {
        let id = unsafe {
            let last = PIPELINE_CACHE_LAST;
            PIPELINE_CACHE_LAST += 1;
            last
        };
        Self { id }
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
            x.clone()
        } else {
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
            layout: Some(&shader.pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader.module,
                entry_point: shader.vs_entry,
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader.module,
                entry_point: shader.fs_entry,
                targets: &[wgpu::ColorTargetState {
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
                }],
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
        }))
    }
}
