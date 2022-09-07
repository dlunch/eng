use eng::ecs::World;
use eng::render::{CameraComponent, OrthographicCamera};
use eng::ui::Rectangle;
use eng::App;

async fn setup(world: &mut World) {
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
