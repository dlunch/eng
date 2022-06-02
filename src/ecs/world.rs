use alloc::collections::BTreeMap;
use core::any::TypeId;

use super::{any_storage::AnyStorage, builder::EntityBuilder, sparse_raw_vec::SparseRawVec, Component, Entity};

type ComponentType = TypeId;
type ResourceType = TypeId;

pub struct World {
    components: BTreeMap<ComponentType, SparseRawVec<Entity>>,
    resources: BTreeMap<ResourceType, AnyStorage>,
    entities: u32,
}

impl World {
    pub fn new() -> Self {
        Self {
            components: BTreeMap::new(),
            resources: BTreeMap::new(),
            entities: 0,
        }
    }

    pub fn spawn(&mut self) -> EntityBuilder<'_> {
        let id = self.entities;

        self.entities += 1;

        EntityBuilder::new(self, Entity { id })
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

    pub fn add_resource<T: 'static>(&mut self, resource: T) {
        let resource_type = TypeId::of::<T>();

        self.resources.insert(resource_type, AnyStorage::new(resource));
    }

    pub fn resource<T: 'static>(&self) -> Option<&T> {
        let resource_type = TypeId::of::<T>();

        Some(self.resources.get(&resource_type)?.get::<T>())
    }

    pub fn resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let resource_type = TypeId::of::<T>();

        Some(self.resources.get_mut(&resource_type)?.get_mut::<T>())
    }

    pub fn take_resource<T: 'static>(&mut self) -> Option<T> {
        let resource_type = TypeId::of::<T>();

        Some(self.resources.remove(&resource_type)?.into_inner::<T>())
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use alloc::{vec, vec::Vec};

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
        let entity = world.spawn().with(TestComponent { test: 1 }).entity();

        assert_eq!(world.component::<TestComponent>(entity).unwrap().test, 1);
    }

    #[test]
    fn test_components() {
        struct TestComponent {
            test: u32,
        }

        impl Component for TestComponent {}

        let mut world = World::new();

        let entity = world.spawn().with(TestComponent { test: 1 }).entity();

        let entity = world.spawn().with(TestComponent { test: 2 }).entity();

        let mut it = world.components::<TestComponent>();
        assert_eq!(it.next().unwrap().1.test, 1);
        assert_eq!(it.next().unwrap().1.test, 2);
        assert!(it.next().is_none());
    }

    #[test]
    fn test_resource() {
        struct TestResource1 {
            a: u32,
        }
        struct TestResource2 {
            b: Vec<u32>,
        }
        let mut world = World::new();

        world.add_resource(TestResource1 { a: 123 });
        world.add_resource(TestResource2 { b: vec![1234] });

        assert_eq!(world.resource::<TestResource1>().unwrap().a, 123);
        assert_eq!(world.resource::<TestResource2>().unwrap().b, [1234]);
    }

    #[test]
    fn test_resource_overwrite() {
        struct TestResource {
            a: u32,
        }
        let mut world = World::new();

        world.add_resource(TestResource { a: 123 });
        assert_eq!(world.resource::<TestResource>().unwrap().a, 123);

        world.add_resource(TestResource { a: 1234 });
        assert_eq!(world.resource::<TestResource>().unwrap().a, 1234);
    }
}
