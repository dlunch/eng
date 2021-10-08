use alloc::{boxed::Box, vec::Vec};

use crate::Renderable;

pub struct Scene {
    pub models: Vec<Box<dyn Renderable>>,
}

impl Scene {
    pub fn new() -> Self {
        Self { models: Vec::new() }
    }

    pub fn add<F: Renderable + 'static>(&mut self, model: F) {
        self.models.push(Box::new(model));
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
