use alloc::{boxed::Box, vec::Vec};

use crate::{camera::Camera, Renderable};

pub struct Scene {
    pub camera: Camera,
    pub models: Vec<Box<dyn Renderable>>,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Self { camera, models: Vec::new() }
    }

    pub fn add<F: Renderable + 'static>(&mut self, model: F) {
        self.models.push(Box::new(model));
    }
}
