#![no_std]
extern crate alloc;

use core::future::Future;

use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub mod ecs;
pub mod render;
pub mod ui;
mod utils;

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

        let window_size = window.inner_size();
        let renderer = render::Renderer::new(&window, window_size.width, window_size.height).await;

        let mut world = ecs::World::new();
        world.add_resource(renderer);

        Self { event_loop, window, world }
    }

    pub async fn setup<'a, F, Fut>(mut self, setup: F) -> Self
    where
        F: FnOnce(&'a mut ecs::World) -> Fut,
        Fut: Future<Output = ()>,
    {
        // why do we need to remove lifetime here?
        let world = unsafe { core::mem::transmute(&mut self.world) };
        setup(world).await;

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
