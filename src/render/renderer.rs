use alloc::{boxed::Box, sync::Arc, vec};
use core::ops::Range;

use raw_window_handle::HasRawWindowHandle;
use zerocopy::AsBytes;

use super::{
    buffer_pool::BufferPool,
    camera::Camera,
    components::{CameraComponent, RenderComponent},
    pipeline_cache::PipelineCache,
    render_target::OffscreenRenderTarget,
    transform::Transform,
    uniform_buffer::DynamicUniformBuffer,
    Material, Mesh, RenderTarget, Shader, VertexFormat, VertexFormatItem, VertexItemType, WindowRenderTarget,
};
use crate::ecs::World;

#[derive(AsBytes)]
#[repr(C)]
pub(crate) struct ShaderTransform {
    pub model: [f32; 16],
    pub view: [f32; 16],
    pub projection: [f32; 16],
}

pub struct Renderer {
    pub(crate) device: Arc<wgpu::Device>,
    pub(crate) shader_transform: DynamicUniformBuffer<ShaderTransform>,
    pub buffer_pool: BufferPool,

    pub(crate) queue: Arc<wgpu::Queue>,

    render_target: Box<dyn RenderTarget>,

    offscreen_target: OffscreenRenderTarget,
    offscreen_to_render_target_component: RenderComponent,
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

        let limits = adapter.limits();

        log::info!("{:?}", adapter.get_info());
        log::info!("{:?}", adapter.features());
        log::info!("{:?}", limits);
        log::info!("{:?}", adapter.get_downlevel_properties());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits,
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

        let (offscreen_target, offscreen_to_render_target_component) =
            Self::create_offscreen_target(&device, &pipeline_cache, &buffer_pool, width, height, render_target.output_format());

        let shader_transform = DynamicUniformBuffer::with_buffer_pool(&buffer_pool, 64); // TODO realloc

        Self {
            device,
            shader_transform,
            buffer_pool,
            queue,
            render_target,
            offscreen_target,
            offscreen_to_render_target_component,
            pipeline_cache,
        }
    }

    pub fn render_world(&mut self, world: &World) {
        let render_components = world.components::<RenderComponent>().map(|x| x.1);
        let camera = &world.components::<CameraComponent>().next().unwrap().1.camera;

        self.render_components(camera.as_ref(), render_components);
    }

    fn render_components<'a, T>(&mut self, camera: &dyn Camera, components: T)
    where
        T: Iterator<Item = &'a RenderComponent>,
    {
        let transform = ShaderTransform {
            model: [1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.],
            view: camera.view().to_cols_array(),
            projection: camera.projection().to_cols_array(),
        };
        self.shader_transform.write(0, &transform);

        let mut command_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.render_scene(&mut command_encoder, components, self.render_target.size());
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
        format: wgpu::TextureFormat,
    ) -> (OffscreenRenderTarget, RenderComponent) {
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

        let material = Material::with_device(device, None, &[("texture", offscreen_target.color_attachment.clone())], Arc::new(shader));

        (
            offscreen_target,
            RenderComponent::with_device(device, pipeline_cache, mesh, material, Transform::new(), format, None),
        )
    }

    fn render_scene<'a, T>(&self, command_encoder: &mut wgpu::CommandEncoder, components: T, viewport_size: (u32, u32))
    where
        T: Iterator<Item = &'a RenderComponent>,
    {
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

        for component in components {
            Self::render_ranges(
                &component.mesh,
                &component.material,
                &component.pipeline,
                &mut render_pass,
                &[0..component.mesh.index_count as u32],
            );
        }
    }

    fn render_ranges<'a>(
        mesh: &'a Mesh,
        material: &'a Material,
        pipeline: &'a wgpu::RenderPipeline,
        render_pass: &mut wgpu::RenderPass<'a>,
        ranges: &[Range<u32>],
    ) {
        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, &material.bind_group, &[]);
        render_pass.set_index_buffer(mesh.index_buffer.as_slice(), wgpu::IndexFormat::Uint16);
        for (i, vertex_buffer) in mesh.vertex_buffers.iter().enumerate() {
            render_pass.set_vertex_buffer(i as u32, vertex_buffer.as_slice());
        }

        let mut last_start = ranges[0].start;
        let mut last_end = ranges[0].start;
        for range in ranges {
            if last_end != range.start {
                render_pass.draw_indexed(last_start..last_end, 0, 0..1);
                last_start = range.start;
            }
            last_end = range.end;
        }
        render_pass.draw_indexed(last_start..last_end, 0, 0..1);
    }

    fn present(&self, command_encoder: &mut wgpu::CommandEncoder, target: &dyn RenderTarget) {
        let pipeline = self.pipeline_cache.get(
            &self.device,
            &self.offscreen_to_render_target_component.material.shader,
            &self.offscreen_to_render_target_component.mesh.vertex_formats,
            target.output_format(),
            None,
        );
        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            Self::render_ranges(
                &self.offscreen_to_render_target_component.mesh,
                &self.offscreen_to_render_target_component.material,
                &pipeline,
                &mut render_pass,
                &[0..self.offscreen_to_render_target_component.mesh.index_count as u32],
            );
        }
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
