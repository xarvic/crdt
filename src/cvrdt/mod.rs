//!
//!
//!
//!
//!
//!

mod small_vec;
mod crdt_box;

pub use small_vec::{SmallVector, VectorUpdate};
pub use crdt_box::CrdtBox;

///
///
///
#[allow(clippy::upper_case_acronyms)]
pub trait CvRDT {
    type Update: Clone;

    fn update(&mut self, update: Self::Update);
}

