use std::ops::{Index, IndexMut};

pub(crate) struct UninitializedCollection<T> {
    collection: T,
}

impl <T> UninitializedCollection<T> {
    pub fn new(collection: T) -> Self {
        UninitializedCollection {
            collection,
        }
    }

    pub fn into_inner(self) -> T {
        self.collection
    }
}

impl <T: IndexMut<I, Output=Option<O>>, I, O> Index<I> for UninitializedCollection<T> {
    type Output = O;

    fn index(&self, key: I) -> &Self::Output {
        &self.collection[key].as_ref().expect("unitialized")
    }
}

impl <T: IndexMut<I, Output=Option<O>>, I, O: Default> IndexMut<I> for UninitializedCollection<T> {
    fn index_mut(&mut self, key: I) -> &mut Self::Output {
        let val = &mut self.collection[key];
        if val.is_none() {
            *val = Some(Default::default())
        }
        val.as_mut().unwrap()
    }
}

pub(crate) struct OffsetCollection<T: IndexMut<usize>> {
    collection: T,
    offset: i64,
}

impl <T: IndexMut<usize>> OffsetCollection<T> {
    pub fn new(collection: T, offset: i64) -> Self {
        OffsetCollection {
            collection,
            offset,
        }
    }

    pub fn into_inner(self) -> T {
        self.collection
    }
}

impl <T: IndexMut<usize>> Index<i64> for OffsetCollection<T> {
    type Output = <T as Index<usize>>::Output;

    fn index(&self, key: i64) -> &Self::Output {
        &self.collection[(key + self.offset) as usize]
    }
}

impl <T: IndexMut<usize>> IndexMut<i64> for OffsetCollection<T> {
    fn index_mut(&mut self, key: i64) -> &mut Self::Output {
        &mut self.collection[(key + self.offset) as usize]
    }
}
