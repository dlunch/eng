#![no_std]
extern crate alloc;

use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub mod ecs;
pub mod render;

pub struct App {
    event_loop: EventLoop<()>,
    window: Window,
    world: ecs::World,
    renderer: render::Renderer,
}

impl App {
    pub async fn new() -> Self {
        let event_loop = EventLoop::new();

        let mut builder = winit::window::WindowBuilder::new();
        builder = builder.with_title("test").with_inner_size(LogicalSize::new(1920, 1080));
        let window = builder.build(&event_loop).unwrap();

        let renderer = render::Renderer::new(&window, 1920, 1080).await;

        Self {
            event_loop,
            window,
            world: ecs::World::new(),
            renderer,
        }
    }

    pub fn setup<F>(mut self, setup: F) -> Self
    where
        F: FnOnce(&mut ecs::World, &render::Renderer),
    {
        setup(&mut self.world, &self.renderer);
        self
    }

    pub fn run<T>(mut self, camera: T)
    where
        T: render::Camera + 'static,
    {
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared => self.window.request_redraw(),
                Event::RedrawRequested(_) => {
                    self.renderer.render_world(&camera, &self.world);
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
}
