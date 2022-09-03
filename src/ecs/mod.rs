mod any_storage;
mod builder;
mod hierarchy;
mod raw_vec;
mod sparse_raw_vec;
mod world;

pub use hierarchy::HierarchyExt;
pub use world::World;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Entity {
    id: u32,
}

pub trait Component {}
