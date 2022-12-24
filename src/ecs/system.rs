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

pub struct SystemFunction<F, Input>(F, PhantomData<Input>)
where
    Input: SystemInput;

impl<F, Input> SystemFunction<F, Input>
where
    Input: SystemInput,
{
    pub fn new(f: F) -> Self {
        SystemFunction(f, PhantomData)
    }
}

impl<T, Input> System for SystemFunction<T, Input>
where
    T: Fn(&Input) -> CommandList,
    Input: SystemInput,
{
    fn run(&self, world: &World) -> CommandList {
        (self.0)(Input::from_world(world))
    }
}
