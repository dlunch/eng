use alloc::vec::Vec;

use super::raw_vec::RawVec;

pub struct SparseRawVec<IndexType = usize>
where
    IndexType: Ord + Copy,
{
    data: RawVec,
    indices: Vec<IndexType>,
}

impl<IndexType> SparseRawVec<IndexType>
where
    IndexType: Ord + Copy,
{
    pub fn new<T: 'static>() -> Self {
        Self {
            data: RawVec::new::<T>(),
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

    pub fn iter<T: 'static>(&self) -> impl Iterator<Item = (IndexType, &T)> {
        self.indices.iter().cloned().zip(self.data.iter())
    }

    pub fn contains(&self, index: IndexType) -> bool {
        self.indices.binary_search(&index).is_ok()
    }

    pub fn remove(&mut self, index: IndexType) -> bool {
        let pos = self.indices.binary_search(&index);
        if pos.is_err() {
            return false;
        }
        let pos = pos.unwrap();

        self.data.remove(pos);
        self.indices.remove(pos);

        true
    }
}

#[cfg(test)]
mod test {
    use alloc::{vec, vec::Vec};

    use super::SparseRawVec;

    #[test]
    fn test_insert() {
        let mut vec = SparseRawVec::new::<i32>();

        vec.insert(0, 10);
        vec.insert(10, 12);
        vec.insert(5, 7);
        vec.insert(3, 1);

        assert_eq!(vec.iter().collect::<Vec<_>>(), vec![(0, &10), (3, &1), (5, &7), (10, &12)]);
    }

    #[test]
    fn test_get() {
        let mut vec = SparseRawVec::new::<i32>();

        vec.insert(0, 10);
        vec.insert(10, 12);

        assert_eq!(*vec.get::<i32>(0).unwrap(), 10);
        assert_eq!(*vec.get_mut::<i32>(10).unwrap(), 12);

        assert_eq!(vec.get::<i32>(1234), None);
    }

    #[test]
    fn test_contains() {
        let mut vec = SparseRawVec::new::<i32>();

        vec.insert(0, 10);
        vec.insert(10, 12);

        assert!(vec.contains(0));
        assert!(!vec.contains(1));
    }

    #[test]
    fn test_remove() {
        let mut vec = SparseRawVec::new::<i32>();

        vec.insert(0, 10);
        vec.insert(10, 12);
        vec.insert(5, 7);
        vec.insert(3, 1);

        vec.remove(5);

        assert_eq!(vec.iter().collect::<Vec<_>>(), vec![(0, &10), (3, &1), (10, &12)]);
    }
}
