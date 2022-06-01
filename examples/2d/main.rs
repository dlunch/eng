use std::sync::Arc;

use winit::dpi::LogicalSize;

use eng::ecs::World;
use eng::render::{Material, Mesh, OrthographicCamera, RenderComponent, Renderer, Shader, SimpleVertex};
use eng::App;

fn setup(world: &mut World) {
    let render_component = {
        let renderer = world.resource::<Renderer>().unwrap();

        let (vertices, indices) = create_vertices();
        let mesh = Mesh::with_simple_vertex(renderer, &vertices, &indices);

        let shader = Shader::new(renderer, include_str!("shader.wgsl"));

        let material = Material::new(renderer, &[], &[], Arc::new(shader));
        RenderComponent::new(renderer, mesh, material)
    };

    let entity = world.spawn();
    world.add_component(entity, render_component);
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let size = LogicalSize::new(1920, 1080);

    let camera = OrthographicCamera::new(size.width, size.height);

    App::new().await.setup(setup).run(camera)
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
