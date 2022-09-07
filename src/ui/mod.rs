use alloc::{sync::Arc, vec};

use glam::Vec3;

use super::{
    ecs::ComponentBundle,
    render::{Material, Mesh, RenderComponent, Renderer, Shader, SimpleVertex, Transform},
};

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
            SimpleVertex::new([0.0, 1.0, 0.0, 1.0], [1.0, 0.0]),
            SimpleVertex::new([1.0, 0.0, 0.0, 1.0], [0.0, 1.0]),
            SimpleVertex::new([1.0, 1.0, 0.0, 1.0], [1.0, 1.0]),
        ];

        let indices = vec![0, 1, 2, 2, 1, 3];

        let renderer = world.resource::<Renderer>().unwrap();

        let mesh = Mesh::with_simple_vertex(renderer, &vertices, &indices);
        let shader = Shader::new(renderer, include_str!("ui.wgsl"));
        let material = Material::with_custom_shader(renderer, &[], Arc::new(shader));

        let transform = Transform::with_values(
            Vec3::new(self.x as f32, self.y as f32, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(self.width as f32, self.height as f32, 0.0),
        );

        let component = RenderComponent::new(mesh, material, transform);

        world.add_component(entity, component)
    }
}