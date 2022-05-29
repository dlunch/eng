use alloc::vec::Vec;
use core::{any::TypeId, mem::align_of, mem::size_of, slice};

struct ItemInfo {
    r#type: TypeId,
    item_size: usize,
}

// heterogeneous vec
pub struct AnyVec {
    storage: Vec<u8>,
    types: Vec<ItemInfo>,
}

impl AnyVec {
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
            types: Vec::new(),
        }
    }

    pub fn insert<T: 'static>(&mut self, index: usize, value: T) {
        let offset = self.calculate_offset(index);

        self.insert_at(offset, value);
        self.types.insert(
            index,
            ItemInfo {
                r#type: TypeId::of::<T>(),
                item_size: Self::round_up(size_of::<T>(), align_of::<T>()),
            },
        );
    }

    pub fn get<T: 'static>(&self, index: usize) -> Option<&T> {
        assert_eq!(self.types[index].r#type, TypeId::of::<T>());

        let offset = self.calculate_offset(index);
        if offset >= self.storage.len() {
            return None;
        }

        let value_ptr = &self.storage[offset..offset + size_of::<T>()] as *const [u8] as *const T;
        Some(unsafe { &*value_ptr })
    }

    pub fn get_mut<T: 'static>(&mut self, index: usize) -> Option<&mut T> {
        assert_eq!(self.types[index].r#type, TypeId::of::<T>());

        let offset = self.calculate_offset(index);
        if offset >= self.storage.len() {
            return None;
        }

        let value_ptr = &mut self.storage[offset..offset + size_of::<T>()] as *mut [u8] as *mut T;
        Some(unsafe { &mut *value_ptr })
    }

    fn calculate_offset(&self, index: usize) -> usize {
        self.types.iter().take(index).map(|x| x.item_size).sum()
    }

    fn insert_at<T: 'static>(&mut self, offset: usize, value: T) {
        let size = size_of::<T>();

        let value_ptr = &value as *const T as *const u8;
        let value_slice = unsafe { slice::from_raw_parts(value_ptr, size) };
        self.storage.splice(offset..offset, value_slice.iter().cloned());

        core::mem::forget(value);
    }

    const fn round_up(num_to_round: usize, multiple: usize) -> usize {
        if multiple == 0 {
            return num_to_round;
        }

        let remainder = num_to_round % multiple;
        if remainder == 0 {
            num_to_round
        } else {
            num_to_round + multiple - remainder
        }
    }
}

#[cfg(test)]
mod test {
    use super::AnyVec;
    use alloc::{vec, vec::Vec};

    #[test]
    fn test_push() {
        struct TestStruct {
            a: usize,
            b: usize,
        }

        let mut vec = AnyVec::new();
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

        let mut vec = AnyVec::new();

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

        let mut vec = AnyVec::new();
        vec.insert(0, TestStruct { test: vec![1, 2, 3, 4] });

        assert_eq!(vec.get::<TestStruct>(0).unwrap().test[0], 1);
        assert_eq!(vec.get::<TestStruct>(0).unwrap().test[1], 2);
    }

    #[test]
    fn test_heterogeneous() {
        struct TestStruct1 {
            a: usize,
        }
        struct TestStruct2 {
            b: usize,
            c: usize,
        }

        let mut vec = AnyVec::new();
        vec.insert(0, TestStruct1 { a: 0 });
        vec.insert(1, TestStruct2 { b: 1, c: 2 });
        vec.insert(2, TestStruct1 { a: 3 });

        assert_eq!(vec.get::<TestStruct1>(0).unwrap().a, 0);
        assert_eq!(vec.get::<TestStruct2>(1).unwrap().b, 1);
        assert_eq!(vec.get::<TestStruct2>(1).unwrap().c, 2);
        assert_eq!(vec.get::<TestStruct1>(2).unwrap().a, 3);
    }
}
