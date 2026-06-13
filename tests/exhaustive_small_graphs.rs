//! Exhaustive small-graph characterization test.
//!
//! This is the deterministic counterpart of the `graphlet_counts` fuzz target:
//! instead of sampling arbitrary graphs, it enumerates *every* undirected graph
//! on a small number of nodes (over several label assignments) and runs the
//! counter on each edge. Two things make it a strong regression and mutation
//! gate:
//!
//! * the crate's internal differential oracle runs on every edge (the test
//!   profile enables debug assertions), so an incorrect intermediate count
//!   panics; and
//! * an order-independent checksum of all produced graphlet counts is pinned to
//!   a golden value, so any change to the final counts is detected.

#![allow(
    missing_docs,
    missing_debug_implementations,
    unreachable_pub,
    clippy::unwrap_used,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::must_use_candidate
)]

use hashbrown::HashMap;
use heterogeneous_graphlets::prelude::*;
use std::collections::BTreeSet;

/// In-memory CSR graph with sorted neighbour lists.
struct MemGraph {
    offsets: Vec<usize>,
    edges: Vec<usize>,
    labels: Vec<u8>,
    num_labels: u8,
}

impl MemGraph {
    fn new(num_nodes: usize, edge_pairs: &[(usize, usize)], num_labels: u8) -> Self {
        let mut adjacency: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); num_nodes];
        for &(a, b) in edge_pairs {
            adjacency[a].insert(b);
            adjacency[b].insert(a);
        }
        let mut offsets = vec![0];
        let mut edges = Vec::new();
        for neighbours in &adjacency {
            edges.extend(neighbours.iter().copied());
            offsets.push(edges.len());
        }
        let labels = (0..num_nodes).map(|i| (i as u8) % num_labels).collect();
        Self {
            offsets,
            edges,
            labels,
            num_labels,
        }
    }
}

impl Graph for MemGraph {
    type NeighbourIter<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

    fn get_number_of_nodes(&self) -> usize {
        self.labels.len()
    }

    fn get_number_of_edges(&self) -> usize {
        self.edges.len()
    }

    fn iter_neighbours(&self, node: usize) -> Self::NeighbourIter<'_> {
        self.edges[self.offsets[node]..self.offsets[node + 1]]
            .iter()
            .copied()
    }
}

impl TypedGraph for MemGraph {
    type NodeLabel = u8;

    fn get_number_of_node_labels(&self) -> u8 {
        self.num_labels
    }

    fn get_number_of_node_labels_usize(&self) -> usize {
        self.num_labels as usize
    }

    fn get_node_label_from_usize(&self, label_index: usize) -> u8 {
        label_index as u8
    }

    fn get_node_label_index(&self, label: u8) -> usize {
        label as usize
    }

    fn get_node_label(&self, node: usize) -> u8 {
        self.labels[node]
    }
}

impl HeterogeneousGraphlets<u16, u32> for MemGraph {
    type GraphLetCounter = HashMap<u16, u32>;
}

/// Order-independent mix of a single (graphlet, count) entry.
fn mix(graphlet: u16, count: u32) -> u64 {
    let g = u64::from(graphlet).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let c = u64::from(count).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    let mut h = g ^ c.rotate_left(32);
    h ^= h >> 29;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^ (h >> 32)
}

/// Enumerates every undirected graph on `num_nodes` nodes, for label counts
/// `1..=max_labels`, and folds all graphlet counts into a single checksum.
fn checksum_over_all_graphs(num_nodes: usize, max_labels: u8) -> u64 {
    let pairs: Vec<(usize, usize)> = (0..num_nodes)
        .flat_map(|a| ((a + 1)..num_nodes).map(move |b| (a, b)))
        .collect();
    let mut checksum: u64 = 0;
    for mask in 0u32..(1u32 << pairs.len()) {
        let edges: Vec<(usize, usize)> = pairs
            .iter()
            .enumerate()
            .filter(|(i, _)| (mask >> i) & 1 == 1)
            .map(|(_, &p)| p)
            .collect();
        for num_labels in 1..=max_labels {
            let graph = MemGraph::new(num_nodes, &edges, num_labels);
            // Per-graph contribution is an order-independent sum over entries.
            let mut per_graph: u64 = 0;
            for src in 0..num_nodes {
                for dst in graph.iter_neighbours(src) {
                    if src < dst {
                        let counts = graph.get_heterogeneous_graphlet(src, dst);
                        for (graphlet, count) in &counts {
                            per_graph = per_graph.wrapping_add(mix(*graphlet, *count));
                        }
                    }
                }
            }
            // The outer loops are ordered, so this fold is deterministic.
            checksum = checksum.wrapping_mul(0x1_0000_01B3).wrapping_add(per_graph);
        }
    }
    checksum
}

