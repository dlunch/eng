use super::{component::ComponentContainer, world::ComponentType, Component, ComponentBundle, Entity};

pub(super) enum Command {
    CreateEntity(Vec<ComponentContainer>),
    DestroyEntity(Entity),
    CreateComponent(Entity, Vec<ComponentContainer>),
    DestroyComponent(Vec<ComponentType>),
}

#[derive(Default)]
pub struct CommandList {
    pub(super) commands: Vec<Command>,
}

impl CommandList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_entity<T>(mut self, bundle: T) -> Self
    where
        T: ComponentBundle,
    {
        self.commands.push(Command::CreateEntity(bundle.to_component_containers()));

        self
    }

    pub fn destroy_entity(mut self, entity: Entity) -> Self {
        self.commands.push(Command::DestroyEntity(entity));

        self
    }

    pub fn create_component<T>(mut self, entity: Entity, component: T) -> Self
    where
        T: Component + 'static,
    {
        self.commands
            .push(Command::CreateComponent(entity, vec![ComponentContainer::new(component)]));

        self
    }

    pub fn destroy_component(mut self, components: Vec<ComponentType>) -> Self {
        self.commands.push(Command::DestroyComponent(components));

        self
    }
}
