use crate::util::CrdtCollection;
use crate::cvrdt::{CvRDT, CrdtBox};
use crate::crdt::{SmallVectorImpl, StableId};
use serde::__private::fmt::Debug;

#[derive(Clone)]
pub struct SmallVector<V: CrdtCollection + Debug> {
    inner: SmallVectorImpl<V>,
}

impl<V: CrdtCollection + Debug> SmallVector<V> {
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

    pub fn dbg_spans(&self) {
        self.inner.dbg_spans()
    }

    pub fn document(&self) -> &V {
        self.inner.document()
    }

    pub fn insert(&mut self, index: usize, element: V::Element) -> VectorUpdate<V::Element> {
        println!("\n---------------");
        dbg!(index);
        let previous = self.inner.position(index);
        dbg!(previous);
        let current = StableId::new(self.inner.author(), self.inner.next_local_id());

        self.inner.insert(previous, current, element.clone());

        dbg!(self.inner.document());
        self.inner.dbg_spans();

        VectorUpdate::Insert {
            previous: previous.stable(),
            current,
            element,
        }
    }

    pub fn delete(&mut self, index: usize) -> VectorUpdate<V::Element> {
        let position = self.inner.position(index);

        self.inner.delete(position);
        
        VectorUpdate::Delete {
            id: position.stable(),
        }
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

impl<V: CrdtCollection + Debug> CvRDT for SmallVector<V> {
    type Update = VectorUpdate<V::Element>;

    fn update(&mut self, update: Self::Update) {
        match update {
            VectorUpdate::Insert { previous, current, element } => {
                println!("\n---------------");
                dbg!(previous);
                let Local_previous = self.inner.stable_position(previous);
                dbg!(Local_previous);
                self.inner.insert(Local_previous, current, element);
                dbg!(self.inner.document());
                self.inner.dbg_spans();
            }
            VectorUpdate::Delete { id } => {
                let id = self.inner.stable_position(id);
                self.inner.delete(id);
            }
        }
    }
}

#[derive(Clone)]
pub enum VectorUpdate<T> {
    Insert {
        previous: StableId,
        current: StableId,
        element: T,
    },
    Delete {
        id: StableId,
    },
}

impl<V: CrdtCollection + Debug> CrdtBox<SmallVector<V>> {
    pub fn insert(&mut self, index: usize, value: V::Element) {
        let update = self.data.insert(index, value);
        self.update.push_back(update);
    }

    pub fn delete(&mut self, index: usize) {
        let update = self.data.delete(index);
        self.update.push_back(update);
    }
}