use alloc::{sync::Arc, vec, vec::Vec};

use glam::Vec3;

use super::{
    ecs::{Component, ComponentBundle},
    render::{Material, Mesh, RenderBundle, Renderer, Shader, SimpleVertex, Texture, TextureFormat, Transform},
};

pub struct UiComponent {}
impl Component for UiComponent {}

pub struct Rectangle {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl ComponentBundle for Rectangle {
    fn add_components(self, world: &mut crate::ecs::World, entity: crate::ecs::Entity) {
        let vertices = vec![
            SimpleVertex::new([0.0, 0.0, 0.0, 1.0], [0.0, 0.0]),
            SimpleVertex::new([0.0, 1.0, 0.0, 1.0], [0.0, 1.0]),
            SimpleVertex::new([1.0, 0.0, 0.0, 1.0], [1.0, 0.0]),
            SimpleVertex::new([1.0, 1.0, 0.0, 1.0], [1.0, 1.0]),
        ];

        let indices = vec![0, 1, 2, 2, 1, 3];

        let renderer = world.resource::<Renderer>().unwrap();
        let empty_texture = Texture::with_texels(renderer, 1, 1, &[0, 0, 0, 0], TextureFormat::Rgba8Unorm); // TODO remove

        let mesh = Mesh::with_simple_vertex(renderer, &vertices, &indices);
        let shader = Shader::new(renderer, include_str!("ui.wgsl"));
        let material = Material::with_custom_shader(renderer, &[("texture", Arc::new(empty_texture))], Arc::new(shader));

        let transform = Transform::with_values(
            Vec3::new(self.x as f32, self.y as f32, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(self.width as f32, self.height as f32, 0.0),
        );

        let bundle = RenderBundle {
            mesh,
            material,
            transform,
            ranges: None,
        };

        world.add_bundle(entity, bundle);
        world.add_component(entity, UiComponent {});
    }
}

pub struct Sprite {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub image_data: Vec<u8>, // TODO asset storage
    pub image_width: u32,
    pub image_height: u32,
}

impl ComponentBundle for Sprite {
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
        let shader = Shader::new(renderer, include_str!("ui.wgsl"));
        let material = Material::with_custom_shader(renderer, &[("texture", Arc::new(texture))], Arc::new(shader));

        let transform = Transform::with_values(
            Vec3::new(self.x as f32, self.y as f32, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(self.width as f32, self.height as f32, 0.0),
        );

        let bundle = RenderBundle {
            mesh,
            material,
            transform,
            ranges: None,
        };

        world.add_bundle(entity, bundle);
        world.add_component(entity, UiComponent {});
    }
}
