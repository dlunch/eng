use alloc::{boxed::Box, vec::Vec};
use core::{
    any::TypeId,
    mem::{align_of, size_of},
    ops::Drop,
    slice,
};

use crate::utils::round_up;

// homogeneous vec
pub struct RawVec {
    storage: Vec<u8>,
    #[cfg(debug_assertions)]
    actual_type: TypeId,
    drop_all: Box<fn(&mut [u8])>,
}

impl RawVec {
    pub fn new<T: 'static>() -> Self {
        Self {
            storage: Vec::new(),
            #[cfg(debug_assertions)]
            actual_type: TypeId::of::<T>(),
            drop_all: Box::new(Self::drop_all::<T>),
        }
    }

    pub fn insert<T: 'static>(&mut self, index: usize, value: T) {
        let item_size = round_up(size_of::<T>(), align_of::<T>());
        let offset = index * item_size;

        self.insert_at(offset, value);
    }

    pub fn get<T: 'static>(&self, index: usize) -> Option<&T> {
        #[cfg(debug_assertions)]
        assert!(TypeId::of::<T>() == self.actual_type);

        let item_size = round_up(size_of::<T>(), align_of::<T>());
        let offset = index * item_size;

        if offset >= self.storage.len() {
            return None;
        }

        let value_ptr = &self.storage[offset..offset + size_of::<T>()] as *const [u8] as *const T;
        Some(unsafe { &*value_ptr })
    }

    pub fn get_mut<T: 'static>(&mut self, index: usize) -> Option<&mut T> {
        #[cfg(debug_assertions)]
        assert!(TypeId::of::<T>() == self.actual_type);

        let item_size = round_up(size_of::<T>(), align_of::<T>());
        let offset = index * item_size;
        if offset >= self.storage.len() {
            return None;
        }

        let value_ptr = &mut self.storage[offset..offset + size_of::<T>()] as *mut [u8] as *mut T;
        Some(unsafe { &mut *value_ptr })
    }

    pub fn iter<T: 'static>(&self) -> impl Iterator<Item = &T> {
        #[cfg(debug_assertions)]
        assert!(TypeId::of::<T>() == self.actual_type);

        let item_size = round_up(size_of::<T>(), align_of::<T>());
        self.storage.chunks(item_size).map(move |x| unsafe { &*(x as *const [u8] as *const T) })
    }

    fn insert_at<T: 'static>(&mut self, offset: usize, value: T) {
        #[cfg(debug_assertions)]
        assert!(TypeId::of::<T>() == self.actual_type);

        let value_ptr = &value as *const T as *const u8;
        let value_slice = unsafe { slice::from_raw_parts(value_ptr, size_of::<T>()) };
        self.storage.splice(offset..offset, value_slice.iter().cloned());

        core::mem::forget(value);
    }

    fn drop_all<T: 'static>(storage: &mut [u8]) {
        let item_size = round_up(size_of::<T>(), align_of::<T>());

        storage.chunks_mut(item_size).for_each(|x| {
            let value_ptr = x.as_mut_ptr() as *mut T;
            unsafe { value_ptr.drop_in_place() }
        })
    }
}

impl Drop for RawVec {
    fn drop(&mut self) {
        (self.drop_all)(&mut self.storage)
    }
}

#[cfg(test)]
mod test {
    use alloc::{rc::Rc, vec, vec::Vec};
    use core::cell::RefCell;

    use super::RawVec;

    #[test]
    fn test_push() {
        struct TestStruct {
            a: usize,
            b: usize,
        }

        let mut vec = RawVec::new::<TestStruct>();
        vec.insert(0, TestStruct { a: 1, b: 2 });

        assert_eq!(vec.get::<TestStruct>(0).unwrap().a, 1);
        assert_eq!(vec.get::<TestStruct>(0).unwrap().b, 2);
    }

    #[test]
    fn test_insert() {
        struct TestStruct {
            a: usize,
            b: usize,
        }

        let mut vec = RawVec::new::<TestStruct>();

        vec.insert(0, TestStruct { a: 1, b: 2 });
        vec.insert(1, TestStruct { a: 2, b: 3 });
        vec.insert(2, TestStruct { a: 4, b: 5 });

        vec.insert(1, TestStruct { a: 6, b: 8 });

        assert_eq!(vec.get::<TestStruct>(0).unwrap().a, 1);
        assert_eq!(vec.get::<TestStruct>(0).unwrap().b, 2);

        assert_eq!(vec.get::<TestStruct>(1).unwrap().a, 6);
        assert_eq!(vec.get::<TestStruct>(1).unwrap().b, 8);

        assert_eq!(vec.get::<TestStruct>(2).unwrap().a, 2);
        assert_eq!(vec.get::<TestStruct>(2).unwrap().b, 3);
    }

    #[test]
    fn test_complex() {
        struct TestStruct {
            test: Vec<u8>,
        }

        let mut vec = RawVec::new::<TestStruct>();
        vec.insert(0, TestStruct { test: vec![1, 2, 3, 4] });

        assert_eq!(vec.get::<TestStruct>(0).unwrap().test[0], 1);
        assert_eq!(vec.get::<TestStruct>(0).unwrap().test[1], 2);
    }

    #[test]
    fn test_drop() {
        let dropped = vec![Rc::new(RefCell::new(false)), Rc::new(RefCell::new(false))];

        struct TestStruct {
            dropped: Rc<RefCell<bool>>,
        }

        impl Drop for TestStruct {
            fn drop(&mut self) {
                *self.dropped.borrow_mut() = true;
            }
        }
        {
            let mut vec = RawVec::new::<TestStruct>();
            vec.insert(0, TestStruct { dropped: dropped[0].clone() });
            vec.insert(1, TestStruct { dropped: dropped[1].clone() });
        }

        assert!(*dropped[0].borrow());
        assert!(*dropped[1].borrow());
    }
}
