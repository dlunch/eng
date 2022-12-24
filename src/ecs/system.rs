use core::{any::Any, marker::PhantomData};

use super::{CommandList, World};

pub trait SystemInput {
    fn new<'a>(world: &'a World, extra: Option<&'a dyn Any>) -> &'a Self;
}

impl SystemInput for World {
    fn new<'a>(world: &'a World, _: Option<&'a dyn Any>) -> &'a Self {
        world
    }
}

impl SystemInput for u32 {
    fn new<'a>(_: &'a World, extra: Option<&'a dyn Any>) -> &'a Self {
        extra.unwrap().downcast_ref::<Self>().unwrap()
    }
}

pub trait System {
    fn run(&self, world: &World, extra: Option<&dyn Any>) -> CommandList;
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
    fn run(&self, world: &World, extra: Option<&dyn Any>) -> CommandList {
        (self.0)(Input1::new(world, extra))
    }
}

impl<T, Input1, Input2> System for SystemFunction<T, (Input1, Input2)>
where
    T: Fn(&Input1, &Input2) -> CommandList,
    Input1: SystemInput,
    Input2: SystemInput,
{
    fn run(&self, world: &World, extra: Option<&dyn Any>) -> CommandList {
        (self.0)(Input1::new(world, extra), Input2::new(world, extra))
    }
}
