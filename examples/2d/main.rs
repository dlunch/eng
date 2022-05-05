use std::sync::Arc;

use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use eng::ecs::World;
use eng::render::{Material, Mesh, OrthographicCamera, RenderComponent, Renderer, Shader, SimpleVertex};

struct App {
    renderer: Renderer,
    world: World,
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

        let camera = OrthographicCamera::new(size.width, size.height);

        let mut world = World::new();

        let entity = world.spawn();
        world.add_component(entity, RenderComponent::new(&renderer, mesh, material));

        Self { renderer, world, camera }
    }

    pub fn render(&mut self) {
        self.renderer.render_world(&self.camera, &self.world);
    }
}

#[tokio::main]
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
