//! The module contains a list CRDT for small elements like strings.
//!
//!
//!

mod cmrdt;
mod cvrdt;
mod structure;

pub use cvrdt::{Vector as CvrdtVector, VectorUpdate};
pub use structure::CrdtCollection;
