use alloc::vec::Vec;
use core::ops::Range;

use crate::{constants::INTERNAL_COLOR_ATTACHMENT_FORMAT, Material, Mesh, RenderContext, Renderable, Renderer};

pub struct Model {
    mesh: Mesh,
    material: Material,
    mesh_parts: Vec<Range<u32>>,

    pipeline: wgpu::RenderPipeline,
}

impl Model {
    pub fn new(renderer: &Renderer, mesh: Mesh, material: Material, mesh_parts: Vec<Range<u32>>) -> Self {
        Self::with_surface_and_depth_format(
            renderer,
            mesh,
            material,
            mesh_parts,
            INTERNAL_COLOR_ATTACHMENT_FORMAT.wgpu_type(),
            Some(wgpu::TextureFormat::Depth32Float),
        )
    }

    pub(crate) fn with_surface_and_depth_format(
        renderer: &Renderer,
        mesh: Mesh,
        material: Material,
        mesh_parts: Vec<Range<u32>>,
        surface_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Self {
        let attributes = mesh
            .vertex_formats
            .iter()
            .map(|x| x.wgpu_attributes(&material.vertex_shader.inputs))
            .collect::<Vec<_>>();

        let vertex_buffers = attributes
            .iter()
            .zip(mesh.strides.iter())
            .map(|(attributes, stride)| wgpu::VertexBufferLayout {
                array_stride: *stride as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes,
            })
            .collect::<Vec<_>>();

        let pipeline = renderer.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: Some(&material.pipeline_layout),
            vertex: wgpu::VertexState {
                module: &material.vertex_shader.module,
                entry_point: material.vertex_shader.entry,
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &material.fragment_shader.module,
                entry_point: material.fragment_shader.entry,
                targets: &[wgpu::ColorTargetState {
                    format: surface_format,
                    color_blend: wgpu::BlendState {
                        operation: wgpu::BlendOperation::Add,
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    },
                    alpha_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default()
            },
            depth_stencil: depth_format.map(|x| wgpu::DepthStencilState {
                format: x,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                clamp_depth: false,
            }),
            label: None,
            multisample: wgpu::MultisampleState::default(),
        });

        Self {
            mesh,
            material,
            pipeline,
            mesh_parts,
        }
    }
}

impl Renderable for Model {
    fn render<'a>(&'a self, render_context: &mut RenderContext<'a>) {
        render_context.render_pass.set_pipeline(&self.pipeline);
        render_context.render_pass.set_bind_group(0, &self.material.bind_group, &[]);
        render_context
            .render_pass
            .set_index_buffer(self.mesh.index_buffer.as_slice(), wgpu::IndexFormat::Uint16);
        for (i, vertex_buffer) in self.mesh.vertex_buffers.iter().enumerate() {
            render_context.render_pass.set_vertex_buffer(i as u32, vertex_buffer.as_slice());
        }

        let mut last_start = self.mesh_parts[0].start;
        let mut last_end = self.mesh_parts[0].start;
        for mesh_part in &self.mesh_parts {
            if last_end != mesh_part.start {
                render_context.render_pass.draw_indexed(last_start..last_end, 0, 0..1);
                last_start = mesh_part.start;
            }
            last_end = mesh_part.end;
        }
        render_context.render_pass.draw_indexed(last_start..last_end, 0, 0..1);
    }
}
