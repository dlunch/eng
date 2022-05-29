use alloc::vec::Vec;

use super::any_vec::AnyVec;

pub struct SparseAnyVec<IndexType = usize>
where
    IndexType: Ord + Copy,
{
    data: AnyVec,
    indices: Vec<IndexType>,
}

impl<IndexType> SparseAnyVec<IndexType>
where
    IndexType: Ord + Copy,
{
    pub fn new() -> Self {
        Self {
            data: AnyVec::new(),
            indices: Vec::new(),
        }
    }

    pub fn insert<T: 'static>(&mut self, index: IndexType, value: T) {
        // find position
        let pos = self.indices.partition_point(|&x| x < index);

        self.data.insert(pos, value);
        self.indices.insert(pos, index);
    }

    pub fn get<T: 'static>(&self, index: IndexType) -> Option<&T> {
        let pos = self.indices.binary_search(&index).ok()?;

        self.data.get::<T>(pos)
    }

    pub fn get_mut<T: 'static>(&mut self, index: IndexType) -> Option<&mut T> {
        let pos = self.indices.binary_search(&index).ok()?;

        self.data.get_mut::<T>(pos)
    }
}

#[cfg(test)]
mod test {
    use super::SparseAnyVec;

    #[test]
    fn test_insert() {
        let mut vec = SparseAnyVec::new();

        vec.insert(0, 10);
        vec.insert(10, 12.0f32);
        vec.insert(5, 7);
        vec.insert(3, 1);

        assert_eq!(*vec.get::<i32>(0).unwrap(), 10);
        assert_eq!(*vec.get::<f32>(10).unwrap(), 12.0);
        assert_eq!(*vec.get::<i32>(5).unwrap(), 7);
        assert_eq!(*vec.get::<i32>(3).unwrap(), 1);
    }

    #[test]
    fn test_get() {
        let mut vec = SparseAnyVec::new();

        vec.insert(0, 10);
        vec.insert(10, 12);

        assert_eq!(*vec.get::<i32>(0).unwrap(), 10);
        assert_eq!(*vec.get_mut::<i32>(10).unwrap(), 12);

        assert_eq!(vec.get::<i32>(1234), None);
    }
}
