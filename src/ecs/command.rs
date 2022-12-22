use super::{component::ComponentContainer, world::ComponentType, Entity};

pub(super) enum Command {
    CreateEntity(Vec<ComponentContainer>),
    DestroyEntity(Entity),
    CreateComponent(Entity, Vec<ComponentContainer>),
    DestroyComponent(Vec<ComponentType>),
    UpdateComponent(()),
}

#[derive(Default)]
pub struct CommandList {
    pub(super) commands: Vec<Command>,
}

impl CommandList {
    pub fn new() -> Self {
        Self::default()
    }

    fn create_entity(&mut self, components: Vec<ComponentContainer>) {
        self.commands.push(Command::CreateEntity(components));
    }

    fn destroy_entity(&mut self, entity: Entity) {
        self.commands.push(Command::DestroyEntity(entity));
    }

    fn create_component(&mut self, entity: Entity, components: Vec<ComponentContainer>) {
        self.commands.push(Command::CreateComponent(entity, components));
    }

    fn destroy_component(&mut self, components: Vec<ComponentType>) {
        self.commands.push(Command::DestroyComponent(components));
    }
}
