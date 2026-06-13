//! Independent brute-force reference counter for differential testing.
//!
//! This module deliberately re-derives typed-graphlet orbit counting from the
//! graphlet definitions by direct enumeration of 3- and 4-node induced
//! subgraphs, rather than from the fast algorithm. It is used by the
//! differential proptest tests and the fuzz harness to check that the fast
//! counter agrees with a simple, obviously-correct reference.
//!
//! Counting is done per graphlet *kind* (the twelve [`ExtendedGraphletType`]
//! edge orbits), ignoring node labels: summing the fast counter's typed counts
//! over all labels yields the same per-kind totals, so this validates the orbit
//! classification and the combinatorial counting without depending on the
//! perfect-hash label encoding (which the hash tests cover separately).

// This is differential-testing infrastructure; the graph-theory code reads most
// clearly with short mathematical names and boolean edge-presence flags.
#![allow(
    clippy::many_single_char_names,
    clippy::similar_names,
    clippy::too_long_first_doc_paragraph,
    clippy::fn_params_excessive_bools,
    clippy::format_push_string
)]

use crate::prelude::*;
use alloc::vec::Vec;
use hashbrown::HashMap;

/// Compressed-sparse-row graph with sorted neighbour lists, used as the subject
/// of differential tests.
#[derive(Debug, Clone)]
pub struct OracleGraph {
    offsets: Vec<usize>,
    edges: Vec<usize>,
    labels: Vec<u8>,
    num_labels: u8,
}

impl OracleGraph {
    /// Builds an undirected graph from `num_nodes`, an edge list (deduplicated
    /// and symmetrised), per-node labels and a label count. Out-of-range nodes
    /// are reduced modulo `num_nodes`, self-loops dropped, labels reduced modulo
    /// `num_labels`.
    #[must_use]
    pub fn new(
        num_nodes: usize,
        edge_pairs: &[(usize, usize)],
        node_labels: &[u8],
        num_labels: u8,
    ) -> Self {
        let num_labels = num_labels.max(1);
        let num_nodes = num_nodes.max(1);
        let mut adjacency: Vec<alloc::collections::BTreeSet<usize>> =
            alloc::vec![alloc::collections::BTreeSet::new(); num_nodes];
        for &(a, b) in edge_pairs {
            let a = a % num_nodes;
            let b = b % num_nodes;
            if a != b {
                adjacency[a].insert(b);
                adjacency[b].insert(a);
            }
        }
        let mut offsets = Vec::with_capacity(num_nodes + 1);
        let mut edges = Vec::new();
        offsets.push(0);
        for neighbours in &adjacency {
            edges.extend(neighbours.iter().copied());
            offsets.push(edges.len());
        }
        let labels = (0..num_nodes)
            .map(|i| node_labels.get(i).copied().unwrap_or(0) % num_labels)
            .collect();
        Self {
            offsets,
            edges,
            labels,
            num_labels,
        }
    }

    /// Iterates over the edges `(src, dst)` with `src < dst`.
    pub fn edges(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        (0..self.labels.len()).flat_map(move |src| {
            self.edges[self.offsets[src]..self.offsets[src + 1]]
                .iter()
                .copied()
                .filter(move |&dst| src < dst)
                .map(move |dst| (src, dst))
        })
    }
}

