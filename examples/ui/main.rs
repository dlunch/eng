use image::{io::Reader as ImageReader, EncodableLayout};
use std::io::Cursor;

use eng::ecs::World;
use eng::render::{CameraComponent, OrthographicCamera};
use eng::ui::{Rectangle, Sprite};
use eng::App;

async fn setup(world: &mut World) {
    let img = ImageReader::new(Cursor::new(include_bytes!("./image.png")))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();

    let img = img.into_rgba8();

    world.spawn_bundle(Sprite {
        x: 500,
        y: 500,
        width: 500,
        height: 500,
        image_data: img.as_bytes().to_vec(),
        image_width: img.width(),
        image_height: img.height(),
    });

    world.spawn_bundle(Rectangle {
        x: 0,
        y: 0,
        width: 500,
        height: 500,
    });

    let camera = OrthographicCamera::new();
    world.spawn().with(CameraComponent { camera: Box::new(camera) });
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    App::new().await.setup(setup).await.run()
}
