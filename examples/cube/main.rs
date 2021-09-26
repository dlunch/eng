use std::sync::Arc;
use std::time::Duration;

use async_std::task;
use nalgebra::Point3;
use winit::{
    dpi::LogicalSize,
    event,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
};

use renderer::{
    ArcballCamera, Material, Mesh, Model, Renderer, Scene, Shader, ShaderBinding, ShaderBindingType, ShaderStage, SimpleVertex, Texture,
    TextureFormat,
};

fn main() {
    pretty_env_logger::init();
    let event_loop = EventLoop::new();

    let mut builder = winit::window::WindowBuilder::new();
    builder = builder.with_title("test").with_inner_size(LogicalSize::new(1920, 1080));
    let window = Arc::new(builder.build(&event_loop).unwrap());
    let size = window.inner_size();

    let window1 = window.clone();
    task::spawn(async move {
        let mut renderer = Renderer::new(&*window1, size.width, size.height).await;

        let (vertices, indices) = create_vertices();
        let mesh = Mesh::with_simple_vertex(&renderer, &vertices, &indices);

        let texture_data = create_texels(512, 512);
        let texture = Texture::with_texels(&renderer, 512, 512, &texture_data, TextureFormat::Rgba8Unorm);

        let shader = Shader::new(
            &renderer,
            include_str!("shader.wgsl"),
            "vs_main",
            "fs_main",
            &[
                ("Mvp", ShaderBinding::new(ShaderStage::Vertex, 0, ShaderBindingType::UniformBuffer)),
                ("Texture", ShaderBinding::new(ShaderStage::Fragment, 1, ShaderBindingType::Texture2D)),
                ("Sampler", ShaderBinding::new(ShaderStage::Fragment, 2, ShaderBindingType::Sampler)),
            ],
            &[("Position", 0), ("TexCoord", 1)],
        );

        let material = Material::new(&renderer, &[("Texture", Arc::new(texture))], &[], Arc::new(shader));
        let model = Model::new(&renderer, mesh, material);

        let camera = ArcballCamera::new(Point3::new(0.0, 0.0, 0.0), 5.0);
        let mut scene = Scene::new(camera);
        scene.add(model);

        loop {
            renderer.render(&scene);
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

// Copied from https://github.com/gfx-rs/wgpu-rs/blob/master/examples/cube/main.rs#L23
fn create_vertices() -> (Vec<SimpleVertex>, Vec<u16>) {
    let vertices = vec![
        // top (0, 0, 1)
        SimpleVertex::new([-1.0, -1.0, 1.0, 1.0], [0.0, 0.0]),
        SimpleVertex::new([1.0, -1.0, 1.0, 1.0], [1.0, 0.0]),
        SimpleVertex::new([1.0, 1.0, 1.0, 1.0], [1.0, 1.0]),
        SimpleVertex::new([-1.0, 1.0, 1.0, 1.0], [0.0, 1.0]),
        // bottom (0, 0, -1)
        SimpleVertex::new([-1.0, 1.0, -1.0, 1.0], [1.0, 0.0]),
        SimpleVertex::new([1.0, 1.0, -1.0, 1.0], [0.0, 0.0]),
        SimpleVertex::new([1.0, -1.0, -1.0, 1.0], [0.0, 1.0]),
        SimpleVertex::new([-1.0, -1.0, -1.0, 1.0], [1.0, 1.0]),
        // right (1, 0, 0)
        SimpleVertex::new([1.0, -1.0, -1.0, 1.0], [0.0, 0.0]),
        SimpleVertex::new([1.0, 1.0, -1.0, 1.0], [1.0, 0.0]),
        SimpleVertex::new([1.0, 1.0, 1.0, 1.0], [1.0, 1.0]),
        SimpleVertex::new([1.0, -1.0, 1.0, 1.0], [0.0, 1.0]),
        // left (-1, 0, 0)
        SimpleVertex::new([-1.0, -1.0, 1.0, 1.0], [1.0, 0.0]),
        SimpleVertex::new([-1.0, 1.0, 1.0, 1.0], [0.0, 0.0]),
        SimpleVertex::new([-1.0, 1.0, -1.0, 1.0], [0.0, 1.0]),
        SimpleVertex::new([-1.0, -1.0, -1.0, 1.0], [1.0, 1.0]),
        // front (0, 1, 0)
        SimpleVertex::new([1.0, 1.0, -1.0, 1.0], [1.0, 0.0]),
        SimpleVertex::new([-1.0, 1.0, -1.0, 1.0], [0.0, 0.0]),
        SimpleVertex::new([-1.0, 1.0, 1.0, 1.0], [0.0, 1.0]),
        SimpleVertex::new([1.0, 1.0, 1.0, 1.0], [1.0, 1.0]),
        // back (0, -1, 0)
        SimpleVertex::new([1.0, -1.0, 1.0, 1.0], [0.0, 0.0]),
        SimpleVertex::new([-1.0, -1.0, 1.0, 1.0], [1.0, 0.0]),
        SimpleVertex::new([-1.0, -1.0, -1.0, 1.0], [1.0, 1.0]),
        SimpleVertex::new([1.0, -1.0, -1.0, 1.0], [0.0, 1.0]),
    ];

    let indices = vec![
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    (vertices, indices)
}

fn create_texels(width: usize, height: usize) -> Vec<u8> {
    (0..width * height).flat_map(|_| vec![127, 127, 127, 255]).collect()
}
