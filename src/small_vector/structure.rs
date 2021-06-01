#[derive(Copy, Clone, Debug)]
pub(in crate::small_vector) struct Span {
    pub document_index: u32,
    pub length: u32,
    pub start_id: u32,
    pub author: u16,
    pub deleted: bool,
}

/// A simple collection trait for small_vector.
///
pub trait CrdtCollection<T: Clone> {
    fn new() -> Self;
    fn length(&self) -> usize;
    fn get(&self, index: usize) -> T;
    fn insert(&mut self, index: usize, value: T);
    fn remove(&mut self, index: usize);
}

impl<T: Clone> CrdtCollection<T> for im::Vector<T> {
    fn new() -> Self {
        Self::new()
    }

    fn length(&self) -> usize {
        self.len()
    }

    fn get(&self, index: usize) -> T {
        self[index].clone()
    }

    fn insert(&mut self, index: usize, value: T) {
        self.insert(index, value)
    }

    fn remove(&mut self, index: usize) {
        self.remove(index);
    }
}

impl CrdtCollection<char> for ropey::Rope {
    fn new() -> Self {
        Self::new()
    }

    fn length(&self) -> usize {
        self.len_chars()
    }

    fn get(&self, index: usize) -> char {
        self.char(index)
    }

    fn insert(&mut self, index: usize, value: char) {
        self.insert_char(index, value)
    }

    fn remove(&mut self, index: usize) {
        self.remove(index..=index)
    }
}