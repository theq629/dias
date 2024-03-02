use std::marker::PhantomData;

pub struct ArgId<T> {
    _phantom: PhantomData<T>,
    pub id: usize,
}

impl<T> ArgId<T> {
    pub fn new(id: usize) -> Self {
        Self {
            _phantom: PhantomData,
            id,
        }
    }
}
