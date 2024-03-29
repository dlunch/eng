use alloc::{boxed::Box, vec::Vec};
use core::{
    any::{Any, TypeId},
    future::Future,
};

use futures::{future::BoxFuture, poll, task::Poll, FutureExt};
use hashbrown::HashMap;

use super::{
    builder::EntityBuilder,
    bundle::ComponentBundle,
    command::{Command, CommandList},
    component::ComponentContainer,
    sparse_raw_vec::SparseRawVec,
    system::{IntoSystem, System, SystemInput},
    Component, Entity,
};

pub type ComponentType = TypeId;
pub type ResourceType = TypeId;
pub type EventType = TypeId;

type PendingFuture = BoxFuture<'static, Box<dyn Any>>;

pub trait AsyncSingleArgFnOnce<Arg>: FnOnce(Arg) -> <Self as AsyncSingleArgFnOnce<Arg>>::Fut {
    type Fut: Future<Output = <Self as AsyncSingleArgFnOnce<Arg>>::Output>;
    type Output;
}

impl<Arg, F, Fut> AsyncSingleArgFnOnce<Arg> for F
where
    F: FnOnce(Arg) -> Fut,
    Fut: Future,
{
    type Fut = Fut;
    type Output = Fut::Output;
}

pub struct World {
    components: HashMap<ComponentType, SparseRawVec<Entity>>,
    resources: HashMap<ResourceType, Box<dyn Any>>,
    entities: u32,
    pending: Vec<(PendingFuture, Box<dyn System>)>,
    events: HashMap<EventType, Box<dyn Any>>,
    systems: Vec<Box<dyn System>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            resources: HashMap::new(),
            entities: 0,
            pending: Vec::new(),
            events: HashMap::new(),
            systems: Vec::new(),
        }
    }

    pub async fn setup<F>(&mut self, setup_fn: F)
    where
        F: for<'a> AsyncSingleArgFnOnce<&'a Self, Output = CommandList>,
    {
        let commands = setup_fn(self).await.commands;

        self.run_commands(commands)
    }

    pub fn spawn(&mut self) -> EntityBuilder<'_> {
        let id = self.entities;

        self.entities += 1;

        EntityBuilder::new(self, Entity { id })
    }

    pub fn destroy(&mut self, entity: Entity) {
        for (_, storage) in self.components.iter_mut() {
            storage.remove(entity);
        }
    }

    pub fn spawn_bundle<T: 'static + ComponentBundle>(&mut self, bundle: T) -> Entity {
        let entity = self.spawn().entity();

        self.add_bundle(entity, bundle);

        entity
    }

    pub fn add_bundle<T: 'static + ComponentBundle>(&mut self, entity: Entity, bundle: T) {
        bundle.add_components(self, entity)
    }

    pub fn add_component<T: 'static + Component>(&mut self, entity: Entity, component: T) {
        let component_type = Self::get_component_type::<T>();

        let vec = if let Some(x) = self.components.get_mut(&component_type) {
            x
        } else {
            let vec = SparseRawVec::new::<T>();
            self.components.insert(component_type, vec);

            self.components.get_mut(&component_type).unwrap()
        };

        vec.insert(entity, component);
    }

    fn add_component_raw(&mut self, entity: Entity, component_container: ComponentContainer) {
        let vec = if let Some(x) = self.components.get_mut(&component_container.component_type) {
            x
        } else {
            let vec = SparseRawVec::with_type_descriptor(component_container.type_descriptor);
            self.components.insert(component_container.component_type, vec);

            self.components.get_mut(&component_container.component_type).unwrap()
        };

        vec.insert_raw(entity, &component_container.data);
    }

    pub fn component<T: 'static + Component>(&self, entity: Entity) -> Option<&T> {
        let component_type = Self::get_component_type::<T>();

        self.components.get(&component_type)?.get::<T>(entity)
    }

    pub fn component_mut<T: 'static + Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let component_type = Self::get_component_type::<T>();

        self.components.get_mut(&component_type)?.get_mut::<T>(entity)
    }

    pub fn components<T: 'static + Component>(&self) -> impl Iterator<Item = (Entity, &T)> {
        let component_type = Self::get_component_type::<T>();

        self.components.get(&component_type).unwrap().iter()
    }

    pub fn components_mut<T: 'static + Component>(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
        let component_type = Self::get_component_type::<T>();

        self.components.get_mut(&component_type).unwrap().iter_mut()
    }

    pub fn has_component<T: 'static + Component>(&self, entity: Entity) -> bool {
        let component_type = Self::get_component_type::<T>();

        if let Some(components) = self.components.get(&component_type) {
            components.contains(entity)
        } else {
            false
        }
    }

    pub fn add_resource<T: 'static>(&mut self, resource: T) {
        let resource_type = Self::get_resource_type::<T>();

        self.resources.insert(resource_type, Box::new(resource));
    }

    pub fn resource<T: 'static>(&self) -> Option<&T> {
        let resource_type = Self::get_resource_type::<T>();

        let storage = self.resources.get(&resource_type)?;

        Some(storage.downcast_ref::<T>().unwrap())
    }

    pub fn resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let resource_type = Self::get_resource_type::<T>();

        let storage = self.resources.get_mut(&resource_type)?;

        Some(storage.downcast_mut::<T>().unwrap())
    }

    pub fn take_resource<T: 'static>(&mut self) -> Option<T> {
        let resource_type = Self::get_resource_type::<T>();

        Some(*self.resources.remove(&resource_type)?.downcast::<T>().unwrap())
    }

    pub fn entities(&self) -> impl Iterator<Item = Entity> {
        (0..self.entities).map(|x| Entity { id: x })
    }

    pub fn event<T>(&self) -> Option<&T>
    where
        T: 'static,
    {
        self.events.get(&Self::get_event_type::<T>()).map(|x| x.downcast_ref::<T>().unwrap())
    }

    pub fn async_job<'w, Job, JobFut, Callback, Output>(&mut self, job: Job, callback: Callback)
    where
        Job: FnOnce() -> JobFut,
        for<'a> JobFut: Future<Output = Output> + Sync + Send + 'a,
        Callback: IntoSystem<(&'w World, Output)>,
        Output: SystemInput + 'static,
    {
        let fut = job().map(|x| Box::new(x) as Box<dyn Any>).fuse().boxed();

        self.pending.push((fut, callback.into_system()));
    }

    pub(crate) async fn update(&mut self) {
        let mut pending = Vec::with_capacity(self.pending.len());
        core::mem::swap(&mut self.pending, &mut pending);

        let mut commands = Vec::new();
        for (mut future, callback) in pending {
            if let Poll::Ready(x) = poll!(&mut future) {
                commands.extend(callback.run(self, Some(&*x)).commands.into_iter());
            } else {
                self.pending.push((future, callback));
            }
        }

        commands.extend(
            self.systems
                .iter()
                .filter(|x| x.is_available(self))
                .flat_map(|x| x.run(self, None).commands),
        );

        self.run_commands(commands);
        self.events.clear();
    }

    #[allow(dead_code)]
    pub(crate) fn on_event<EventT>(&mut self, event: EventT)
    where
        EventT: 'static,
    {
        let event_type = Self::get_event_type::<EventT>();

        self.events.insert(event_type, Box::new(event));
    }

    fn run_commands(&mut self, commands: Vec<Command>) {
        for command in commands {
            match command {
                Command::CreateEntity(components) => {
                    let entity = self.spawn().entity();
                    for component in components {
                        self.add_component_raw(entity, component);
                    }
                }
                Command::DestroyEntity(entity) => self.destroy(entity),
                Command::CreateComponent(entity, components) => {
                    for component in components {
                        self.add_component_raw(entity, component);
                    }
                }
                Command::DestroyComponent(_) => (), // TOOD
            }
        }
    }

    pub fn add_system<T, P>(&mut self, system: T)
    where
        T: IntoSystem<P>,
    {
        self.systems.push(system.into_system());
    }

    fn get_component_type<ComponentT>() -> ComponentType
    where
        ComponentT: Component + 'static,
    {
        TypeId::of::<ComponentT>()
    }

    fn get_resource_type<ResourceT>() -> ResourceType
    where
        ResourceT: 'static,
    {
        TypeId::of::<ResourceT>()
    }

    fn get_event_type<EventT>() -> EventType
    where
        EventT: 'static,
    {
        TypeId::of::<EventT>()
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

    use crate::ecs::CommandList;

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
    fn test_component_empty() {
        struct TestComponent {}

        impl Component for TestComponent {}

        let mut world = World::new();
        let entity = world.spawn().with(TestComponent {}).entity();

        assert!(world.has_component::<TestComponent>(entity));
        assert!(world.component::<TestComponent>(entity).is_some());
    }

    #[test]
    fn test_components() {
        struct TestComponent {
            test: u32,
        }

        impl Component for TestComponent {}

        let mut world = World::new();

        world.spawn().with(TestComponent { test: 1 }).entity();

        world.spawn().with(TestComponent { test: 2 }).entity();

        let mut it = world.components::<TestComponent>();
        assert_eq!(it.next().unwrap().1.test, 1);
        assert_eq!(it.next().unwrap().1.test, 2);
        assert!(it.next().is_none());
    }

    #[test]
    fn test_components_mut() {
        struct TestComponent {
            test: u32,
        }

        impl Component for TestComponent {}

        let mut world = World::new();

        world.spawn().with(TestComponent { test: 1 }).entity();

        world.spawn().with(TestComponent { test: 2 }).entity();

        {
            let mut it = world.components_mut::<TestComponent>();
            it.next().unwrap().1.test = 123;
        }

        let mut it = world.components::<TestComponent>();

        assert_eq!(it.next().unwrap().1.test, 123);
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

    #[test]
    fn test_bundle() {
        struct TestComponent1 {
            a: u32,
        }
        impl Component for TestComponent1 {}
        struct TestComponent2 {
            a: u32,
        }
        impl Component for TestComponent2 {}

        let mut world = World::new();

        let bundle = (TestComponent1 { a: 1 }, TestComponent2 { a: 2 });
        let entity = world.spawn_bundle(bundle);

        assert_eq!(world.component::<TestComponent1>(entity).unwrap().a, 1);
        assert_eq!(world.component::<TestComponent2>(entity).unwrap().a, 2);
    }

    #[test]
    fn test_has_component() {
        struct TestComponent {}

        impl Component for TestComponent {}

        let mut world = World::new();

        let entity1 = world.spawn().with(TestComponent {}).entity();
        let entity2 = world.spawn().entity();

        assert!(world.has_component::<TestComponent>(entity1));
        assert!(!world.has_component::<TestComponent>(entity2));
    }

    #[test]
    fn test_destroy() {
        struct TestComponent {}

        impl Component for TestComponent {}

        let mut world = World::new();
        let entity = world.spawn().with(TestComponent {}).entity();

        world.destroy(entity);

        assert!(world.component::<TestComponent>(entity).is_none());
    }

    #[tokio::test]
    async fn test_async() {
        struct TestComponent {
            v: u32,
        }

        impl Component for TestComponent {}

        let mut world = World::new();

        world.async_job(|| async { 1 }, |_: &World, v| CommandList::new().create_entity((TestComponent { v },)));

        world.update().await;

        assert_eq!(world.components::<TestComponent>().next().unwrap().1.v, 1);
    }

    #[test]
    fn test_command() {
        struct TestComponent1 {
            a: u32,
        }
        impl Component for TestComponent1 {}
        struct TestComponent2 {
            a: u32,
        }
        impl Component for TestComponent2 {}

        let mut world = World::new();

        let cmd_list = CommandList::new().create_entity((TestComponent1 { a: 1 },));

        world.run_commands(cmd_list.commands);

        let (entity, component) = world.components::<TestComponent1>().next().unwrap();
        assert_eq!(component.a, 1);

        let cmd_list = CommandList::new().create_component(entity, TestComponent2 { a: 2 });

        world.run_commands(cmd_list.commands);

        assert_eq!(world.component::<TestComponent2>(entity).unwrap().a, 2);
    }
}
