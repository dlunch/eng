use core::marker::PhantomData;

use super::{CommandList, World};

pub trait SystemInput {
    fn from_world(world: &World) -> &Self;
}

impl SystemInput for World {
    fn from_world(world: &World) -> &Self {
        world
    }
}

pub trait System {
    fn run(&self, world: &World) -> CommandList;
}

pub struct SystemFunction<F, Input>(F, PhantomData<Input>);

impl<F, Input> SystemFunction<F, Input> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<T, Input1> System for SystemFunction<T, (Input1,)>
where
    T: Fn(&Input1) -> CommandList,
    Input1: SystemInput,
{
    fn run(&self, world: &World) -> CommandList {
        (self.0)(Input1::from_world(world))
    }
}

impl<T, Input1, Input2> System for SystemFunction<T, (Input1, Input2)>
where
    T: Fn(&Input1, &Input2) -> CommandList,
    Input1: SystemInput,
    Input2: SystemInput,
{
    fn run(&self, world: &World) -> CommandList {
        (self.0)(Input1::from_world(world), Input2::from_world(world))
    }
}
