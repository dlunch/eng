use alloc::vec;

use glam::Vec3;

use super::{
    ecs::{Component, ComponentBundle},
    render::{AssetLoader, Material, Mesh, RenderBundle, Renderer, SimpleVertex, TextureAsset, Transform},
};

pub struct UiComponent {}
impl Component for UiComponent {}

pub struct UiNode {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl ComponentBundle for UiNode {
    fn add_components(self, world: &mut crate::ecs::World, entity: crate::ecs::Entity) {
        let vertices = vec![
            SimpleVertex::new([0.0, 0.0, 0.0, 1.0], [0.0, 0.0]),
            SimpleVertex::new([0.0, 1.0, 0.0, 1.0], [0.0, 1.0]),
            SimpleVertex::new([1.0, 0.0, 0.0, 1.0], [1.0, 0.0]),
            SimpleVertex::new([1.0, 1.0, 0.0, 1.0], [1.0, 1.0]),
        ];

        let indices = vec![0, 1, 2, 2, 1, 3];

        let bundle = {
            let renderer = world.resource::<Renderer>().unwrap();

            let mesh = Mesh::with_simple_vertex(&renderer, &vertices, &indices);
            let material = Material::new(&renderer, &renderer.empty_texture);

            let transform = Transform::with_values(
                Vec3::new(self.x as f32, self.y as f32, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(self.width as f32, self.height as f32, 0.0),
            );

            RenderBundle {
                mesh,
                material,
                transform,
                ranges: None,
            }
        };

        world.add_bundle(entity, bundle);
        world.add_component(entity, UiComponent {});
    }
}

pub struct UiSprite {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub texture_asset: TextureAsset,
}

impl ComponentBundle for UiSprite {
    fn add_components(self, world: &mut crate::ecs::World, entity: crate::ecs::Entity) {
        let vertices = vec![
            SimpleVertex::new([0.0, 0.0, 0.0, 1.0], [0.0, 0.0]),
            SimpleVertex::new([0.0, 1.0, 0.0, 1.0], [0.0, 1.0]),
            SimpleVertex::new([1.0, 0.0, 0.0, 1.0], [1.0, 0.0]),
            SimpleVertex::new([1.0, 1.0, 0.0, 1.0], [1.0, 1.0]),
        ];

        let indices = vec![0, 1, 2, 2, 1, 3];

        let bundle = {
            let renderer = world.resource::<Renderer>().unwrap();
            let mut asset_loader = world.resource_mut::<AssetLoader>().unwrap();

            let mesh = Mesh::with_simple_vertex(&renderer, &vertices, &indices);
            let material = Material::new(&renderer, asset_loader.texture(&renderer, self.texture_asset).unwrap());

            let transform = Transform::with_values(
                Vec3::new(self.x as f32, self.y as f32, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(self.width as f32, self.height as f32, 0.0),
            );

            RenderBundle {
                mesh,
                material,
                transform,
                ranges: None,
            }
        };

        world.add_bundle(entity, bundle);
        world.add_component(entity, UiComponent {});
    }
}
