use std::f32::consts::PI;
use std::sync::Arc;

use nalgebra::Point3;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use eng::render::{ArcballCameraController, Material, Mesh, Model, PerspectiveCamera, Renderer, Scene, Shader, SimpleVertex, Texture, TextureFormat};

struct App {
    renderer: Renderer,
    scene: Scene,
    camera: PerspectiveCamera<ArcballCameraController>,
    size: (u32, u32),
    mouse_down: bool,
    mouse_down_pos: Option<(f32, f32)>,
}

impl App {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let renderer = Renderer::new(window, size.width, size.height).await;

        let (vertices, indices) = create_vertices();
        let mesh = Mesh::with_simple_vertex(&renderer, &vertices, &indices);

        let texture_data = create_texels(512, 512);
        let texture = Texture::with_texels(&renderer, 512, 512, &texture_data, TextureFormat::Rgba8Unorm);

        let shader = Shader::new(&renderer, include_str!("shader.wgsl"));

        let material = Material::new(&renderer, &[("texture", Arc::new(texture))], &[], Arc::new(shader));
        let model = Model::new(&renderer, mesh, material);

        let controller = ArcballCameraController::new(Point3::new(0.0, 0.0, 0.0), 5.0);
        let camera = PerspectiveCamera::new(45.0 * PI / 180.0, size.width as f32 / size.height as f32, 0.1, 100.0, controller);
        let mut scene = Scene::new();
        scene.add(model);

        Self {
            renderer,
            scene,
            camera,
            size: (size.width, size.height),
            mouse_down: false,
            mouse_down_pos: None,
        }
    }

    pub fn render(&mut self) {
        self.renderer.render(&self.camera, &self.scene);
    }

    pub fn mouse_down(&mut self) {
        self.mouse_down = true;
    }

    pub fn mouse_move(&mut self, x: f64, y: f64) {
        let last_pos = self.mouse_down_pos;
        self.mouse_down_pos = Some((x as f32, y as f32));

        if let Some(pos) = last_pos {
            if self.mouse_down {
                let (x0, y0) = pos;
                let (x1, y1) = (x as f32, y as f32);

                let controller = self.camera.controller_mut();
                controller.update((x1 - x0) / self.size.0 as f32, (y1 - y0) / self.size.1 as f32);
            }
        }
    }

    pub fn mouse_up(&mut self) {
        self.mouse_down = false;
        self.mouse_down_pos = None;
    }

    pub fn r#move(&mut self, forward: f32, right: f32) {
        let controller = self.camera.controller_mut();
        controller.r#move(forward, right);
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
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Left,
                    ..
                } => {
                    app.mouse_down();
                }
                WindowEvent::MouseInput {
                    state: ElementState::Released,
                    button: MouseButton::Left,
                    ..
                } => {
                    app.mouse_up();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    app.mouse_move(position.x, position.y);
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::W),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    app.r#move(0.1, 0.0);
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
