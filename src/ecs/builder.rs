use super::{Component, Entity, World};

pub struct EntityBuilder<'a> {
    world: &'a mut World,
    entity: Entity,
}

impl<'a> EntityBuilder<'a> {
    pub fn new(world: &'a mut World, entity: Entity) -> Self {
        Self { world, entity }
    }

    pub fn with<T: 'static + Component>(self, component: T) -> Self {
        self.world.add_component(self.entity, component);

        self
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }
}
