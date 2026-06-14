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
//!   panics, and
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

/// In-memory CSR graph with sorted neighbour lists and a deterministic edge
/// colouring, used to exercise edge-coloured counting. The colour of edge
/// `(a, b)` is `(a + b) % num_edge_labels`, which is symmetric and reproducible.
struct EdgeMemGraph {
    offsets: Vec<usize>,
    edges: Vec<usize>,
    labels: Vec<u8>,
    num_labels: u8,
    num_edge_labels: u8,
}

impl EdgeMemGraph {
    fn new(
        num_nodes: usize,
        edge_pairs: &[(usize, usize)],
        num_labels: u8,
        num_edge_labels: u8,
    ) -> Self {
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
            num_edge_labels,
        }
    }
}

impl Graph for EdgeMemGraph {
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

impl TypedGraph for EdgeMemGraph {
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

impl EdgeTypedGraph for EdgeMemGraph {
    type EdgeLabel = u8;

    fn get_number_of_edge_labels(&self) -> u8 {
        self.num_edge_labels
    }

    fn get_number_of_edge_labels_usize(&self) -> usize {
        self.num_edge_labels as usize
    }

    fn get_edge_label_from_usize(&self, label_index: usize) -> u8 {
        label_index as u8
    }

    fn get_edge_label_index(&self, label: u8) -> usize {
        label as usize
    }

    fn get_edge_label(&self, src: usize, dst: usize) -> u8 {
        ((src + dst) % usize::from(self.num_edge_labels)) as u8
    }
}

impl EdgeTypedGraphlets<u64, u64> for EdgeMemGraph {
    type GraphLetCounter = HashMap<u64, u64>;
}

/// Order-independent mix of a single (edge-typed key, count) entry.
fn mix64(key: u64, count: u64) -> u64 {
    let g = key.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let c = count.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    let mut h = g ^ c.rotate_left(32);
    h ^= h >> 29;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^ (h >> 32)
}

/// Enumerates every undirected graph on `num_nodes` nodes, for node-label counts
/// `1..=max_labels` and the given `num_edge_labels`, and folds all edge-coloured
/// graphlet counts into a single checksum.
fn edge_checksum_over_all_graphs(num_nodes: usize, max_labels: u8, num_edge_labels: u8) -> u64 {
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
            let graph = EdgeMemGraph::new(num_nodes, &edges, num_labels, num_edge_labels);
            let mut per_graph: u64 = 0;
            for src in 0..num_nodes {
                for dst in graph.iter_neighbours(src) {
                    if src < dst {
                        let counts = graph.get_edge_typed_graphlet(src, dst).unwrap();
                        for (key, count) in &counts {
                            per_graph = per_graph.wrapping_add(mix64(*key, *count));
                        }
                    }
                }
            }
            checksum = checksum.wrapping_mul(0x1_0000_01B3).wrapping_add(per_graph);
        }
    }
    checksum
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
                        let counts = graph.get_heterogeneous_graphlet(src, dst).unwrap();
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
                    let counts = graph.get_heterogeneous_graphlet(src, dst).unwrap();
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
fn hash_capacity_accepts_largest_fitting_label_count() {
    // The hash base is the label count plus one (a reserved 3-node sentinel
    // digit). The maximal hash is 13 * base^4 + base^3 + base^2 + base. The
    // largest label count whose hash fits a `u32` (max 4_294_967_295) is 133:
    // base 134 gives 13 * 134^4 + 134^3 + 134^2 + 134 = 4_193_857_362 <= u32::MAX.
    // This call must therefore return Ok. Sitting exactly on the boundary, any
    // mutation that grows the bound past u32::MAX here would wrongly return an
    // error and fail this test.
    let graph = WideGraph { num_labels: 133 };
    assert!(graph.get_heterogeneous_graphlet(0, 1).is_ok());
}

#[test]
fn hash_capacity_rejects_one_label_too_many() {
    // One colour more (134, base 135) overflows a `u32`: 13 * 135^4 + ... =
    // 4_317_958_125 > u32::MAX, so the encodability check must return an error.
    // Sitting exactly one past the boundary, any mutation that shrinks the bound
    // below u32::MAX here would wrongly accept it (a silent overflow), so this
    // test fails for such mutations.
    let graph = WideGraph { num_labels: 134 };
    assert!(matches!(
        graph.get_heterogeneous_graphlet(0, 1),
        Err(GraphletError::GraphletKeyTooSmall { .. })
    ));
}

#[test]
fn exhaustive_five_node_graphs_match_golden() {
    // Every undirected graph on 5 nodes (2^10 = 1024) over 1..=3 labels, run
    // through the counter with the differential oracle live. Pins a checksum of
    // all produced counts as an exhaustive small-graph correctness guard.
    let checksum = checksum_over_all_graphs(5, 3);
    assert_eq!(
        checksum, 14_460_882_348_754_391_394,
        "five-node graphlet checksum changed"
    );
}

#[test]
fn sampled_seven_node_graphs_match_golden() {
    // Exhaustive enumeration is intractable at 7 nodes (2^21 graphs), so sample.
    let checksum = checksum_over_sampled_graphs(7, 20_000, 0x5EED_0007);
    assert_eq!(
        checksum, 12_488_835_846_226_054_564,
        "seven-node graphlet checksum changed"
    );
}

#[test]
fn sampled_eight_node_graphs_match_golden() {
    // Exhaustive enumeration is intractable at 8 nodes (2^28 graphs), so sample.
    let checksum = checksum_over_sampled_graphs(8, 20_000, 0x5EED_0008);
    assert_eq!(
        checksum, 12_635_832_868_487_958_678,
        "eight-node graphlet checksum changed"
    );
}

/// Samples random undirected graphs on `num_nodes` nodes (random density,
/// 1..=3 node labels, the given edge-colour count) and folds all edge-coloured
/// graphlet counts into a checksum, for sizes where exhaustive enumeration is
/// intractable.
fn edge_checksum_over_sampled_graphs(
    num_nodes: usize,
    num_samples: u32,
    num_edge_labels: u8,
    seed: u64,
) -> u64 {
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
        let graph = EdgeMemGraph::new(num_nodes, &edges, num_labels, num_edge_labels);
        let mut per_graph: u64 = 0;
        for src in 0..num_nodes {
            for dst in graph.iter_neighbours(src) {
                if src < dst {
                    let counts = graph.get_edge_typed_graphlet(src, dst).unwrap();
                    for (key, count) in &counts {
                        per_graph = per_graph.wrapping_add(mix64(*key, *count));
                    }
                }
            }
        }
        checksum = checksum.wrapping_mul(0x1_0000_01B3).wrapping_add(per_graph);
    }
    checksum
}

