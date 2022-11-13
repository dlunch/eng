use alloc::{
    alloc::{alloc, dealloc, Layout},
    boxed::Box,
};
use core::{
    any::TypeId,
    mem::{align_of, size_of},
    ops::Drop,
    ptr,
};

pub struct AnyStorage {
    storage: *mut u8,
    drop: Box<fn(*mut u8)>,
    #[cfg(debug_assertions)]
    actual_type: TypeId,
}

impl AnyStorage {
    pub fn new<T: 'static>(value: T) -> Self {
        let storage = unsafe {
            let storage = alloc(Layout::from_size_align(size_of::<T>(), align_of::<T>()).unwrap());
            ptr::copy_nonoverlapping(&value as *const T as *const u8, storage, size_of::<T>());

            storage
        };

        core::mem::forget(value);

        Self {
            storage,
            drop: Box::new(Self::drop::<T>),
            #[cfg(debug_assertions)]
            actual_type: TypeId::of::<T>(),
        }
    }

    pub fn get<T: 'static>(&self) -> &T {
        #[cfg(debug_assertions)]
        assert_eq!(self.actual_type, TypeId::of::<T>());

        let value_ptr = self.storage as *const T;
        unsafe { &*value_ptr }
    }

    pub fn get_mut<T: 'static>(&mut self) -> &mut T {
        #[cfg(debug_assertions)]
        assert_eq!(self.actual_type, TypeId::of::<T>());

        let value_ptr = self.storage as *mut T;
        unsafe { &mut *value_ptr }
    }

    pub fn into_inner<T: 'static>(self) -> T {
        #[cfg(debug_assertions)]
        assert_eq!(self.actual_type, TypeId::of::<T>());

        let value_ptr = self.storage as *const T;
        core::mem::forget(self);

        unsafe { value_ptr.read() }
    }

    fn drop<T: 'static>(storage: *mut u8) {
        unsafe {
            let value_ptr = storage as *mut T;
            value_ptr.drop_in_place();

            dealloc(storage, Layout::from_size_align(size_of::<T>(), align_of::<T>()).unwrap());
        }
    }
}

impl Drop for AnyStorage {
    fn drop(&mut self) {
        (self.drop)(self.storage);
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

    #[test]
    fn test_into_inner() {
        let dropped = Rc::new(RefCell::new(false));

        struct TestStruct {
            dropped: Rc<RefCell<bool>>,
        }

        impl Drop for TestStruct {
            fn drop(&mut self) {
                *self.dropped.borrow_mut() = true;
            }
        }
        let test = {
            let storage = AnyStorage::new(TestStruct { dropped: dropped.clone() });
            storage.into_inner::<TestStruct>()
        };

        assert!(!*dropped.borrow());
        core::mem::drop(test);
        assert!(*dropped.borrow());
    }
}
