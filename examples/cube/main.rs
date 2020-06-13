use std::time::Duration;

use async_std::task;
use nalgebra::Point3;
use winit::{
    dpi::LogicalSize,
    event,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
};

use renderer::{Camera, Renderer, Scene, WindowRenderTarget};

fn main() {
    let event_loop = EventLoop::new();

    let mut builder = winit::window::WindowBuilder::new();
    builder = builder.with_title("test").with_inner_size(LogicalSize::new(1920, 1080));
    let window = builder.build(&event_loop).unwrap();

    let size = window.inner_size();
    let mut renderer = task::block_on(async { Renderer::new().await });
    let mut render_target = WindowRenderTarget::new(&renderer, &window, size.width, size.height);

    let camera = Camera::new(Point3::new(0.0, 0.8, 2.5), Point3::new(0.0, 0.8, 0.0));
    let scene = Scene::new(camera);

    task::spawn(async move {
        loop {
            renderer.render(&scene, &mut render_target).await;
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
