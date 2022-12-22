use alloc::boxed::Box;
use core::{
    any::TypeId,
    mem::{align_of, size_of},
};

use crate::utils::round_up;

pub struct TypeDescriptor {
    pub(super) item_size: usize,
    pub(super) drop: Box<dyn Fn(&mut [u8])>,

    #[cfg(debug_assertions)]
    pub(super) actual_type: TypeId,
}

impl TypeDescriptor {
    pub fn new<T: 'static>() -> Self {
        Self {
            item_size: round_up(size_of::<T>(), align_of::<T>()),
            drop: Box::new(Self::drop::<T>),

            #[cfg(debug_assertions)]
            actual_type: TypeId::of::<T>(),
        }
    }

    pub fn from_raw<F>(size: usize, align: usize, drop: F, _type_id: TypeId) -> Self
    where
        for<'a> F: Fn(&'a mut [u8]) + 'static,
    {
        Self {
            item_size: round_up(size, align),
            drop: Box::new(drop),

            #[cfg(debug_assertions)]
            actual_type: _type_id,
        }
    }

    fn drop<T: 'static>(data: &mut [u8]) {
        let value_ptr = data.as_mut_ptr() as *mut T;
        unsafe { value_ptr.drop_in_place() }
    }
}
