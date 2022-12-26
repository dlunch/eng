use core::{any::TypeId, mem::size_of, slice};

use super::type_descriptor::TypeDescriptor;

pub type ComponentType = TypeId;
pub trait Component {}

pub struct ComponentContainer {
    pub component_type: ComponentType,
    pub type_descriptor: TypeDescriptor,
    pub data: Box<[u8]>,
}

impl ComponentContainer {
    pub fn new<T: Component + 'static>(component: T) -> Self {
        let data = unsafe { slice::from_raw_parts(&component as *const T as *const u8, size_of::<T>()) };
        core::mem::forget(component);

        Self {
            component_type: Self::to_component_type::<T>(),
            type_descriptor: TypeDescriptor::new::<T>(),
            data: data.into(),
        }
    }

    pub fn to_component_type<T: Component + 'static>() -> ComponentType {
        TypeId::of::<T>()
    }
}
