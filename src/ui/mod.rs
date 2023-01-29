use alloc::vec;

use glam::Vec3;

use super::{
    ecs::{Component, ComponentBundle, ComponentContainer, Entity, World},
    render::{AssetLoader, Material, Mesh, RenderBundle, Renderer, SimpleVertex, TextureAsset, Transform},
};

pub struct UiComponent {}
impl Component for UiComponent {}

pub struct UiNode {
    render_bundle: RenderBundle,
}

impl UiNode {
    pub fn new(world: &World, x: u32, y: u32, width: u32, height: u32) -> Self {
        let vertices = vec![
            SimpleVertex::new([0.0, 0.0, 0.0, 1.0], [0.0, 0.0]),
            SimpleVertex::new([0.0, 1.0, 0.0, 1.0], [0.0, 1.0]),
            SimpleVertex::new([1.0, 0.0, 0.0, 1.0], [1.0, 0.0]),
            SimpleVertex::new([1.0, 1.0, 0.0, 1.0], [1.0, 1.0]),
        ];

        let indices = vec![0, 1, 2, 2, 1, 3];

        let bundle = {
            let renderer = world.resource::<Renderer>().unwrap();

            let mesh = Mesh::with_simple_vertex(renderer, &vertices, &indices);
            let material = Material::new(renderer, &renderer.empty_texture);

            let transform = Transform::with_values(
                Vec3::new(x as f32, y as f32, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(width as f32, height as f32, 0.0),
            );

            RenderBundle {
                mesh,
                material,
                transform,
                ranges: None,
            }
        };

        Self { render_bundle: bundle }
    }
}

impl ComponentBundle for UiNode {
    fn add_components(self, world: &mut World, entity: Entity) {
        world.add_bundle(entity, self.render_bundle);
        world.add_component(entity, UiComponent {});
    }

    fn to_component_containers(self) -> Vec<ComponentContainer> {
        self.render_bundle
            .to_component_containers()
            .into_iter()
            .chain(vec![ComponentContainer::new(UiComponent {})])
            .collect()
    }
}

pub struct UiSprite {
    render_bundle: RenderBundle,
}
impl UiSprite {
    pub fn new(world: &World, x: u32, y: u32, width: u32, height: u32, texture_asset: TextureAsset) -> Self {
        let vertices = vec![
            SimpleVertex::new([0.0, 0.0, 0.0, 1.0], [0.0, 0.0]),
            SimpleVertex::new([0.0, 1.0, 0.0, 1.0], [0.0, 1.0]),
            SimpleVertex::new([1.0, 0.0, 0.0, 1.0], [1.0, 0.0]),
            SimpleVertex::new([1.0, 1.0, 0.0, 1.0], [1.0, 1.0]),
        ];

        let indices = vec![0, 1, 2, 2, 1, 3];

        let bundle = {
            let renderer = world.resource::<Renderer>().unwrap();
            let asset_loader = world.resource::<AssetLoader>().unwrap();

            let mesh = Mesh::with_simple_vertex(renderer, &vertices, &indices);
            let material = Material::new(renderer, &asset_loader.texture(renderer, texture_asset).unwrap());

            let transform = Transform::with_values(
                Vec3::new(x as f32, y as f32, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(width as f32, height as f32, 0.0),
            );

            RenderBundle {
                mesh,
                material,
                transform,
                ranges: None,
            }
        };

        Self { render_bundle: bundle }
    }
}

impl ComponentBundle for UiSprite {
    fn add_components(self, world: &mut World, entity: Entity) {
        world.add_bundle(entity, self.render_bundle);
        world.add_component(entity, UiComponent {});
    }

    fn to_component_containers(self) -> Vec<ComponentContainer> {
        self.render_bundle
            .to_component_containers()
            .into_iter()
            .chain(vec![ComponentContainer::new(UiComponent {})])
            .collect()
    }
}
