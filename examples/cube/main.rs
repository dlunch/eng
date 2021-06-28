use std::sync::Arc;
use std::time::Duration;

use async_std::task;
use hashbrown::HashMap;
use nalgebra::Point3;
use winit::{
    dpi::LogicalSize,
    event,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
};
use zerocopy::AsBytes;

use renderer::{
    Camera, Material, Mesh, Model, Renderer, Scene, Shader, ShaderBinding, ShaderBindingType, ShaderStage, Texture, TextureFormat, VertexFormat,
    VertexFormatItem, VertexItemType, WindowRenderTarget,
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

fn main() {
    pretty_env_logger::init();
    let event_loop = EventLoop::new();

    let mut builder = winit::window::WindowBuilder::new();
    builder = builder.with_title("test").with_inner_size(LogicalSize::new(1920, 1080));
    let window = Arc::new(builder.build(&event_loop).unwrap());
    let size = window.inner_size();

    let window1 = window.clone();
    task::spawn(async move {
        let mut renderer = Renderer::new().await;
        let mut render_target = WindowRenderTarget::new(&renderer, &*window1, size.width, size.height);

        let (vertex_data, index_data) = create_vertices();
        let vertex_format = VertexFormat::new(vec![
            VertexFormatItem::new("Position", VertexItemType::Float4, 0),
            VertexFormatItem::new("TexCoord", VertexItemType::Float2, 16),
        ]);
        let vertex_data = [vertex_data.as_bytes()];
        let mesh = Mesh::new(&renderer, &vertex_data, &[24], index_data.as_bytes(), vec![vertex_format]);

        let texture_data = create_texels(512, 512);
        let texture = Texture::with_texels(&renderer, 512, 512, &texture_data, TextureFormat::Rgba8Unorm);

        let shader = Shader::new(
            &renderer,
            include_str!("shader.wgsl"),
            "vs_main",
            "fs_main",
            hashmap! {
                "Mvp" => ShaderBinding::new(ShaderStage::Vertex, 0, ShaderBindingType::UniformBuffer),
                "Texture" => ShaderBinding::new(ShaderStage::Fragment, 1, ShaderBindingType::Texture2D),
                "Sampler" => ShaderBinding::new(ShaderStage::Fragment, 2, ShaderBindingType::Sampler),
            },
            hashmap! {
                "Position" => 0,
                "TexCoord" => 1,
            },
        );

        let material = Material::new(&renderer, hashmap! {"Texture" => Arc::new(texture)}, HashMap::new(), Arc::new(shader));
        let model = Model::new(&renderer, mesh, material, vec![0..index_data.len() as u32]);

        let camera = Camera::new(Point3::new(5.0, 5.0, 5.0), Point3::new(0.0, 0.0, 0.0));
        let mut scene = Scene::new(camera);
        scene.add(model);

        loop {
            renderer.render(&scene, &mut render_target);
            task::sleep(Duration::from_millis(16)).await;
        }
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            event::Event::MainEventsCleared => window.request_redraw(),
            event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::Escape),
                            state: event::ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            _ => {}
        }
    });
}

#[repr(C)]
#[derive(Clone, Copy, AsBytes)]
struct Vertex {
    pos: [f32; 4],
    tex_coord: [f32; 2],
}

impl Vertex {
    pub fn new(pos: [i8; 3], tex_coord: [i8; 2]) -> Self {
        Self {
            pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
            tex_coord: [tex_coord[0] as f32, tex_coord[1] as f32],
        }
    }
}

// Copied from https://github.com/gfx-rs/wgpu-rs/blob/master/examples/cube/main.rs#L23
fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
    let vertex_data = [
        // top (0, 0, 1)
        Vertex::new([-1, -1, 1], [0, 0]),
        Vertex::new([1, -1, 1], [1, 0]),
        Vertex::new([1, 1, 1], [1, 1]),
        Vertex::new([-1, 1, 1], [0, 1]),
        // bottom (0, 0, -1)
        Vertex::new([-1, 1, -1], [1, 0]),
        Vertex::new([1, 1, -1], [0, 0]),
        Vertex::new([1, -1, -1], [0, 1]),
        Vertex::new([-1, -1, -1], [1, 1]),
        // right (1, 0, 0)
        Vertex::new([1, -1, -1], [0, 0]),
        Vertex::new([1, 1, -1], [1, 0]),
        Vertex::new([1, 1, 1], [1, 1]),
        Vertex::new([1, -1, 1], [0, 1]),
        // left (-1, 0, 0)
        Vertex::new([-1, -1, 1], [1, 0]),
        Vertex::new([-1, 1, 1], [0, 0]),
        Vertex::new([-1, 1, -1], [0, 1]),
        Vertex::new([-1, -1, -1], [1, 1]),
        // front (0, 1, 0)
        Vertex::new([1, 1, -1], [1, 0]),
        Vertex::new([-1, 1, -1], [0, 0]),
        Vertex::new([-1, 1, 1], [0, 1]),
        Vertex::new([1, 1, 1], [1, 1]),
        // back (0, -1, 0)
        Vertex::new([1, -1, 1], [0, 0]),
        Vertex::new([-1, -1, 1], [1, 0]),
        Vertex::new([-1, -1, -1], [1, 1]),
        Vertex::new([1, -1, -1], [0, 1]),
    ];

    let index_data: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

fn create_texels(width: usize, height: usize) -> Vec<u8> {
    (0..width * height).flat_map(|_| vec![127, 127, 127, 255]).collect()
}
