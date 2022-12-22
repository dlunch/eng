use super::{component::ComponentContainer, Entity, World};

use super::Component;

pub trait ComponentBundle {
    fn add_components(self, world: &mut World, entity: Entity);
    fn to_component_containers(self, world: &mut World) -> Vec<ComponentContainer>; // TODO we have to remove world from here
}

impl<T1> ComponentBundle for (T1,)
where
    T1: 'static + Component,
{
    fn add_components(self, world: &mut World, entity: Entity) {
        world.add_component(entity, self.0);
    }

    fn to_component_containers(self, _: &mut World) -> Vec<ComponentContainer> {
        vec![ComponentContainer::new(self.0)]
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

    fn to_component_containers(self, _: &mut World) -> Vec<ComponentContainer> {
        vec![ComponentContainer::new(self.0), ComponentContainer::new(self.1)]
    }
}
