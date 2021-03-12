use alloc::sync::Arc;

use nalgebra::Matrix4;
use zerocopy::AsBytes;

use crate::{buffer::Buffer, buffer_pool::BufferPool, Camera, RenderContext, RenderTarget, Scene};

pub struct Renderer {
    pub(crate) instance: wgpu::Instance,
    pub(crate) device: Arc<wgpu::Device>,
    pub(crate) mvp_buf: Buffer,
    pub buffer_pool: BufferPool,

    pub(crate) queue: Arc<wgpu::Queue>,
}

impl Renderer {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let device = Arc::new(device);
        let queue = Arc::new(queue);
        let buffer_pool = BufferPool::new(device.clone(), queue.clone());

        let mvp_buf = buffer_pool.alloc(64);

        Self {
            instance,
            device,
            queue,
            buffer_pool,
            mvp_buf,
        }
    }

    pub async fn render(&mut self, scene: &Scene<'_>, target: &mut dyn RenderTarget) {
        let size = target.size();

        let mvp = Self::get_mvp(&scene.camera, size.0 as f32 / size.1 as f32);
        self.mvp_buf.write(mvp.as_slice().as_bytes());

        let mut command_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let color_target = target.color_attachment();
        let depth_target = target.depth_attachment();
        {
            let render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: color_target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 1., g: 1., b: 1., a: 1. }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: depth_target,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
                label: None,
            });
            let mut render_context = RenderContext::new(render_pass);

            for model in &scene.models {
                model.render(&mut render_context);
            }
        }

        self.queue.submit(Some(command_encoder.finish()));
        target.submit();
    }

    fn get_mvp(camera: &Camera, aspect_ratio: f32) -> Matrix4<f32> {
        use core::f32::consts::PI;

        // nalgebra's perspective uses [-1, 1] NDC z range, so convert it to [0, 1].
        #[rustfmt::skip]
        let correction = nalgebra::Matrix4::<f32>::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.5,
            0.0, 0.0, 0.0, 1.0,
        );

        let projection = nalgebra::Matrix4::new_perspective(aspect_ratio, 45.0 * PI / 180.0, 1.0, 10.0);
        correction * projection * camera.view()
    }
}
