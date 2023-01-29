use windowing::Window;

use super::{ecs, render};

pub struct App {
    window: Window,
    world: ecs::World,
}

impl App {
    pub async fn new() -> Self {
        let window = Window::new(1920, 1080, "test").await;

        let renderer = render::Renderer::new(&window, 1920, 1080).await;
        let asset_loader = render::AssetLoader::new();

        let mut world = ecs::World::new();
        world.add_resource(renderer);
        world.add_resource(asset_loader);

        Self { window, world }
    }

    pub fn add_system<T, P>(mut self, system: T) -> Self
    where
        T: ecs::IntoSystem<P>,
    {
        self.world.add_system(system);

        self
    }

    pub async fn setup<F>(mut self, setup_fn: F) -> Self
    where
        F: for<'a> ecs::AsyncSingleArgFnOnce<&'a ecs::World, Output = ecs::CommandList>,
    {
        self.world.setup(setup_fn).await;

        self
    }

    pub async fn run(mut self) {
        loop {
            let events = self.window.next_events(false).await;
            for _ in events {
                // TODO
            }

            self.world.update().await;

            let mut renderer = self.world.take_resource::<render::Renderer>().unwrap();
            renderer.render_world(&self.world);

            self.world.add_resource(renderer);
        }
    }
}
