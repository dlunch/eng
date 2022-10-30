use alloc::{boxed::Box, vec, vec::Vec};
use core::ops::Range;

use super::{transform::Transform, Camera, Material, Mesh, Renderer, SimpleVertex, Texture, TextureFormat};
use crate::ecs::{Component, ComponentBundle};

pub struct RenderComponent {
    pub mesh: Mesh,
    pub material: Material,
    pub ranges: Vec<Range<u32>>,
}

impl Component for RenderComponent {}

pub struct TransformComponent {
    pub transform: Transform,
}

impl Component for TransformComponent {}

pub struct CameraComponent {
    pub camera: Box<dyn Camera>,
}

impl Component for CameraComponent {}

pub struct RenderBundle {
    pub mesh: Mesh,
    pub material: Material,
    pub ranges: Option<Vec<Range<u32>>>,
    pub transform: Transform,
}

impl ComponentBundle for RenderBundle {
    fn add_components(self, world: &mut crate::ecs::World, entity: crate::ecs::Entity) {
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
}

pub struct SpriteBundle {
    pub image_data: Vec<u8>,
    pub image_width: u32,
    pub image_height: u32,
    pub transform: Transform,
}

impl ComponentBundle for SpriteBundle {
    fn add_components(self, world: &mut crate::ecs::World, entity: crate::ecs::Entity) {
        let vertices = vec![
            SimpleVertex::new([0.0, 0.0, 0.0, 1.0], [0.0, 0.0]),
            SimpleVertex::new([0.0, 1.0, 0.0, 1.0], [0.0, 1.0]),
            SimpleVertex::new([1.0, 0.0, 0.0, 1.0], [1.0, 0.0]),
            SimpleVertex::new([1.0, 1.0, 0.0, 1.0], [1.0, 1.0]),
        ];

        let indices = vec![0, 1, 2, 2, 1, 3];

        let renderer = world.resource::<Renderer>().unwrap();
        let texture = Texture::with_texels(renderer, self.image_width, self.image_height, &self.image_data, TextureFormat::Rgba8Unorm);

        let mesh = Mesh::with_simple_vertex(renderer, &vertices, &indices);
        let material = Material::new(renderer, texture);

        let bundle = RenderBundle {
            mesh,
            material,
            transform: self.transform,
            ranges: None,
        };

        world.add_bundle(entity, bundle);
    }
}
