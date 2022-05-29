use alloc::vec::Vec;
use core::{any::TypeId, mem::size_of, slice};

pub struct AnyStorage {
    storage: Vec<u8>,
    #[cfg(debug_assertions)]
    actual_type: TypeId,
}

impl AnyStorage {
    pub fn new<T: 'static>(value: T) -> Self {
        let value_ptr = &value as *const T as *const u8;
        let value_slice = unsafe { slice::from_raw_parts(value_ptr, size_of::<T>()) };
        core::mem::forget(value);

        Self {
            storage: value_slice.to_vec(),
            #[cfg(debug_assertions)]
            actual_type: TypeId::of::<T>(),
        }
    }

    pub fn get<T: 'static>(&self) -> &T {
        #[cfg(debug_assertions)]
        assert_eq!(self.actual_type, TypeId::of::<T>());

        let value_ptr = self.storage.as_ptr() as *const T;
        unsafe { &*value_ptr }
    }

    pub fn get_mut<T: 'static>(&mut self) -> &mut T {
        #[cfg(debug_assertions)]
        assert_eq!(self.actual_type, TypeId::of::<T>());

        let value_ptr = self.storage.as_mut_ptr() as *mut T;
        unsafe { &mut *value_ptr }
    }
}
