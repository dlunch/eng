use alloc::{sync::Arc, vec};

use hashbrown::HashMap;
use nalgebra::Matrix4;
use zerocopy::AsBytes;

use crate::{
    buffer::Buffer, buffer_pool::BufferPool, render_target::OffscreenRenderTarget, Camera, Material, Mesh, Model, RenderContext, RenderTarget,
    Renderable, Scene, Shader, ShaderBinding, ShaderBindingType, VertexFormat, VertexFormatItem, VertexItemType,
};

// Copied from https://github.com/bluss/maplit/blob/master/src/lib.rs#L46
macro_rules! hashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = hashmap!(@count $($key),*);
            let mut _map = HashMap::with_capacity(_cap);
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    };
}

pub struct Renderer {
    pub(crate) instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: Arc<wgpu::Device>,
    pub(crate) mvp_buf: Buffer,
    pub buffer_pool: BufferPool,

    pub(crate) queue: Arc<wgpu::Queue>,

    offscreen_target: Option<OffscreenRenderTarget>,
    offscreen_model: Option<Model>,
    offscreen_size: Option<(u32, u32)>,
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
            adapter,
            device,
            queue,
            buffer_pool,
            mvp_buf,
            offscreen_target: None,
            offscreen_model: None,
            offscreen_size: None,
        }
    }

    pub fn render(&mut self, scene: &Scene<'_>, target: &mut dyn RenderTarget) {
        let size = target.size();

        if self.offscreen_target.is_none() || self.offscreen_size.unwrap() != target.size() {
            self.reset_offscreen_pipeline(size, target.output_format());
        }

        let mvp = Self::get_mvp(&scene.camera, size.0 as f32 / size.1 as f32);
        self.mvp_buf.write(mvp.as_slice().as_bytes());

        let mut command_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        Self::render_scene(&mut command_encoder, scene, self.offscreen_target.as_ref().unwrap(), target.size());
        self.present(&mut command_encoder, target);

        self.queue.submit(Some(command_encoder.finish()));
        target.submit();
    }

    fn reset_offscreen_pipeline(&mut self, new_size: (u32, u32), surface_format: wgpu::TextureFormat) {
        let texture_width = Self::round_up_power_of_two(new_size.0);
        let texture_height = Self::round_up_power_of_two(new_size.1);

        self.offscreen_target = Some(OffscreenRenderTarget::new(self, texture_width, texture_height));
        self.offscreen_size = Some(new_size);

        let right = new_size.0 as f32 / texture_width as f32;
        let bottom = new_size.1 as f32 / texture_height as f32;

        #[rustfmt::skip]
        let quad = [
            -1.0,  1.0, 0.0,   0.0,
            -1.0, -1.0, 0.0,   bottom,
             1.0, -1.0, right, bottom,
            -1.0,  1.0, 0.0,   0.0,
             1.0, -1.0, right, bottom,
             1.0,  1.0, right, 0.0,
        ];

        let mesh = Mesh::new(
            self,
            &[quad.as_bytes()],
            &[core::mem::size_of::<f32>() * 4],
            [0u16, 1, 2, 3, 4, 5].as_bytes(),
            vec![VertexFormat::new(vec![
                VertexFormatItem::new("Position", VertexItemType::Float2, 0),
                VertexFormatItem::new("TexCoord", VertexItemType::Float2, core::mem::size_of::<f32>() * 2),
            ])],
        );

        let vs = Shader::new(
            self,
            include_bytes!("../shaders/vertex.vert.spv"),
            "main",
            HashMap::new(),
            hashmap! {
                    "Position" => 0,
                    "TexCoord" => 1,
            },
        );

        let fs = Shader::new(
            self,
            include_bytes!("../shaders/fragment.frag.spv"),
            "main",
            hashmap! {
                "Texture" => ShaderBinding::new(1, ShaderBindingType::Texture2D),
                "Sampler" => ShaderBinding::new(2, ShaderBindingType::Sampler),
            },
            HashMap::new(),
        );

        let material = Material::new(
            self,
            hashmap! {"Texture" => self.offscreen_target.as_ref().unwrap().color_attachment.clone()},
            HashMap::new(),
            Arc::new(vs),
            Arc::new(fs),
        );

        self.offscreen_model = Some(Model::with_surface_and_depth_format(
            self,
            mesh,
            material,
            vec![0..6],
            surface_format,
            None,
        ));
    }

    fn render_scene(command_encoder: &mut wgpu::CommandEncoder, scene: &Scene<'_>, target: &OffscreenRenderTarget, viewport_size: (u32, u32)) {
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: target.color_attachment(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 1., g: 1., b: 1., a: 1. }),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &target.depth_attachment.texture_view,
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

        self.offscreen_model.as_ref().unwrap().render(&mut render_context);
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
