use alloc::sync::Arc;
use core::ops::Range;

use super::{constants::INTERNAL_COLOR_ATTACHMENT_FORMAT, pipeline_cache::PipelineCache, Material, Mesh, RenderContext, Renderable, Renderer};

pub struct Model {
    mesh: Mesh,
    material: Material,
    pipeline: Arc<wgpu::RenderPipeline>,
}

impl Model {
    pub fn new(renderer: &Renderer, mesh: Mesh, material: Material) -> Self {
        Self::with_surface_and_depth_format(
            &*renderer.device,
            &renderer.pipeline_cache,
            mesh,
            material,
            INTERNAL_COLOR_ATTACHMENT_FORMAT.wgpu_type(),
            Some(wgpu::TextureFormat::Depth32Float),
        )
    }

    pub(crate) fn with_surface_and_depth_format(
        device: &wgpu::Device,
        pipeline_cache: &PipelineCache,
        mesh: Mesh,
        material: Material,
        surface_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Self {
        let pipeline = pipeline_cache.get(device, &material.shader, &mesh.vertex_formats, surface_format, depth_format);

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
