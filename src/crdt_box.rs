use crate::CvRDT;
use std::ops::Deref;
use std::mem::replace;
use im::Vector;

#[derive(Clone)]
pub struct CrdtBox<T: CvRDT> {
    pub(crate) data: T,
    pub(crate) update: im::Vector<T::Update>,
}

impl<T: CvRDT> CrdtBox<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            update: im::Vector::new(),
        }
    }

    pub fn update(&mut self, update: T::Update) {
        self.data.update(update.clone());
        self.update.push_back(update);
    }

    pub(crate) fn drain_update(&mut self) -> im::Vector<T::Update> {
        replace(&mut self.update, Vector::new())
    }
}

impl<T: CvRDT> Deref for CrdtBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}