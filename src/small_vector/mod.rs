//! The module contains a list CRDT for small elements like strings.
//!
//!
//!

mod structure;
mod cvrdt;
mod cmrdt;

pub use cvrdt::{Vector as CvrdtVector, VectorUpdate};
pub use structure::CrdtCollection;