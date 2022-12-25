use core::{any::Any, marker::PhantomData};

use super::{system::SystemInput, Component, Entity, World};

pub struct Query<'a, P>
where
    P: QueryParam,
{
    world: &'a World,
    _phantom: PhantomData<P>,
}

impl<'a, P> Query<'a, P>
where
    P: QueryParam,
{
    pub fn new(world: &'a World) -> Self {
        Self {
            world,
            _phantom: PhantomData,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_
    where
        P: QueryParam,
    {
        self.world.entities().filter(move |&entity| P::matches(self.world, entity))
    }
}

pub trait QueryParam {
    fn matches(world: &World, entity: Entity) -> bool;
}

impl<T1> QueryParam for (T1,)
where
    T1: Component + 'static,
{
    fn matches(world: &World, entity: Entity) -> bool {
        world.has_component::<T1>(entity)
    }
}

impl<T1, T2> QueryParam for (T1, T2)
where
    T1: Component + 'static,
    T2: Component + 'static,
{
    fn matches(world: &World, entity: Entity) -> bool {
        world.has_component::<T1>(entity) && world.has_component::<T2>(entity)
    }
}

impl<'a, P> SystemInput for Query<'a, P>
where
    P: QueryParam,
{
    type ActualInput<'i> = Query<'i, P>;

    fn is_available(_: &World) -> bool {
        true
    }

    fn new<'w>(world: &'w World, _: Option<&dyn Any>) -> Self::ActualInput<'w> {
        Self::ActualInput {
            world,
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_query1() {
        struct TestComponent {}

        impl Component for TestComponent {}

        let mut world = World::new();

        let entity1 = world.spawn().with(TestComponent {}).entity();
        world.spawn().entity();

        let query = Query::<(TestComponent,)>::new(&world);

        let mut it = query.iter();

        assert!(it.next().unwrap() == entity1);
        assert!(it.next().is_none());
    }

    #[test]
    fn test_query2() {
        struct TestComponent1 {}
        impl Component for TestComponent1 {}
        struct TestComponent2 {}
        impl Component for TestComponent2 {}

        let mut world = World::new();
        let entity1 = world.spawn().with(TestComponent1 {}).with(TestComponent2 {}).entity();
        world.spawn().entity();

        let query = Query::<(TestComponent1, TestComponent2)>::new(&world);

        let mut it = query.iter();

        assert!(it.next().unwrap() == entity1);
        assert!(it.next().is_none());
    }
}
