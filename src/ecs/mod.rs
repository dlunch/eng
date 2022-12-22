mod builder;
mod bundle;
mod command;
mod component;
mod event;
mod hierarchy;
mod query;
mod raw_vec;
mod sparse_raw_vec;
mod type_descriptor;
mod world;

pub use bundle::ComponentBundle;
pub use command::CommandList;
pub use component::Component;
pub use event::KeyboardEvent;
pub use hierarchy::HierarchyExt;
pub use world::World;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Entity {
    id: u32,
}
