use std::sync::Arc;

use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use eng::render::{Material, Mesh, Model, OrthographicCamera, Renderer, Scene, Shader, SimpleVertex};

struct App {
    renderer: Renderer,
    scene: Scene,
    camera: OrthographicCamera,
}

impl App {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let renderer = Renderer::new(window, size.width, size.height).await;

        let (vertices, indices) = create_vertices();
        let mesh = Mesh::with_simple_vertex(&renderer, &vertices, &indices);

        let shader = Shader::new(&renderer, include_str!("shader.wgsl"));

        let material = Material::new(&renderer, &[], &[], Arc::new(shader));
        let model = Model::new(&renderer, mesh, material);

        let camera = OrthographicCamera::new(size.width, size.height);
        let mut scene = Scene::new();
        scene.add(model);

        Self { renderer, scene, camera }
    }

    pub fn render(&mut self) {
        self.renderer.render(&self.camera, &self.scene);
    }
}

#[async_std::main]
async fn main() {
    pretty_env_logger::init();
    let event_loop = EventLoop::new();

    let mut builder = winit::window::WindowBuilder::new();
    builder = builder.with_title("test").with_inner_size(LogicalSize::new(1920, 1080));
    let window = Arc::new(builder.build(&event_loop).unwrap());

    let mut app = App::new(&window).await;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => {
                app.render();
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
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

fn create_vertices() -> (Vec<SimpleVertex>, Vec<u16>) {
    let vertices = vec![
        SimpleVertex::new([0.0, 0.0, 0.0, 1.0], [0.0, 0.0]),
        SimpleVertex::new([0.0, 500.0, 0.0, 1.0], [1.0, 0.0]),
        SimpleVertex::new([500.0, 0.0, 0.0, 1.0], [0.0, 1.0]),
    ];

    let indices = vec![0, 1, 2];

    (vertices, indices)
}
