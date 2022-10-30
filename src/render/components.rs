use alloc::{boxed::Box, vec::Vec};
use core::ops::Range;

use super::{transform::Transform, Camera, Material, Mesh};
use crate::ecs::Component;

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
