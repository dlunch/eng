use alloc::{sync::Arc, vec::Vec};

use nalgebra::Matrix4;
use spinning_top::Spinlock;
use zerocopy::AsBytes;

use crate::{buffer::Buffer, buffer_pool::BufferPool, Camera, RenderContext, RenderTarget, Scene};

type TextureUploadItem = (wgpu::Buffer, wgpu::Texture, usize, wgpu::Extent3d);

pub struct Renderer {
    pub(crate) device: Arc<wgpu::Device>,
    pub(crate) mvp_buf: Buffer,
    pub(crate) buffer_pool: BufferPool,

    queue: wgpu::Queue,

    texture_upload_queue: Spinlock<Vec<TextureUploadItem>>,
}

impl Renderer {
    pub async fn new() -> Self {
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: None,
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: wgpu::Limits::default(),
            })
            .await;

        let device = Arc::new(device);
        let buffer_pool = BufferPool::new(device.clone());

        let mvp_buf = buffer_pool.alloc(64);

        Self {
            device,
            queue,
            texture_upload_queue: Spinlock::new(Vec::new()),
            buffer_pool,
            mvp_buf,
        }
    }

    pub async fn render(&mut self, scene: &Scene<'_>, target: &mut dyn RenderTarget) {
        let size = target.size();

        let mvp = Self::get_mvp(&scene.camera, size.0 as f32 / size.1 as f32);
        self.mvp_buf.write(mvp.as_slice().as_bytes()).await.unwrap();

        let mut command_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.dequeue_texture_uploads(&mut command_encoder);

        let color_target = target.color_attachment();
        let depth_target = target.depth_attachment();
        {
            let render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: color_target,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color { r: 1., g: 1., b: 1., a: 1. },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: depth_target,
                    depth_load_op: wgpu::LoadOp::Clear,
                    depth_store_op: wgpu::StoreOp::Store,
                    stencil_load_op: wgpu::LoadOp::Clear,
                    stencil_store_op: wgpu::StoreOp::Store,
                    clear_depth: 1.,
                    clear_stencil: 0,
                }),
            });
            let mut render_context = RenderContext::new(render_pass);

            for model in &scene.models {
                model.render(&mut render_context);
            }
        }

        self.queue.submit(&[command_encoder.finish()]);
        target.submit();
    }

    pub(crate) fn enqueue_texture_upload(&self, buffer: wgpu::Buffer, texture: wgpu::Texture, bytes_per_row: usize, extent: wgpu::Extent3d) {
        let mut texture_upload_queue = self.texture_upload_queue.lock();
        texture_upload_queue.push((buffer, texture, bytes_per_row, extent));
    }

    fn dequeue_texture_uploads(&mut self, command_encoder: &mut wgpu::CommandEncoder) {
        let mut queue = Spinlock::new(Vec::new());
        core::mem::swap(&mut self.texture_upload_queue, &mut queue);

        for (buffer, texture, bytes_per_row, extent) in queue.into_inner() {
            command_encoder.copy_buffer_to_texture(
                wgpu::BufferCopyView {
                    buffer: &buffer,
                    offset: 0,
                    bytes_per_row: bytes_per_row as u32,
                    rows_per_image: 0,
                },
                wgpu::TextureCopyView {
                    texture: &texture,
                    mip_level: 0,
                    array_layer: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                extent,
            );
        }
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
