use alloc::sync::Arc;

use crate::ecs::Component;

use super::{constants::INTERNAL_COLOR_ATTACHMENT_FORMAT, pipeline_cache::PipelineCache, Material, Mesh, Renderer};

pub struct RenderComponent {
    pub mesh: Mesh,
    pub material: Material,
    pub(crate) pipeline: Arc<wgpu::RenderPipeline>,
}

impl Component for RenderComponent {}

impl RenderComponent {
    pub fn new(renderer: &Renderer, mesh: Mesh, material: Material) -> Self {
        Self::with_device(
            &renderer.device,
            &renderer.pipeline_cache,
            mesh,
            material,
            INTERNAL_COLOR_ATTACHMENT_FORMAT.wgpu_type(),
            Some(wgpu::TextureFormat::Depth32Float),
        )
    }

    pub(crate) fn with_device(
        device: &wgpu::Device,
        pipeline_cache: &PipelineCache,
        mesh: Mesh,
        material: Material,
        format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Self {
        let pipeline = pipeline_cache.get(device, &material.shader, &mesh.vertex_formats, format, depth_format);

        Self { mesh, material, pipeline }
    }
}
