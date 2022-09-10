use super::{Component, Entity, World};

pub trait Query {
    fn matches(world: &World, entity: Entity) -> bool;
}

impl<T1: 'static + Component> Query for (T1,) {
    fn matches(world: &World, entity: Entity) -> bool {
        world.has_component::<T1>(entity)
    }
}

impl<T1: 'static + Component, T2: 'static + Component> Query for (T1, T2) {
    fn matches(world: &World, entity: Entity) -> bool {
        world.has_component::<T1>(entity) && world.has_component::<T2>(entity)
    }
}