/// Order-independent mix of a single deduplicated `(pattern, count)` entry, where
/// the pattern is `(kind, four node colours, six edge colours)` with `None` for an
/// absent node or edge (folded as the byte 255).
#[allow(clippy::type_complexity)]
fn mix_dedup(pattern: &(ReducedGraphletType, [Option<u8>; 4], [Option<u8>; 6]), count: u64) -> u64 {
    let (reduced, nodes, edges) = pattern;
    let mut h = u64::from(u8::from(*reduced)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for slot in nodes {
        h = (h ^ u64::from(slot.unwrap_or(255))).wrapping_mul(0x1_0000_01B3);
    }
    for slot in edges {
        h = (h ^ u64::from(slot.unwrap_or(255))).wrapping_mul(0x1_0000_01B3);
    }
    let c = count.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    let mut out = h ^ c.rotate_left(32);
    out ^= out >> 29;
    out = out.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    out ^ (out >> 32)
}

/// Whole-graph signature contribution: the over-counted signature folded as an
/// order-independent sum over its `(key, count)` entries.
fn signature_contribution(graph: &EdgeMemGraph) -> u64 {
    let mut per_graph: u64 = 0;
    for (key, count) in &graph.get_edge_typed_graph_signature().unwrap() {
        per_graph = per_graph.wrapping_add(mix64(*key, *count));
    }
    per_graph
}

/// Whole-graph exact-count contribution: the deduplicated counts folded as an
/// order-independent sum over their `(pattern, count)` entries.
fn dedup_contribution(graph: &EdgeMemGraph) -> u64 {
    let mut per_graph: u64 = 0;
    for (pattern, count) in &graph.get_edge_typed_graphlet_counts().unwrap() {
        per_graph = per_graph.wrapping_add(mix_dedup(pattern, *count));
    }
    per_graph
}

/// Enumerates every undirected graph on `num_nodes` nodes (node-label counts
/// `1..=max_labels`, the given edge-colour count) and folds `per_graph` over all of
/// them into one checksum.
fn whole_graph_checksum_over_all_graphs(
    num_nodes: usize,
    max_labels: u8,
    num_edge_labels: u8,
    per_graph: impl Fn(&EdgeMemGraph) -> u64,
) -> u64 {
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
            let graph = EdgeMemGraph::new(num_nodes, &edges, num_labels, num_edge_labels);
            checksum = checksum
                .wrapping_mul(0x1_0000_01B3)
                .wrapping_add(per_graph(&graph));
        }
    }
    checksum
}