/// Deterministic `SplitMix64` PRNG, so the sampled tests need no dependency and
/// produce a stable golden checksum.
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// Samples `num_samples` random undirected graphs on `num_nodes` nodes (random
/// density and 1..=3 labels) and folds all graphlet counts into a checksum.
/// Used where exhaustive enumeration is intractable (n >= 7 has up to 2^21..2^28
/// graphs).
fn checksum_over_sampled_graphs(num_nodes: usize, num_samples: u32, seed: u64) -> u64 {
    let pairs: Vec<(usize, usize)> = (0..num_nodes)
        .flat_map(|a| ((a + 1)..num_nodes).map(move |b| (a, b)))
        .collect();
    let mut rng = SplitMix64(seed);
    let mut checksum: u64 = 0;
    for _ in 0..num_samples {
        let num_labels = (rng.next() % 3) as u8 + 1;
        let density = rng.next() % 101;
        let edges: Vec<(usize, usize)> = pairs
            .iter()
            .filter(|_| rng.next() % 100 < density)
            .copied()
            .collect();
        let graph = MemGraph::new(num_nodes, &edges, num_labels);
        let mut per_graph: u64 = 0;
        for src in 0..num_nodes {
            for dst in graph.iter_neighbours(src) {
                if src < dst {
                    let counts = graph.get_heterogeneous_graphlet(src, dst);
                    for (graphlet, count) in &counts {
                        per_graph = per_graph.wrapping_add(mix(*graphlet, *count));
                    }
                }
            }
        }
        checksum = checksum.wrapping_mul(0x1_0000_01B3).wrapping_add(per_graph);
    }
    checksum
}

/// Two-node graph that merely *reports* a large label count, used to probe the
/// hash-capacity assertion at the entry of `get_heterogeneous_graphlet`.
struct WideGraph {
    num_labels: u8,
}

impl Graph for WideGraph {
    type NeighbourIter<'a> = std::iter::Once<usize>;

    fn get_number_of_nodes(&self) -> usize {
        2
    }

    fn get_number_of_edges(&self) -> usize {
        1
    }

    fn iter_neighbours(&self, node: usize) -> Self::NeighbourIter<'_> {
        std::iter::once(usize::from(node == 0))
    }
}

impl TypedGraph for WideGraph {
    type NodeLabel = u8;

    fn get_number_of_node_labels(&self) -> u8 {
        self.num_labels
    }

    fn get_number_of_node_labels_usize(&self) -> usize {
        usize::from(self.num_labels)
    }

    fn get_node_label_from_usize(&self, label_index: usize) -> u8 {
        label_index as u8
    }

    fn get_node_label_index(&self, label: u8) -> usize {
        usize::from(label)
    }

    fn get_node_label(&self, _node: usize) -> u8 {
        0
    }
}

impl HeterogeneousGraphlets<u32, u32> for WideGraph {
    type GraphLetCounter = HashMap<u32, u32>;
}

#[test]
fn hash_capacity_bound_is_exact() {
    // With 100 labels the correct maximal hash is 13 * 100^4 + 100^3 + 100^2 +
    // 100 = 1_301_010_100, below u32::MAX (4_294_967_295). A `+` -> `*` mutation
    // on the `n^2` term of the bound makes it n^3 * n^2 = n^5 = 10_000_000_000,
    // which exceeds u32::MAX and trips the capacity assertion, so this call must
    // succeed only for the unmutated formula.
    let graph = WideGraph { num_labels: 100 };
    let _ = graph.get_heterogeneous_graphlet(0, 1);
}

#[test]
fn exhaustive_five_node_graphs_match_golden() {
    // Every undirected graph on 5 nodes (2^10 = 1024) over 1..=3 labels, run
    // through the counter with the differential oracle live. Pins a checksum of
    // all produced counts as an exhaustive small-graph correctness guard.
    let checksum = checksum_over_all_graphs(5, 3);
    assert_eq!(
        checksum, 599_562_017_534_749_974,
        "five-node graphlet checksum changed"
    );
}

#[test]
fn sampled_seven_node_graphs_match_golden() {
    // Exhaustive enumeration is intractable at 7 nodes (2^21 graphs), so sample.
    let checksum = checksum_over_sampled_graphs(7, 20_000, 0x5EED_0007);
    assert_eq!(
        checksum, 17_578_673_648_220_574_958,
        "seven-node graphlet checksum changed"
    );
}

#[test]
fn sampled_eight_node_graphs_match_golden() {
    // Exhaustive enumeration is intractable at 8 nodes (2^28 graphs), so sample.
    let checksum = checksum_over_sampled_graphs(8, 20_000, 0x5EED_0008);
    assert_eq!(
        checksum, 17_140_356_329_161_639_925,
        "eight-node graphlet checksum changed"
    );
}
