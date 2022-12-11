use alloc::vec::Vec;
use core::future::Future;

use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use super::{ecs, render};

type UpdateFn = fn(&mut ecs::World) -> ();

pub struct App {
    event_loop: EventLoop<()>,
    window: Window,
    world: ecs::World,
    update_fn: Vec<UpdateFn>,
}

impl App {
    pub async fn new() -> Self {
        let event_loop = EventLoop::new();

        let mut builder = winit::window::WindowBuilder::new();
        builder = builder.with_title("test").with_inner_size(LogicalSize::new(1920, 1080));
        let window = builder.build(&event_loop).unwrap();

        let window_size = window.inner_size();
        let renderer = render::Renderer::new(&window, window_size.width, window_size.height).await;
        let asset_loader = render::AssetLoader::new();

        let mut world = ecs::World::new();
        world.add_resource(renderer);
        world.add_resource(asset_loader);

        Self {
            event_loop,
            window,
            world,
            update_fn: Vec::new(),
        }
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

    pub fn update(mut self, update: UpdateFn) -> Self {
        self.update_fn.push(update);
        self
    }

    pub fn run(mut self) {
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            let mut renderer = self.world.take_resource::<render::Renderer>().unwrap();

            match event {
                Event::MainEventsCleared => self.window.request_redraw(),
                Event::RedrawRequested(_) => {
                    for update in self.update_fn.iter() {
                        update(&mut self.world);
                    }

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
