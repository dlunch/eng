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
}

impl App {
    pub async fn new() -> Self {
        let event_loop = EventLoop::new();

        let mut builder = winit::window::WindowBuilder::new();
        builder = builder.with_title("test").with_inner_size(LogicalSize::new(1920, 1080));
        let window = builder.build(&event_loop).unwrap();

        let renderer = render::Renderer::new(&window, 1920, 1080).await;

        let mut world = ecs::World::new();
        world.add_resource(renderer);

        Self { event_loop, window, world }
    }

    pub fn setup<F>(mut self, setup: F) -> Self
    where
        F: FnOnce(&mut ecs::World),
    {
        setup(&mut self.world);
        self
    }

    pub fn run(mut self) {
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            let mut renderer = self.world.take_resource::<render::Renderer>().unwrap();

            match event {
                Event::MainEventsCleared => self.window.request_redraw(),
                Event::RedrawRequested(_) => {
                    renderer.render_world(&self.world);
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

            self.world.add_resource(renderer);
        });
    }
}
