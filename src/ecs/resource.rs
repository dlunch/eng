use core::{any::Any, cell::Ref, marker::PhantomData};

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
    pub fn get<'r>(&self) -> Ref<'r, T>
    where
        'a: 'r,
    {
        self.world.resource::<T>().unwrap()
    }
}
