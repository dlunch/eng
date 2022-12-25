use core::{any::Any, marker::PhantomData};

use super::{CommandList, World};

pub trait SystemInput {
    type ActualInput<'i>: SystemInput;

    fn is_available(world: &World) -> bool;
    fn new<'w>(world: &'w World, extra: Option<&dyn Any>) -> Self::ActualInput<'w>;
}

impl<'a> SystemInput for &'a World {
    type ActualInput<'i> = &'i World;

    fn is_available(_: &World) -> bool {
        true
    }

    fn new<'w>(world: &'w World, _: Option<&dyn Any>) -> Self::ActualInput<'w> {
        world
    }
}

impl SystemInput for u32 {
    type ActualInput<'i> = u32;
    fn is_available(_: &World) -> bool {
        true
    }

    fn new<'w>(_: &'w World, extra: Option<&dyn Any>) -> Self::ActualInput<'w> {
        *extra.unwrap().downcast_ref::<Self>().unwrap()
    }
}

pub trait System {
    fn is_available(&self, world: &World) -> bool;
    fn run(&self, world: &World, extra: Option<&dyn Any>) -> CommandList;
}

struct SystemFunction<F, Input>(F, PhantomData<Input>);

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
    fn is_available(&self, world: &World) -> bool {
        Input1::is_available(world)
    }

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
    fn is_available(&self, world: &World) -> bool {
        Input1::is_available(world) && Input2::is_available(world)
    }

    fn run(&self, world: &World, extra: Option<&dyn Any>) -> CommandList {
        (self.0)(Input1::new(world, extra), Input2::new(world, extra))
    }
}

pub trait IntoSystem<T> {
    fn into_system(self) -> Box<dyn System>;
}

impl<Func, Input1> IntoSystem<(Input1,)> for Func
where
    Func: Fn(Input1) -> CommandList + Fn(Input1::ActualInput<'_>) -> CommandList + 'static,
    Input1: SystemInput + 'static,
{
    fn into_system(self) -> Box<dyn System> {
        Box::new(SystemFunction::new(self))
    }
}

impl<Func, Input1, Input2> IntoSystem<(Input1, Input2)> for Func
where
    Func: Fn(Input1, Input2) -> CommandList + Fn(Input1::ActualInput<'_>, Input2::ActualInput<'_>) -> CommandList + 'static,
    Input1: SystemInput + 'static,
    Input2: SystemInput + 'static,
{
    fn into_system(self) -> Box<dyn System> {
        Box::new(SystemFunction::new(self))
    }
}
