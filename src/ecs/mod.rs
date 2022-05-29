use alloc::collections::BTreeMap;
use core::any::{Any, TypeId};

mod hierarchy;
mod raw_vec;
mod sparse_raw_vec;

use sparse_raw_vec::SparseRawVec;

type ComponentType = TypeId;

pub struct World {
    components: BTreeMap<ComponentType, SparseRawVec<Entity>>,
    entities: u32,
}

impl World {
    pub fn new() -> Self {
        Self {
            components: BTreeMap::new(),
            entities: 0,
        }
    }

    pub fn spawn(&mut self) -> Entity {
        let id = self.entities;

        self.entities += 1;

        Entity { id }
    }

    pub fn add_component<T: 'static + Component>(&mut self, entity: Entity, component: T) {
        let component_type = TypeId::of::<T>();

        let vec = if let Some(x) = self.components.get_mut(&component_type) {
            x
        } else {
            let vec = SparseRawVec::new::<T>();
            self.components.insert(component_type, vec);

            self.components.get_mut(&component_type).unwrap()
        };

        vec.insert(entity, component);
    }

    pub fn component<T: 'static + Component>(&self, entity: Entity) -> Option<&T> {
        let component_type = TypeId::of::<T>();

        self.components.get(&component_type)?.get::<T>(entity)
    }

    pub fn component_mut<T: 'static + Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let component_type = TypeId::of::<T>();

        self.components.get_mut(&component_type)?.get_mut::<T>(entity)
    }

    pub fn components<T: 'static + Component>(&self) -> impl Iterator<Item = (Entity, &T)> {
        let component_type = TypeId::of::<T>();

        self.components.get(&component_type).unwrap().iter()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Entity {
    id: u32,
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub trait Component: AsAny {}

#[cfg(test)]
mod test {
    use super::{Component, World};

    #[test]
    fn test_entity() {
        let mut world = World::new();

        world.spawn();
    }

    #[test]
    fn test_component() {
        struct TestComponent {
            test: u32,
        }

        impl Component for TestComponent {}

        let mut world = World::new();
        let entity = world.spawn();

        world.add_component(entity, TestComponent { test: 1 });
        assert_eq!(world.component::<TestComponent>(entity).unwrap().test, 1);
    }

    #[test]
    fn test_components() {
        struct TestComponent {
            test: u32,
        }

        impl Component for TestComponent {}

        let mut world = World::new();

        let entity = world.spawn();
        world.add_component(entity, TestComponent { test: 1 });

        let entity = world.spawn();
        world.add_component(entity, TestComponent { test: 2 });

        let mut it = world.components::<TestComponent>();
        assert_eq!(it.next().unwrap().1.test, 1);
        assert_eq!(it.next().unwrap().1.test, 2);
        assert!(it.next().is_none());
    }
}
