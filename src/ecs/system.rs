use core::{any::Any, marker::PhantomData};

use super::{CommandList, World};

pub trait SystemInput {
    type ActualInput<'i>: SystemInput;

    fn new<'w>(world: &'w World, extra: Option<&dyn Any>) -> Self::ActualInput<'w>;
}

impl<'a> SystemInput for &'a World {
    type ActualInput<'i> = &'i World;

    fn new<'w>(world: &'w World, _: Option<&dyn Any>) -> Self::ActualInput<'w> {
        world
    }
}

impl SystemInput for u32 {
    type ActualInput<'i> = u32;

    fn new<'w>(_: &'w World, extra: Option<&dyn Any>) -> Self::ActualInput<'w> {
        *extra.unwrap().downcast_ref::<Self>().unwrap()
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

impl<Func, Input1> System for SystemFunction<Func, (Input1,)>
where
    Func: Fn(Input1::ActualInput<'_>) -> CommandList,
    Input1: SystemInput,
{
    fn run(&self, world: &World, extra: Option<&dyn Any>) -> CommandList {
        (self.0)(Input1::new(world, extra))
    }
}

impl<Func, Input1, Input2> System for SystemFunction<Func, (Input1, Input2)>
where
    Func: Fn(Input1::ActualInput<'_>, Input2::ActualInput<'_>) -> CommandList,
    Input1: SystemInput,
    Input2: SystemInput,
{
    fn run(&self, world: &World, extra: Option<&dyn Any>) -> CommandList {
        (self.0)(Input1::new(world, extra), Input2::new(world, extra))
    }
}
