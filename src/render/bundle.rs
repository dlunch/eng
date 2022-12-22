use alloc::{vec, vec::Vec};
use core::ops::Range;

use super::{
    asset::{AssetLoader, TextureAsset},
    components::TransformComponent,
    transform::Transform,
    Material, Mesh, RenderComponent, Renderer, SimpleVertex,
};
use crate::ecs::{ComponentBundle, ComponentContainer, Entity, World};

pub struct RenderBundle {
    pub mesh: Mesh,
    pub material: Material,
    pub ranges: Option<Vec<Range<u32>>>,
    pub transform: Transform,
}

impl ComponentBundle for RenderBundle {
    fn add_components(self, world: &mut World, entity: Entity) {
        let index_count = self.mesh.index_count;

        world.add_component(
            entity,
            RenderComponent {
                mesh: self.mesh,
                material: self.material,
                ranges: self.ranges.unwrap_or_else(|| vec![0..index_count as u32]),
            },
        );
        world.add_component(entity, TransformComponent { transform: self.transform });
    }

    fn to_component_containers(self, _: &mut World) -> Vec<ComponentContainer> {
        let index_count = self.mesh.index_count;

        vec![
            ComponentContainer::new(RenderComponent {
                mesh: self.mesh,
                material: self.material,
                ranges: self.ranges.unwrap_or_else(|| vec![0..index_count as u32]),
            }),
            ComponentContainer::new(TransformComponent { transform: self.transform }),
        ]
    }
}

pub struct SpriteBundle {
    pub texture_asset: TextureAsset,
    pub transform: Transform,
}

impl ComponentBundle for SpriteBundle {
    fn add_components(self, world: &mut World, entity: Entity) {
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
            let texture = asset_loader.texture(&renderer, self.texture_asset).unwrap();

            let mesh = Mesh::with_simple_vertex(&renderer, &vertices, &indices);
            let material = Material::new(&renderer, texture);

            RenderBundle {
                mesh,
                material,
                transform: self.transform,
                ranges: None,
            }
        };

        world.add_bundle(entity, bundle);
    }

    fn to_component_containers(self, world: &mut World) -> Vec<ComponentContainer> {
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
            let texture = asset_loader.texture(&renderer, self.texture_asset).unwrap();

            let mesh = Mesh::with_simple_vertex(&renderer, &vertices, &indices);
            let material = Material::new(&renderer, texture);

            RenderBundle {
                mesh,
                material,
                transform: self.transform,
                ranges: None,
            }
        };

        bundle.to_component_containers(world)
    }
}