impl Graph for OracleGraph {
    type NeighbourIter<'a> = core::iter::Copied<core::slice::Iter<'a, usize>>;

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

impl TypedGraph for OracleGraph {
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

impl HeterogeneousGraphlets<u32, u64> for OracleGraph {
    type GraphLetCounter = HashMap<u32, u64>;
}

/// The fast counter's per-kind totals for edge `(i, j)`: runs the crate and sums
/// the typed counts per [`ExtendedGraphletType`] discriminant (decoding the kind
/// from the perfect hash).
#[must_use]
pub fn fast_per_kind_counts(graph: &OracleGraph, i: usize, j: usize) -> [u64; 12] {
    let counts = graph.get_heterogeneous_graphlet(i, j);
    let n = u64::from(graph.get_number_of_node_labels());
    let n4 = n * n * n * n;
    let mut totals = [0u64; 12];
    for (hash, count) in &counts {
        let kind = (u64::from(*hash) / n4) as usize;
        totals[kind] += count;
    }
    totals
}

/// Paper-faithful counts of the two 4-path orbits for edge `(i, j)`, following
/// the Table 3 definitions of Rossi et al.: `g3` (4-path edge, the queried edge
/// is an end edge) and `g4` (4-path center). Returns `(g3, g4)`.
#[must_use]
pub fn paper_four_path_counts<G: Graph>(graph: &G, i: usize, j: usize) -> (u64, u64) {
    let n = graph.get_number_of_nodes();
    let adjacent = |a: usize, b: usize| graph.iter_neighbours(a).any(|x| x == b);

    // S_i / S_j: exclusive neighbours; far (I): adjacent to neither i nor j.
    let s_i: Vec<usize> = (0..n)
        .filter(|&w| w != i && w != j && adjacent(i, w) && !adjacent(j, w))
        .collect();
    let s_j: Vec<usize> = (0..n)
        .filter(|&w| w != i && w != j && adjacent(j, w) && !adjacent(i, w))
        .collect();
    let far: Vec<usize> = (0..n)
        .filter(|&w| w != i && w != j && !adjacent(i, w) && !adjacent(j, w))
        .collect();

    // g3 (end edge): w_k exclusive to i or j, w_r a far node adjacent to w_k.
    let mut g3 = 0u64;
    for &w_k in s_i.iter().chain(s_j.iter()) {
        for &w_r in &far {
            if adjacent(w_k, w_r) {
                g3 += 1;
            }
        }
    }

    // g4 (center edge): w_k in S_i, w_r in S_j, not adjacent.
    let mut g4 = 0u64;
    for &w_k in &s_i {
        for &w_r in &s_j {
            if !adjacent(w_k, w_r) {
                g4 += 1;
            }
        }
    }

    (g3, g4)
}

/// Paper-faithful per-kind orbit counts for edge `(i, j)`, implementing every
/// Table 3 set definition of Rossi et al. directly. Returns an array indexed by
/// [`ExtendedGraphletType`] discriminant. Label-agnostic (structural counts).
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn paper_per_kind_counts<G: Graph>(graph: &G, i: usize, j: usize) -> [u64; 12] {
    let n = graph.get_number_of_nodes();
    let adj = |a: usize, b: usize| graph.iter_neighbours(a).any(|x| x == b);
    let s_i: Vec<usize> = (0..n)
        .filter(|&w| w != i && w != j && adj(i, w) && !adj(j, w))
        .collect();
    let s_j: Vec<usize> = (0..n)
        .filter(|&w| w != i && w != j && adj(j, w) && !adj(i, w))
        .collect();
    let t: Vec<usize> = (0..n)
        .filter(|&w| w != i && w != j && adj(i, w) && adj(j, w))
        .collect();
    let far: Vec<usize> = (0..n)
        .filter(|&w| w != i && w != j && !adj(i, w) && !adj(j, w))
        .collect();

    // Counts an ordered product P x Q (disjoint sets) where the pair is/ isn't an edge.
    let prod = |p: &[usize], q: &[usize], want_edge: bool| -> u64 {
        let mut c = 0u64;
        for &a in p {
            for &b in q {
                if adj(a, b) == want_edge {
                    c += 1;
                }
            }
        }
        c
    };
    // Counts unordered pairs within a set that are / aren't an edge.
    let pairs = |s: &[usize], want_edge: bool| -> u64 {
        let mut c = 0u64;
        for a in 0..s.len() {
            for b in (a + 1)..s.len() {
                if adj(s[a], s[b]) == want_edge {
                    c += 1;
                }
            }
        }
        c
    };

    let s_ij: Vec<usize> = s_i.iter().chain(&s_j).copied().collect();
    let mut counts = [0u64; 12];
    counts[kind::TRIAD] = (s_i.len() + s_j.len()) as u64;
    counts[kind::TRIANGLE] = t.len() as u64;
    // g3 4-path edge: w_k in S_i|S_j, w_r in I, adjacent.
    counts[kind::FOUR_PATH_EDGE] = prod(&s_ij, &far, true);
    // g4 4-path center: w_k in S_i, w_r in S_j, not adjacent.
    counts[kind::FOUR_PATH_CENTER] = prod(&s_i, &s_j, false);
    // g5 4-star: non-adjacent pair both exclusive to the same endpoint.
    counts[kind::FOUR_STAR] = pairs(&s_i, false) + pairs(&s_j, false);
    // g6 4-cycle: w_k in S_i, w_r in S_j, adjacent.
    counts[kind::FOUR_CYCLE] = prod(&s_i, &s_j, true);
    // g7 tailed-tri tail edge: adjacent pair both exclusive to the same endpoint.
    counts[kind::TAILED_TRI_TAIL] = pairs(&s_i, true) + pairs(&s_j, true);
    // g8 tailed-tri center: w_k in T, w_r in I, adjacent.
    counts[kind::TAILED_TRI_CENTER] = prod(&t, &far, true);
    // g9 tailed-tri tri-edge: w_k in T, w_r in S_i|S_j, not adjacent.
    counts[kind::TAILED_TRI_EDGE] = prod(&t, &s_ij, false);
    // g10 chordal edge: w_k in T, w_r in S_i|S_j, adjacent.
    counts[kind::CHORDAL_CYCLE_EDGE] = prod(&t, &s_ij, true);
    // g11 chordal center: non-adjacent pair both in T.
    counts[kind::CHORDAL_CYCLE_CENTER] = pairs(&t, false);
    // g12 4-clique: adjacent pair both in T.
    counts[kind::FOUR_CLIQUE] = pairs(&t, true);
    counts
}

/// Index of a [`ExtendedGraphletType`] kind in the per-kind arrays (its `u8`
/// discriminant). Kept as named constants for readability.
mod kind {
    pub(crate) const TRIAD: usize = 0;
    pub(crate) const TRIANGLE: usize = 1;
    pub(crate) const FOUR_PATH_EDGE: usize = 2;
    pub(crate) const FOUR_PATH_CENTER: usize = 3;
    pub(crate) const FOUR_STAR: usize = 4;
    pub(crate) const FOUR_CYCLE: usize = 5;
    pub(crate) const TAILED_TRI_TAIL: usize = 6;
    pub(crate) const TAILED_TRI_CENTER: usize = 7;
    pub(crate) const TAILED_TRI_EDGE: usize = 8;
    pub(crate) const CHORDAL_CYCLE_EDGE: usize = 9;
    pub(crate) const CHORDAL_CYCLE_CENTER: usize = 10;
    pub(crate) const FOUR_CLIQUE: usize = 11;
}

/// Brute-force reference: counts, by enumeration of induced subgraphs, the
/// per-kind orbit totals for edge `(i, j)`.
#[must_use]
pub fn reference_per_kind_counts<G: Graph>(graph: &G, i: usize, j: usize) -> [u64; 12] {
    let n = graph.get_number_of_nodes();
    let adjacent = |a: usize, b: usize| graph.iter_neighbours(a).any(|x| x == b);
    let mut counts = [0u64; 12];

    // 3-node graphlets: the third node k.
    for k in 0..n {
        if k == i || k == j {
            continue;
        }
        let ik = adjacent(i, k);
        let jk = adjacent(j, k);
        if ik && jk {
            counts[kind::TRIANGLE] += 1;
        } else if ik || jk {
            counts[kind::TRIAD] += 1;
        }
    }

    // 4-node graphlets: the two other nodes k < l.
    for k in 0..n {
        if k == i || k == j {
            continue;
        }
        for l in (k + 1)..n {
            if l == i || l == j {
                continue;
            }
            // Edges among {i, j, k, l}; (i, j) is always present.
            let a = adjacent(i, k);
            let b = adjacent(i, l);
            let c = adjacent(j, k);
            let d = adjacent(j, l);
            let e = adjacent(k, l);
            if let Some(idx) = classify_four_node(a, b, c, d, e) {
                counts[idx] += 1;
            }
        }
    }

    counts
}

/// Classifies the connected 4-node induced subgraph on `{i, j, k, l}` (where the
/// edge `(i, j)` is present and the five booleans are the other potential edges
/// `ik, il, jk, jl, kl`) into the orbit that edge `(i, j)` plays, or `None` if
/// the subgraph is disconnected.
fn classify_four_node(ik: bool, il: bool, jk: bool, jl: bool, kl: bool) -> Option<usize> {
    let edges = [ik, il, jk, jl, kl];
    let extra: u32 = edges.iter().map(|&x| u32::from(x)).sum();

    // Degrees within the 4-node subgraph (i and j always share their edge).
    let deg_i = 1 + u32::from(ik) + u32::from(il);
    let deg_j = 1 + u32::from(jk) + u32::from(jl);
    let deg_k = u32::from(ik) + u32::from(jk) + u32::from(kl);
    let deg_l = u32::from(il) + u32::from(jl) + u32::from(kl);

    if !is_connected_four_node(ik, il, jk, jl, kl) {
        return None;
    }

    match extra {
        // Tree (3 edges total): 4-path or 4-star.
        2 => {
            if deg_i == 3 || deg_j == 3 || deg_k == 3 || deg_l == 3 {
                Some(kind::FOUR_STAR)
            } else if deg_i == 2 && deg_j == 2 {
                Some(kind::FOUR_PATH_CENTER)
            } else {
                Some(kind::FOUR_PATH_EDGE)
            }
        }
        // 4 edges total: 4-cycle or tailed triangle (paw).
        3 => {
            if deg_i == 2 && deg_j == 2 && deg_k == 2 && deg_l == 2 {
                Some(kind::FOUR_CYCLE)
            } else if deg_i == 1 || deg_j == 1 {
                // (i, j) is the pendant (tail) edge.
                Some(kind::TAILED_TRI_TAIL)
            } else if deg_i == 3 || deg_j == 3 {
                // (i, j) is a triangle edge incident to the tail-attachment node.
                Some(kind::TAILED_TRI_EDGE)
            } else {
                // (i, j) is the triangle edge opposite the tail.
                Some(kind::TAILED_TRI_CENTER)
            }
        }
        // 5 edges total: diamond (K4 minus one edge).
        4 => {
            if deg_i == 3 && deg_j == 3 {
                Some(kind::CHORDAL_CYCLE_CENTER)
            } else {
                Some(kind::CHORDAL_CYCLE_EDGE)
            }
        }
        // 6 edges total: K4.
        5 => Some(kind::FOUR_CLIQUE),
        _ => None,
    }
}

/// Connectivity of the 4-node subgraph (local indices 0=i, 1=j, 2=k, 3=l), with
/// the `(0, 1)` edge always present.
fn is_connected_four_node(ik: bool, il: bool, jk: bool, jl: bool, kl: bool) -> bool {
    let adjacency = [
        [true, true, ik, il],
        [true, true, jk, jl],
        [ik, jk, true, kl],
        [il, jl, kl, true],
    ];
    let mut seen = [false; 4];
    let mut stack = alloc::vec![0usize];
    seen[0] = true;
    while let Some(node) = stack.pop() {
        for (other, &connected) in adjacency[node].iter().enumerate() {
            if connected && !seen[other] {
                seen[other] = true;
                stack.push(other);
            }
        }
    }
    seen.iter().all(|&s| s)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds the canonical graph for one orbit and checks the fast counter
    /// records exactly one occurrence of the expected kind for the named edge,
    /// locking down the orbit -> kind mapping the reference relies on.
    fn assert_single(edge_pairs: &[(usize, usize)], n: usize, i: usize, j: usize, kind: usize) {
        // Single real label (0) with a spare label, so decoding the kind from the
        // perfect hash is exact (the 3-node dummy label of `num_labels` stays
        // below `num_labels^4`).
        let graph = OracleGraph::new(n, edge_pairs, &[0; 16], 2);
        let fast = fast_per_kind_counts(&graph, i, j);
        assert_eq!(fast[kind], 1, "fast counter, kind {kind}: {fast:?}");
        let reference = reference_per_kind_counts(&graph, i, j);
        assert_eq!(reference[kind], 1, "reference, kind {kind}: {reference:?}");
        assert_eq!(fast, reference, "fast vs reference mismatch");
    }

    #[test]
    fn calibrate_triangle() {
        // Triangle 0-1-2 on edge (0,1).
        assert_single(&[(0, 1), (1, 2), (0, 2)], 3, 0, 1, kind::TRIANGLE);
    }

    #[test]
    fn calibrate_triad() {
        // Path 2-0-1: edge (0,1) is an end edge of a 3-path.
        assert_single(&[(0, 1), (0, 2)], 3, 0, 1, kind::TRIAD);
    }

    #[test]
    fn calibrate_four_clique() {
        let k4 = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
        assert_single(&k4, 4, 0, 1, kind::FOUR_CLIQUE);
    }

    #[test]
    fn calibrate_four_cycle() {
        // Cycle 0-1-3-2-0; edge (0,1) is a cycle edge.
        assert_single(&[(0, 1), (1, 3), (3, 2), (2, 0)], 4, 0, 1, kind::FOUR_CYCLE);
    }

    #[test]
    fn calibrate_four_star() {
        // Star centered at 0 with leaves 1,2,3; edge (0,1) is a spoke.
        assert_single(&[(0, 1), (0, 2), (0, 3)], 4, 0, 1, kind::FOUR_STAR);
    }

    #[test]
    fn lone_four_path_end_edge_counts_one() {
        // Path 0-1-2-3. Edge (0,1) is an end edge: paper f_01(g3) = 1.
        let graph = OracleGraph::new(4, &[(0, 1), (1, 2), (2, 3)], &[0; 4], 2);
        let (paper_g3, _paper_g4) = paper_four_path_counts(&graph, 0, 1);
        assert_eq!(paper_g3, 1, "paper reference");
        let fast = fast_per_kind_counts(&graph, 0, 1);
        assert_eq!(
            fast[kind::FOUR_PATH_EDGE],
            1,
            "crate FourPathEdge: {fast:?}"
        );
    }

    #[test]
    fn crate_matches_paper_reference_extent() {
        // Deterministic random graphs; tally per-orbit crate vs paper totals to
        // map the full extent of any divergence.
        let mut state = 0x1234_5678_9abc_def0u64;
        let mut next = || {
            state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        };

        let mut crate_total = [0u64; 12];
        let mut paper_total = [0u64; 12];
        for _ in 0..2000 {
            let num_nodes = 5 + (next() % 3) as usize; // 5..=7
            let mut edges = Vec::new();
            for a in 0..num_nodes {
                for b in (a + 1)..num_nodes {
                    if next() % 2 == 0 {
                        edges.push((a, b));
                    }
                }
            }
            let graph = OracleGraph::new(num_nodes, &edges, &[0; 8], 2);
            for (i, j) in graph.edges() {
                let fast = fast_per_kind_counts(&graph, i, j);
                let paper = paper_per_kind_counts(&graph, i, j);
                for k in 0..12 {
                    crate_total[k] += fast[k];
                    paper_total[k] += paper[k];
                }
            }
        }

        let names = [
            "Triad",
            "Triangle",
            "FourPathEdge",
            "FourPathCenter",
            "FourStar",
            "FourCycle",
            "TailedTriTail",
            "TailedTriCenter",
            "TailedTriEdge",
            "ChordalCycleEdge",
            "ChordalCycleCenter",
            "FourClique",
        ];
        let mut report = alloc::string::String::new();
        for k in 0..12 {
            if crate_total[k] != paper_total[k] {
                report.push_str(&alloc::format!(
                    "\n  {}: crate={} paper={}",
                    names[k],
                    crate_total[k],
                    paper_total[k]
                ));
            }
        }
        assert!(
            report.is_empty(),
            "orbits diverging (crate vs paper):{report}"
        );
    }

    #[test]
    fn four_path_edge_center_identity() {
        // Across all edges, total(4-path-edge) must equal 2 * total(4-path-center)
        // (each induced 4-path has two end edges and one centre edge).
        let graphs = [
            OracleGraph::new(4, &[(0, 1), (1, 2), (2, 3)], &[0; 8], 2),
            OracleGraph::new(5, &[(0, 1), (1, 2), (2, 3), (3, 4), (1, 4)], &[0; 8], 2),
            OracleGraph::new(
                5,
                &[(0, 1), (0, 2), (0, 3), (0, 4), (1, 2), (3, 4)],
                &[0; 8],
                2,
            ),
        ];
        for graph in &graphs {
            let mut total_edge = 0u64;
            let mut total_center = 0u64;
            for (i, j) in graph.edges() {
                let fast = fast_per_kind_counts(graph, i, j);
                total_edge += fast[kind::FOUR_PATH_EDGE];
                total_center += fast[kind::FOUR_PATH_CENTER];
            }
            assert_eq!(
                total_edge,
                2 * total_center,
                "identity violated: edge={total_edge}, center={total_center}"
            );
        }
    }

    use proptest::prelude::*;

    /// Strategy generating an arbitrary small single-label graph.
    fn arbitrary_graph() -> impl Strategy<Value = OracleGraph> {
        (2usize..=8).prop_flat_map(|num_nodes| {
            proptest::collection::vec((0..num_nodes, 0..num_nodes), 0..=num_nodes * num_nodes)
                .prop_map(move |edges| OracleGraph::new(num_nodes, &edges, &[], 2))
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        /// The fast counter must match the paper-faithful reference for every edge
        /// of every generated graph. On failure proptest shrinks to a minimal
        /// counterexample graph.
        #[test]
        fn fast_counter_matches_paper_reference(graph in arbitrary_graph()) {
            for (i, j) in graph.edges() {
                prop_assert_eq!(
                    fast_per_kind_counts(&graph, i, j),
                    paper_per_kind_counts(&graph, i, j),
                    "edge ({}, {})", i, j
                );
            }
        }
    }
}
