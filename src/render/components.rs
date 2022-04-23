use super::Model;
use crate::ecs::Component;

pub struct RenderComponent {
    pub model: Model,
}

impl Component for RenderComponent {}
