use super::{component::ComponentContainer, world::ComponentType, Component, ComponentBundle, Entity, World};

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

    pub fn create_entity<T>(&mut self, world: &mut World, bundle: T)
    where
        T: ComponentBundle,
    {
        self.commands.push(Command::CreateEntity(bundle.to_component_containers(world)));
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self.commands.push(Command::DestroyEntity(entity));
    }

    pub fn create_component<T>(&mut self, entity: Entity, component: T)
    where
        T: Component + 'static,
    {
        self.commands
            .push(Command::CreateComponent(entity, vec![ComponentContainer::new(component)]));
    }

    pub fn destroy_component(&mut self, components: Vec<ComponentType>) {
        self.commands.push(Command::DestroyComponent(components));
    }
}
