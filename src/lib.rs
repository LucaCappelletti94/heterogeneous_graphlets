//! Computation of heterogeneous (typed) graphlets.
//!
//! This crate implements the edge-centric typed-graphlet counting framework of
//! Rossi et al., "Heterogeneous Graphlets" (ACM TKDD 2020). Given a graph whose
//! nodes carry labels, it counts, for each edge, the typed 4-node graphlet
//! orbits incident to that edge.
//!
//! The entry point is the [`HeterogeneousGraphlets`] trait, implemented for any
//! type satisfying [`TypedGraph`]. Bring the public API into scope through the
//! [`prelude`] module.
//!
//! [`HeterogeneousGraphlets`]: crate::edge_typed_graphlets::HeterogeneousGraphlets
//! [`TypedGraph`]: crate::graph::TypedGraph
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

#[cfg(debug_assertions)]
mod debug_typed_graph;

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
