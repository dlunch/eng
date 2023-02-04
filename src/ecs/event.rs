use core::{any::Any, marker::PhantomData};

use super::{system::SystemInput, World};

#[derive(Eq, PartialEq)]
pub enum KeyboardEvent {
    KeyDown(u8),
    KeyUp(u8),
}

pub struct Event<'a, T>
where
    T: 'static,
{
    world: &'a World,
    _phantom: PhantomData<T>,
}

impl<'a, T> SystemInput for Event<'a, T> {
    type ActualInput<'i> = Event<'i, T>;

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

impl<'a, T> Event<'a, T> {
    pub fn get(&self) -> &T {
        self.world.event::<T>().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ecs::{CommandList, Component};

    #[tokio::test]
    async fn test_keyboard_event() {
        struct TestComponent {
            a: u32,
        }
        impl Component for TestComponent {}

        let mut world = World::new();

        world.add_system(|x: Event<KeyboardEvent>| {
            assert!(*x.get() == KeyboardEvent::KeyDown(0));

            CommandList::new().create_entity((TestComponent { a: 1 },))
        });

        world.on_event(KeyboardEvent::KeyDown(0));
        world.update().await;

        assert!(world.components::<TestComponent>().any(|x| x.1.a == 1))
    }
}
