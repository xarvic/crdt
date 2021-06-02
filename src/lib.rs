//TODO: remove this
#![allow(dead_code)]
//!
//!
//!
//!

// The modules which contain the public types
pub mod cvrdt;
pub mod cmrdt;

// The module which contains the common data structures.
mod crdt;

// The module which contains basic structures needed by the CRDTs.
pub mod util;

#[cfg(test)]
mod tests;
