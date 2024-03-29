use alloc::vec::Vec;
use core::{mem::size_of, ops::Drop, slice};

use super::type_descriptor::TypeDescriptor;

// homogeneous vec
pub struct RawVec {
    storage: Vec<u8>,
    type_descriptor: TypeDescriptor,
}

impl RawVec {
    pub fn new<T: 'static>() -> Self {
        Self::with_type_descriptor(TypeDescriptor::new::<T>())
    }

    pub(super) fn with_type_descriptor(type_descriptor: TypeDescriptor) -> Self {
        Self {
            storage: Vec::new(),
            type_descriptor,
        }
    }

    pub fn insert<T: 'static>(&mut self, index: usize, value: T) {
        #[cfg(debug_assertions)]
        assert!(core::any::TypeId::of::<T>() == self.type_descriptor.actual_type);

        let value_ptr = &value as *const T as *const u8;
        let value_slice = unsafe { slice::from_raw_parts(value_ptr, size_of::<T>()) };
        core::mem::forget(value);

        self.insert_raw(index, value_slice);
    }

    pub fn insert_raw(&mut self, index: usize, value_slice: &[u8]) {
        let offset = self.get_offset(index);
        let value_slice = if self.type_descriptor.item_size == 0 { &[0] } else { value_slice };

        self.storage.splice(offset..offset, value_slice.iter().cloned());
    }

    pub fn get<T: 'static>(&self, index: usize) -> Option<&T> {
        #[cfg(debug_assertions)]
        assert!(core::any::TypeId::of::<T>() == self.type_descriptor.actual_type);

        let offset = self.get_offset(index);
        if offset >= self.storage.len() {
            return None;
        }

        let value_ptr = &self.storage[offset..offset + size_of::<T>()] as *const [u8] as *const T;
        Some(unsafe { &*value_ptr })
    }

    pub fn get_mut<T: 'static>(&mut self, index: usize) -> Option<&mut T> {
        #[cfg(debug_assertions)]
        assert!(core::any::TypeId::of::<T>() == self.type_descriptor.actual_type);

        let offset = self.get_offset(index);
        if offset >= self.storage.len() {
            return None;
        }

        let value_ptr = &mut self.storage[offset..offset + size_of::<T>()] as *mut [u8] as *mut T;
        Some(unsafe { &mut *value_ptr })
    }

    pub fn iter<T: 'static>(&self) -> impl Iterator<Item = &T> {
        #[cfg(debug_assertions)]
        assert!(core::any::TypeId::of::<T>() == self.type_descriptor.actual_type);

        let chunk_size = if self.type_descriptor.item_size != 0 {
            self.type_descriptor.item_size
        } else {
            1
        };

        self.storage.chunks(chunk_size).map(move |x| unsafe { &*(x as *const [u8] as *const T) })
    }

    pub fn iter_mut<T: 'static>(&mut self) -> impl Iterator<Item = &mut T> {
        #[cfg(debug_assertions)]
        assert!(core::any::TypeId::of::<T>() == self.type_descriptor.actual_type);

        let chunk_size = if self.type_descriptor.item_size != 0 {
            self.type_descriptor.item_size
        } else {
            1
        };

        self.storage
            .chunks_mut(chunk_size)
            .map(move |x| unsafe { &mut *(x as *mut [u8] as *mut T) })
    }

    pub fn remove(&mut self, index: usize) -> bool {
        let start = self.get_offset(index);

        let end = if self.type_descriptor.item_size != 0 {
            start + self.type_descriptor.item_size
        } else {
            start + 1
        };

        self.storage.drain(start..end);

        true
    }

    fn get_offset(&self, index: usize) -> usize {
        if self.type_descriptor.item_size == 0 {
            index
        } else {
            index * self.type_descriptor.item_size
        }
    }
}

impl Drop for RawVec {
    fn drop(&mut self) {
        if self.type_descriptor.item_size == 0 {
            return;
        }

        self.storage.chunks_mut(self.type_descriptor.item_size).for_each(|x| {
            (self.type_descriptor.drop)(x);
        })
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

    #[test]
    fn test_remove() {
        struct TestStruct {
            a: usize,
            b: usize,
        }

        let mut vec = RawVec::new::<TestStruct>();

        vec.insert(0, TestStruct { a: 1, b: 2 });
        vec.insert(1, TestStruct { a: 2, b: 3 });
        vec.insert(2, TestStruct { a: 4, b: 5 });

        vec.remove(1);

        assert_eq!(vec.get::<TestStruct>(0).unwrap().a, 1);
        assert_eq!(vec.get::<TestStruct>(0).unwrap().b, 2);

        assert_eq!(vec.get::<TestStruct>(1).unwrap().a, 4);
        assert_eq!(vec.get::<TestStruct>(1).unwrap().b, 5);
    }

    #[test]
    fn test_zero_size() {
        struct TestStruct {}

        let mut vec = RawVec::new::<TestStruct>();

        vec.insert(0, TestStruct {});
        vec.insert(1, TestStruct {});
        vec.insert(2, TestStruct {});

        assert_eq!(vec.iter::<TestStruct>().count(), 3);

        vec.remove(1);
        assert_eq!(vec.iter::<TestStruct>().count(), 2);
    }
}
