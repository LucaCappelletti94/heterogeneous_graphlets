#![doc = include_str!("../README.md")]
#![no_std]

#[macro_use]
extern crate alloc;

mod edge_typed_graphlets;
mod error;
pub mod graph;
mod graphlet_counter;
mod graphlet_set;
mod numbers;
mod orbits;
pub mod perfect_graphlet_hash;

#[cfg(any(test, feature = "oracle"))]
pub mod oracle;

/// Re-exports of the crate's public traits, types and errors.
pub mod prelude {
    pub use crate::edge_typed_graphlets::*;
    pub use crate::error::*;
    pub use crate::graph::*;
    pub use crate::graphlet_counter::*;
    pub use crate::graphlet_set::*;
}
