use crate::util::CrdtCollection;
use crate::crdt::{SmallVectorImpl, StableId};
use crate::cmrdt::CmRDT;

#[derive(Clone)]
pub struct SmallVector<V: CrdtCollection> {
    inner: SmallVectorImpl<V>,
}

impl<V: CrdtCollection> SmallVector<V> {
    pub fn new(local_author: u16) -> Self {
        Self {
            inner: SmallVectorImpl::new(local_author)
        }
    }

    pub fn with_data(data: V, local_author: u16) -> Self {
        Self {
            inner: SmallVectorImpl::with_data(data, local_author)
        }
    }

    pub fn document(&self) -> &V {
        self.inner.document()
    }

    pub fn insert(&mut self, index: usize, element: V::Element) {
        let previous = self.inner.position(index);
        let current = StableId::new(self.inner.author(), self.inner.next_local_id());

        self.inner.insert(previous, current, element.clone());
    }

    pub fn delete(&mut self, index: usize) {
        let position = self.inner.position(index);

        self.inner.delete(position);
    }

    #[inline]
    pub fn stable_id(&self, index: usize) -> StableId {
        self.inner.position(index).stable()
    }

    #[inline]
    pub fn index(&self, id: StableId) -> usize {
        self.inner.stable_position(id).position()
    }
}

impl<V: CrdtCollection> CmRDT for SmallVector<V> {
    fn merge(&mut self, other: &Self) {
        self.inner.merge(&other.inner)
    }
}