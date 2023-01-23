use core::{any::Any, marker::PhantomData};

use super::{system::SystemInput, World};

pub struct Resource<'a, T: 'static> {
    world: &'a World,
    _phantom: PhantomData<T>,
}

impl<'a, T> SystemInput for Resource<'a, T> {
    type ActualInput<'i> = Resource<'i, T>;

    fn is_available(_: &World) -> bool {
        true
    }

    fn new<'w>(world: &'w World, _: Option<&dyn Any>) -> Self::ActualInput<'w> {
        Self::ActualInput::<'w> {
            world,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T> Resource<'a, T> {
    pub fn get<'r>(&self) -> &T
    where
        'a: 'r,
    {
        self.world.resource::<T>().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ecs::CommandList;

    #[tokio::test]
    async fn test_resource() {
        struct TestResource1 {
            a: u32,
        }
        struct TestResource2 {
            b: Vec<u32>,
        }
        let mut world = World::new();

        world.add_resource(TestResource1 { a: 123 });
        world.add_resource(TestResource2 { b: vec![1234] });

        world.add_system(|x: Resource<TestResource1>| {
            assert_eq!(x.get().a, 123);

            CommandList::new()
        });

        world.add_system(|x: Resource<TestResource2>| {
            assert_eq!(x.get().b[0], 1234);

            CommandList::new()
        });

        world.update().await;
    }
}
