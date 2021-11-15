use alloc::{boxed::Box, sync::Arc, vec};

use raw_window_handle::HasRawWindowHandle;
use zerocopy::AsBytes;

use super::{
    buffer::Buffer,
    buffer_pool::BufferPool,
    camera::{Camera, CameraController},
    pipeline_cache::PipelineCache,
    render_target::OffscreenRenderTarget,
    Material, Mesh, Model, RenderContext, RenderTarget, Renderable, Scene, Shader, VertexFormat, VertexFormatItem, VertexItemType,
    WindowRenderTarget,
};

pub struct Renderer {
    pub(crate) device: Arc<wgpu::Device>,
    pub(crate) mvp_buf: Buffer,
    pub buffer_pool: BufferPool,

    pub(crate) queue: Arc<wgpu::Queue>,

    render_target: Box<dyn RenderTarget>,

    offscreen_target: OffscreenRenderTarget,
    offscreen_to_render_target_model: Model,
    pub(crate) pipeline_cache: PipelineCache,
}

impl Renderer {
    pub async fn new<W: HasRawWindowHandle>(window: &W, width: u32, height: u32) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let buffer_pool = BufferPool::new(device.clone(), queue.clone());
        let pipeline_cache = PipelineCache::new();

        let render_target = Box::new(WindowRenderTarget::new(surface, &adapter, &device, width, height));

        let (offscreen_target, offscreen_to_render_target_model) =
            Self::create_offscreen_target(&device, &pipeline_cache, &buffer_pool, width, height, render_target.output_format());

        let mvp_buf = buffer_pool.alloc(64);

        Self {
            device,
            mvp_buf,
            buffer_pool,
            queue,
            render_target,
            offscreen_target,
            offscreen_to_render_target_model,
            pipeline_cache,
        }
    }

    pub fn render<T: CameraController>(&mut self, camera: &Camera<T>, scene: &Scene) {
        let mvp = camera.projection() * camera.view();
        self.mvp_buf.write(mvp.as_slice().as_bytes());

        let mut command_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.render_scene(&mut command_encoder, scene, self.render_target.size());
        self.present(&mut command_encoder, &*self.render_target);

        self.queue.submit(Some(command_encoder.finish()));
        self.render_target.submit();
    }

    fn create_offscreen_target(
        device: &wgpu::Device,
        pipeline_cache: &PipelineCache,
        buffer_pool: &BufferPool,
        width: u32,
        height: u32,
        surface_format: wgpu::TextureFormat,
    ) -> (OffscreenRenderTarget, Model) {
        let texture_width = Self::round_up_power_of_two(width);
        let texture_height = Self::round_up_power_of_two(height);
        let offscreen_target = OffscreenRenderTarget::with_device(device, texture_width, texture_height);

        let right = width as f32 / texture_width as f32;
        let bottom = height as f32 / texture_height as f32;

        #[rustfmt::skip]
        let quad = [
            -1.0,  1.0, 0.0,   0.0,
            -1.0, -1.0, 0.0,   bottom,
             1.0, -1.0, right, bottom,
            -1.0,  1.0, 0.0,   0.0,
             1.0, -1.0, right, bottom,
             1.0,  1.0, right, 0.0,
        ];

        let mesh = Mesh::with_buffer_pool(
            buffer_pool,
            &[quad.as_bytes()],
            &[0u16, 1, 2, 3, 4, 5],
            vec![VertexFormat::new(
                vec![
                    VertexFormatItem::new("position", VertexItemType::Float2, 0),
                    VertexFormatItem::new("tex_coord", VertexItemType::Float2, core::mem::size_of::<f32>() * 2),
                ],
                core::mem::size_of::<f32>() * 4,
            )],
        );

        let shader = Shader::with_device(device, include_str!("./shaders/shader.wgsl"));

        let material = Material::with_device(
            device,
            None,
            &[("texture", offscreen_target.color_attachment.clone())],
            &[],
            Arc::new(shader),
        );

        (
            offscreen_target,
            Model::with_surface_and_depth_format(device, pipeline_cache, mesh, material, surface_format, None),
        )
    }

    fn render_scene(&self, command_encoder: &mut wgpu::CommandEncoder, scene: &Scene, viewport_size: (u32, u32)) {
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: self.offscreen_target.color_attachment(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 1., g: 1., b: 1., a: 1. }),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.offscreen_target.depth_attachment.texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
            label: None,
        });
        render_pass.set_viewport(0.0, 0.0, viewport_size.0 as f32, viewport_size.1 as f32, 0.0, 1.0);
        let mut render_context = RenderContext::new(render_pass);

        for model in &scene.models {
            model.render(&mut render_context);
        }
    }

    fn present(&self, command_encoder: &mut wgpu::CommandEncoder, target: &dyn RenderTarget) {
        let render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: target.color_attachment(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 1., g: 1., b: 1., a: 1. }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
            label: None,
        });

        let mut render_context = RenderContext::new(render_pass);

        self.offscreen_to_render_target_model.render(&mut render_context);
    }

    //returns zero if v is zero.
    fn round_up_power_of_two(mut v: u32) -> u32 {
        //from http://graphics.stanford.edu/~seander/bithacks.html#RoundUpPowerOf2 (public domain)

        v -= 1;
        v |= v >> 1;
        v |= v >> 2;
        v |= v >> 4;
        v |= v >> 8;
        v |= v >> 16;
        v += 1;

        v
    }
}