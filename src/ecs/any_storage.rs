use alloc::{boxed::Box, vec::Vec};
use core::{any::TypeId, mem::size_of, ops::Drop, slice};

pub struct AnyStorage {
    storage: Vec<u8>,
    #[cfg(debug_assertions)]
    actual_type: TypeId,
    drop: Box<fn(&mut [u8])>,
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
            drop: Box::new(Self::drop::<T>),
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

    fn drop<T: 'static>(value_slice: &mut [u8]) {
        let value_ptr = value_slice.as_mut_ptr() as *mut T;
        unsafe { value_ptr.drop_in_place() }
    }
}

impl Drop for AnyStorage {
    fn drop(&mut self) {
        (self.drop)(&mut self.storage)
    }
}

#[cfg(test)]
mod test {
    use alloc::{rc::Rc, vec, vec::Vec};
    use core::cell::RefCell;

    use super::AnyStorage;

    #[test]
    fn test_storage() {
        struct TestStruct {
            a: usize,
            b: Vec<u32>,
        }

        let storage = AnyStorage::new(TestStruct { a: 123, b: vec![1, 2, 3, 4] });

        assert_eq!(storage.get::<TestStruct>().a, 123);
        assert_eq!(storage.get::<TestStruct>().b, [1, 2, 3, 4]);
    }

    #[test]
    fn test_drop() {
        let dropped = Rc::new(RefCell::new(false));

        struct TestStruct {
            dropped: Rc<RefCell<bool>>,
        }

        impl Drop for TestStruct {
            fn drop(&mut self) {
                *self.dropped.borrow_mut() = true;
            }
        }
        {
            AnyStorage::new(TestStruct { dropped: dropped.clone() });
        }
        assert!(*dropped.borrow());
    }
}
