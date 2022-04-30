use alloc::vec::Vec;

pub struct SparseVec<T, IndexType = usize>
where
    IndexType: Ord + Copy,
{
    data: Vec<T>,
    indices: Vec<IndexType>,
}

impl<T, IndexType> SparseVec<T, IndexType>
where
    IndexType: Ord + Copy,
{
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn insert(&mut self, index: IndexType, value: T) {
        // find position
        let pos = self.indices.partition_point(|&x| x < index);

        self.data.insert(pos, value);
        self.indices.insert(pos, index);
    }

    pub fn get(&self, index: IndexType) -> Option<&T> {
        let pos = self.indices.binary_search(&index).ok()?;

        Some(&self.data[pos])
    }

    pub fn get_mut(&mut self, index: IndexType) -> Option<&mut T> {
        let pos = self.indices.binary_search(&index).ok()?;

        Some(&mut self.data[pos])
    }

    pub fn iter(&self) -> impl Iterator<Item = (&IndexType, &T)> {
        self.indices.iter().zip(self.data.iter())
    }
}

#[cfg(test)]
mod test {
    use alloc::{vec, vec::Vec};

    use super::SparseVec;

    #[test]
    fn test_insert() {
        let mut vec = SparseVec::new();

        vec.insert(0, 10);
        vec.insert(10, 12);
        vec.insert(5, 7);
        vec.insert(3, 1);

        assert_eq!(vec.iter().collect::<Vec<_>>(), vec![(&0, &10), (&3, &1), (&5, &7), (&10, &12)]);
    }

    #[test]
    fn test_get() {
        let mut vec = SparseVec::new();

        vec.insert(0, 10);
        vec.insert(10, 12);

        assert_eq!(*vec.get(0).unwrap(), 10);
        assert_eq!(*vec.get_mut(10).unwrap(), 12);

        assert_eq!(vec.get(1234), None);
    }
}
