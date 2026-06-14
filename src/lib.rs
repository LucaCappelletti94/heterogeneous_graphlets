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
//! # Example
//!
//! Implement [`Graph`] and [`TypedGraph`] for your graph type, opt into
//! [`HeterogeneousGraphlets`] by choosing the integer types used for the
//! perfect-hash key and for the counts, then call
//! [`get_heterogeneous_graphlet`] on an edge. Neighbour lists must be sorted in
//! ascending order.
//!
//! ```
//! use heterogeneous_graphlets::prelude::*;
//! use hashbrown::HashMap;
//!
//! // A small undirected graph stored as per-node sorted adjacency lists, with
//! // one label (type) per node.
//! struct AdjacencyGraph {
//!     neighbours: Vec<Vec<usize>>,
//!     labels: Vec<u8>,
//!     number_of_labels: u8,
//! }
//!
//! impl Graph for AdjacencyGraph {
//!     type NeighbourIter<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;
//!
//!     fn get_number_of_nodes(&self) -> usize {
//!         self.labels.len()
//!     }
//!
//!     fn get_number_of_edges(&self) -> usize {
//!         self.neighbours.iter().map(Vec::len).sum()
//!     }
//!
//!     fn iter_neighbours(&self, node: usize) -> Self::NeighbourIter<'_> {
//!         self.neighbours[node].iter().copied()
//!     }
//! }
//!
//! impl TypedGraph for AdjacencyGraph {
//!     type NodeLabel = u8;
//!
//!     fn get_number_of_node_labels(&self) -> u8 {
//!         self.number_of_labels
//!     }
//!
//!     fn get_number_of_node_labels_usize(&self) -> usize {
//!         self.number_of_labels as usize
//!     }
//!
//!     fn get_node_label_from_usize(&self, label_index: usize) -> u8 {
//!         label_index as u8
//!     }
//!
//!     fn get_node_label_index(&self, label: u8) -> usize {
//!         label as usize
//!     }
//!
//!     fn get_node_label(&self, node: usize) -> u8 {
//!         self.labels[node]
//!     }
//! }
//!
//! // Pick `u32` for the perfect-hash key and `u32` for the counts, backed by a
//! // `hashbrown::HashMap` accumulator.
//! impl HeterogeneousGraphlets<u32, u32> for AdjacencyGraph {
//!     type GraphLetCounter = HashMap<u32, u32>;
//! }
//!
//! // A 4-clique on nodes {0, 1, 2, 3}, all sharing the single node label 0.
//! let graph = AdjacencyGraph {
//!     neighbours: vec![vec![1, 2, 3], vec![0, 2, 3], vec![0, 1, 3], vec![0, 1, 2]],
//!     labels: vec![0, 0, 0, 0],
//!     number_of_labels: 1,
//! };
//!
//! // Count the typed graphlet orbits incident to the edge (0, 1).
//! let counts = graph.get_heterogeneous_graphlet(0, 1);
//!
//! // Group the per-orbit counts by graphlet-kind name for inspection.
//! let by_kind =
//!     counts.to_graphlet_names::<ExtendedGraphletType, u8>(graph.get_number_of_node_labels());
//!
//! // The edge (0, 1) of a 4-clique lies in two triangles and one 4-clique.
//! assert_eq!(by_kind.get("Triangle"), Some(&2));
//! assert_eq!(by_kind.get("FourClique"), Some(&1));
//! ```
//!
//! [`HeterogeneousGraphlets`]: crate::edge_typed_graphlets::HeterogeneousGraphlets
//! [`get_heterogeneous_graphlet`]: crate::edge_typed_graphlets::HeterogeneousGraphlets::get_heterogeneous_graphlet
//! [`TypedGraph`]: crate::graph::TypedGraph
//! [`Graph`]: crate::graph::Graph
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
