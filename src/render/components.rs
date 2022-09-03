use alloc::{boxed::Box, vec::Vec};
use core::ops::Range;

use super::{transform::Transform, Camera, Material, Mesh};
use crate::ecs::Component;

pub struct RenderComponent {
    pub mesh: Mesh,
    pub material: Material,
    pub transform: Transform,
    pub ranges: Vec<Range<u32>>,
}

impl Component for RenderComponent {}

impl RenderComponent {
    pub fn new(mesh: Mesh, material: Material, ranges: &[Range<u32>], transform: Transform) -> Self {
        Self {
            mesh,
            material,
            transform,
            ranges: ranges.to_vec(),
        }
    }
}

pub struct CameraComponent {
    pub camera: Box<dyn Camera>,
}

impl Component for CameraComponent {}
