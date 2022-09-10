use alloc::{boxed::Box, vec, vec::Vec};
use core::ops::Range;

use super::{transform::Transform, Camera, Material, Mesh};
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
