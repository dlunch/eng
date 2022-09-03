use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::ops::Range;

use super::{constants::INTERNAL_COLOR_ATTACHMENT_FORMAT, pipeline_cache::PipelineCache, transform::Transform, Camera, Material, Mesh, Renderer};
use crate::ecs::Component;

pub struct RenderComponent {
    pub mesh: Mesh,
    pub material: Material,
    pub transform: Transform,
    pub ranges: Vec<Range<u32>>,
    pub(crate) pipeline: Arc<wgpu::RenderPipeline>,
}

impl Component for RenderComponent {}

impl RenderComponent {
    pub fn new(renderer: &Renderer, mesh: Mesh, material: Material, ranges: &[Range<u32>], transform: Transform) -> Self {
        Self::with_device(
            &renderer.device,
            &renderer.pipeline_cache,
            mesh,
            material,
            ranges.to_vec(),
            transform,
            INTERNAL_COLOR_ATTACHMENT_FORMAT.wgpu_format(),
            Some(wgpu::TextureFormat::Depth32Float),
        )
    }

    pub(crate) fn with_device(
        device: &wgpu::Device,
        pipeline_cache: &PipelineCache,
        mesh: Mesh,
        material: Material,
        ranges: Vec<Range<u32>>,
        transform: Transform,
        format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Self {
        let pipeline = pipeline_cache.get(device, &material.shader, &mesh.vertex_formats, format, depth_format);

        Self {
            mesh,
            material,
            ranges,
            transform,
            pipeline,
        }
    }
}

pub struct CameraComponent {
    pub camera: Box<dyn Camera>,
}

impl Component for CameraComponent {}
