use std::f32::consts::PI;

use glam::Vec3;

use eng::ecs::World;
use eng::render::{
    ArcballCameraController, CameraComponent, Material, Mesh, PerspectiveCamera, RenderBundle, Renderer, SimpleVertex, Texture, TextureFormat,
    Transform,
};
use eng::App;

async fn setup(world: &mut World) {
    let render_bundle1 = {
        let renderer = world.resource::<Renderer>().unwrap();

        let (vertices, indices) = create_vertices();
        let mesh = Mesh::with_simple_vertex(&renderer, &vertices, &indices);

        let texture_data = create_texels(512, 512);
        let texture = Texture::with_texels(&renderer, 512, 512, &texture_data, TextureFormat::Rgba8Unorm);

        let material = Material::new(&renderer, texture);
        RenderBundle {
            mesh,
            material,
            transform: Transform::with_values(Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.7, 0.7), Vec3::new(1.0, 1.0, 1.0)),
            ranges: None,
        }
    };

    world.spawn_bundle(render_bundle1);

    let render_bundle2 = {
        let renderer = world.resource::<Renderer>().unwrap();

        let (vertices, indices) = create_vertices();
        let mesh = Mesh::with_simple_vertex(&renderer, &vertices, &indices);

        let texture_data = create_texels(512, 512);
        let texture = Texture::with_texels(&renderer, 512, 512, &texture_data, TextureFormat::Rgba8Unorm);

        let material = Material::new(&renderer, texture);
        RenderBundle {
            mesh,
            material,
            transform: Transform::with_values(Vec3::new(-1.0, 0.0, 0.0), Vec3::new(0.0, 0.1, 0.1), Vec3::new(1.0, 1.0, 1.0)),
            ranges: None,
        }
    };

    world.spawn_bundle(render_bundle2);

    let controller = ArcballCameraController::new(Vec3::new(0.0, 0.0, 0.0), 5.0);
    let camera = PerspectiveCamera::new(45.0 * PI / 180.0, 0.1, 100.0, controller);

    world.spawn().with(CameraComponent { camera: Box::new(camera) });
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    App::new().await.setup(setup).await.run()
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
