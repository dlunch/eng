use alloc::{boxed::Box, vec::Vec};

use crate::{camera::Camera, Renderable};

pub struct Scene {
    pub camera: Box<dyn Camera>,
    pub models: Vec<Box<dyn Renderable>>,
}

impl Scene {
    pub fn new<T: 'static + Camera>(camera: T) -> Self {
        Self {
            camera: Box::new(camera),
            models: Vec::new(),
        }
    }

    pub fn add<F: Renderable + 'static>(&mut self, model: F) {
        self.models.push(Box::new(model));
    }

    pub fn camera_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.camera.as_mut_any().downcast_mut()
    }
}
