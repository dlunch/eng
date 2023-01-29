mod builder;
mod bundle;
mod command;
mod component;
mod event;
mod hierarchy;
mod query;
mod raw_vec;
mod resource;
mod sparse_raw_vec;
mod system;
mod type_descriptor;
mod world;

pub use bundle::ComponentBundle;
pub use command::CommandList;
pub use component::{Component, ComponentContainer};
pub use event::Event;
pub use event::KeyboardEvent;
pub use hierarchy::HierarchyExt;
pub use query::Query;
pub use resource::Resource;
pub use system::IntoSystem;
pub use world::World;

pub(crate) use world::AsyncSingleArgFnOnce;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Entity {
    id: u32,
}
