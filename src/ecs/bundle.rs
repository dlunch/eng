use super::{Entity, World};

use super::Component;

pub trait ComponentBundle {
    fn add_components(self, world: &mut World, entity: Entity);
}

impl<T1> ComponentBundle for (T1,)
where
    T1: 'static + Component,
{
    fn add_components(self, world: &mut World, entity: Entity) {
        world.add_component(entity, self.0);
    }
}

impl<T1, T2> ComponentBundle for (T1, T2)
where
    T1: 'static + Component,
    T2: 'static + Component,
{
    fn add_components(self, world: &mut World, entity: Entity) {
        world.add_component(entity, self.0);
        world.add_component(entity, self.1);
    }
}