/// Samples random undirected graphs on `num_nodes` nodes and folds `per_graph`
/// over them into one checksum, for sizes where exhaustive enumeration is
/// intractable.
fn whole_graph_checksum_over_sampled_graphs(
    num_nodes: usize,
    num_samples: u32,
    num_edge_labels: u8,
    seed: u64,
    per_graph: impl Fn(&EdgeMemGraph) -> u64,
) -> u64 {
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
        let graph = EdgeMemGraph::new(num_nodes, &edges, num_labels, num_edge_labels);
        checksum = checksum
            .wrapping_mul(0x1_0000_01B3)
            .wrapping_add(per_graph(&graph));
    }
    checksum
}

#[test]
fn exhaustive_five_node_edge_typed_signature_matches_golden() {
    // Every undirected graph on 5 nodes over 1..=3 node labels with 2 edge colours,
    // folding the whole-graph (over-counted) edge-coloured signature.
    let checksum = whole_graph_checksum_over_all_graphs(5, 3, 2, signature_contribution);
    assert_eq!(
        checksum, 9_859_388_273_492_615_129,
        "five-node edge-typed signature checksum changed"
    );
}

#[test]
fn sampled_seven_node_edge_typed_signature_matches_golden() {
    let checksum =
        whole_graph_checksum_over_sampled_graphs(7, 20_000, 3, 0x5EED_5167, signature_contribution);
    assert_eq!(
        checksum, 4_907_165_915_009_204_232,
        "seven-node edge-typed signature checksum changed"
    );
}

#[test]
fn exhaustive_five_node_edge_typed_dedup_matches_golden() {
    // Same enumeration, folding the exact deduplicated per-pattern occurrence counts.
    let checksum = whole_graph_checksum_over_all_graphs(5, 3, 2, dedup_contribution);
    assert_eq!(
        checksum, 3_623_995_928_385_104_678,
        "five-node edge-typed dedup checksum changed"
    );
}

#[test]
fn sampled_seven_node_edge_typed_dedup_matches_golden() {
    let checksum =
        whole_graph_checksum_over_sampled_graphs(7, 20_000, 3, 0x5EED_D3D0, dedup_contribution);
    assert_eq!(
        checksum, 3_946_770_340_059_495_689,
        "seven-node edge-typed dedup checksum changed"
    );
}

#[test]
fn exhaustive_five_node_edge_typed_graphs_match_golden() {
    // Every undirected graph on 5 nodes (2^10 = 1024) over 1..=3 node labels with
    // 2 edge colours, run through the edge-coloured counter (its internal oracle
    // live under the test profile). Pins a checksum of all produced edge-coloured
    // counts as an exhaustive small-graph correctness guard.
    let checksum = edge_checksum_over_all_graphs(5, 3, 2);
    assert_eq!(
        checksum, 12_554_057_458_392_349_420,
        "five-node edge-typed graphlet checksum changed"
    );
}

#[test]
fn sampled_seven_node_edge_typed_graphs_match_golden() {
    // Exhaustive enumeration is intractable at 7 nodes, so sample, with 3 edge
    // colours to exercise a wider edge-colour range.
    let checksum = edge_checksum_over_sampled_graphs(7, 20_000, 3, 0x5EED_E007);
    assert_eq!(
        checksum, 10_821_814_031_900_718_787,
        "seven-node edge-typed graphlet checksum changed"
    );
}
