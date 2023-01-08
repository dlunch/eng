use alloc::vec::Vec;
use core::future::Future;

use windowing::Window;

use super::{ecs, render};

type UpdateFn = fn(&mut ecs::World) -> ();

pub struct App {
    window: Window,
    world: ecs::World,
    update_fn: Vec<UpdateFn>,
}

impl App {
    pub async fn new() -> Self {
        let window = Window::new(1920, 1080, "test");

        let renderer = render::Renderer::new(&window, 1920, 1080).await;
        let asset_loader = render::AssetLoader::new();

        let mut world = ecs::World::new();
        world.add_resource(renderer);
        world.add_resource(asset_loader);

        Self {
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

    pub async fn run(mut self) {
        loop {
            let events = self.window.next_events(false).await;
            for _ in events {
                // TODO
            }

            self.world.update().await;
            for update in self.update_fn.iter() {
                update(&mut self.world);
            }

            let mut renderer = self.world.resource_mut::<render::Renderer>().unwrap();
            renderer.render_world(&self.world);
        }
    }
}
