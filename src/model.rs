use alloc::vec::Vec;
use core::ops::Range;

use crate::{constants::INTERNAL_COLOR_ATTACHMENT_FORMAT, Material, Mesh, RenderContext, Renderable, Renderer};

pub struct Model {
    mesh: Mesh,
    material: Material,
    pipeline: wgpu::RenderPipeline,
}

impl Model {
    pub fn new(renderer: &Renderer, mesh: Mesh, material: Material) -> Self {
        Self::with_surface_and_depth_format(
            &*renderer.device,
            mesh,
            material,
            INTERNAL_COLOR_ATTACHMENT_FORMAT.wgpu_type(),
            Some(wgpu::TextureFormat::Depth32Float),
        )
    }

    pub(crate) fn with_surface_and_depth_format(
        device: &wgpu::Device,
        mesh: Mesh,
        material: Material,
        surface_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Self {
        let attributes = mesh
            .vertex_formats
            .iter()
            .map(|x| x.wgpu_attributes(&material.shader.inputs))
            .collect::<Vec<_>>();

        let vertex_buffers = attributes
            .iter()
            .zip(mesh.strides.iter())
            .map(|(attributes, stride)| wgpu::VertexBufferLayout {
                array_stride: *stride as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes,
            })
            .collect::<Vec<_>>();

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: Some(&material.pipeline_layout),
            vertex: wgpu::VertexState {
                module: &material.shader.module,
                entry_point: material.shader.vs_entry,
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &material.shader.module,
                entry_point: material.shader.fs_entry,
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
        });

        Self { mesh, material, pipeline }
    }

    pub fn render_ranges<'a>(&'a self, render_context: &mut RenderContext<'a>, ranges: &[Range<u32>]) {
        render_context.render_pass.set_pipeline(&self.pipeline);
        render_context.render_pass.set_bind_group(0, &self.material.bind_group, &[]);
        render_context
            .render_pass
            .set_index_buffer(self.mesh.index_buffer.as_slice(), wgpu::IndexFormat::Uint16);
        for (i, vertex_buffer) in self.mesh.vertex_buffers.iter().enumerate() {
            render_context.render_pass.set_vertex_buffer(i as u32, vertex_buffer.as_slice());
        }

        let mut last_start = ranges[0].start;
        let mut last_end = ranges[0].start;
        for range in ranges {
            if last_end != range.start {
                render_context.render_pass.draw_indexed(last_start..last_end, 0, 0..1);
                last_start = range.start;
            }
            last_end = range.end;
        }
        render_context.render_pass.draw_indexed(last_start..last_end, 0, 0..1);
    }
}

impl Renderable for Model {
    fn render<'a>(&'a self, render_context: &mut RenderContext<'a>) {
        self.render_ranges(render_context, &[0..self.mesh.index_count as u32]);
    }
}
