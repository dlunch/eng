use crate::ecs::Component;

use super::{Material, Mesh};

pub struct RenderComponent {
    pub mesh: Mesh,
    pub material: Material,
}

impl Component for RenderComponent {}
