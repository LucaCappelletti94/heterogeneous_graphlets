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

// This is differential-testing infrastructure, and the graph-theory code reads most
// clearly with short mathematical names and boolean edge-presence flags.
#![allow(
    clippy::many_single_char_names,
    clippy::similar_names,
    clippy::too_long_first_doc_paragraph,
    clippy::fn_params_excessive_bools,
    clippy::format_push_string,
    // This is test infrastructure whose graphs always use small label counts and
    // wide enough keys, so the encodability check never fails here.
    clippy::unwrap_used,
    clippy::missing_panics_doc
)]

use crate::perfect_graphlet_hash::{canonical_descriptor, decode_edge_typed};
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
    /// Edge colours keyed by the sorted endpoint pair `(min, max)`. Empty for
    /// node-only graphs built with [`OracleGraph::new`].
    edge_labels: alloc::collections::BTreeMap<(usize, usize), u8>,
    num_edge_labels: u8,
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
            edge_labels: alloc::collections::BTreeMap::new(),
            num_edge_labels: 1,
        }
    }

    /// Builds an edge-coloured graph: like [`OracleGraph::new`], plus a colour per
    /// entry of `edge_pairs` (parallel slice `edge_colours`, missing entries
    /// default to 0, reduced modulo `num_edge_labels`). Colours are stored by the
    /// sorted endpoint pair, so the graph stays undirected.
    #[must_use]
    pub fn new_edge_typed(
        num_nodes: usize,
        edge_pairs: &[(usize, usize)],
        edge_colours: &[u8],
        node_labels: &[u8],
        num_labels: u8,
        num_edge_labels: u8,
    ) -> Self {
        let mut graph = Self::new(num_nodes, edge_pairs, node_labels, num_labels);
        let num_edge_labels = num_edge_labels.max(1);
        let num_nodes = graph.labels.len();
        let mut edge_labels = alloc::collections::BTreeMap::new();
        for (idx, &(a, b)) in edge_pairs.iter().enumerate() {
            let a = a % num_nodes;
            let b = b % num_nodes;
            if a != b {
                let colour = edge_colours.get(idx).copied().unwrap_or(0) % num_edge_labels;
                edge_labels.insert((a.min(b), a.max(b)), colour);
            }
        }
        graph.edge_labels = edge_labels;
        graph.num_edge_labels = num_edge_labels;
        graph
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

    /// The number of edge colours (1 for node-only graphs).
    #[must_use]
    pub fn get_number_of_edge_labels(&self) -> u8 {
        self.num_edge_labels
    }

    /// The colour of edge `(a, b)` (order-independent), or 0 if uncoloured.
    #[must_use]
    pub fn get_edge_colour(&self, a: usize, b: usize) -> u8 {
        self.edge_labels
            .get(&(a.min(b), a.max(b)))
            .copied()
            .unwrap_or(0)
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

impl EdgeTypedGraph for OracleGraph {
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
        self.get_edge_colour(src, dst)
    }
}

impl EdgeTypedGraphlets<u32, u64> for OracleGraph {
    type GraphLetCounter = HashMap<u32, u64>;
}

/// The crate's edge-coloured output for edge `(i, j)`, decoded into a map from
/// `(kind, canonical node labels, canonical edge colours)` to count, matching the
/// key shape of [`paper_edge_typed_counts`] for differential comparison.
#[must_use]
pub fn crate_edge_typed_counts(
    graph: &OracleGraph,
    i: usize,
    j: usize,
) -> alloc::collections::BTreeMap<(u8, [u8; 4], [u8; 6]), u64> {
    let counts = graph.get_edge_typed_graphlet(i, j).unwrap();
    let node_base = u32::from(graph.get_number_of_node_labels()) + 1;
    let edge_base = u32::from(EdgeTypedGraph::get_number_of_edge_labels(graph)) + 1;
    let mut map = alloc::collections::BTreeMap::new();
    for (key, count) in &counts {
        let (kind, nodes, edges) = decode_edge_typed::<u32>(*key, node_base, edge_base);
        let nodes = core::array::from_fn(|k| nodes[k] as u8);
        let edges = core::array::from_fn(|k| edges[k] as u8);
        *map.entry((kind as u8, nodes, edges)).or_insert(0u64) += *count;
    }
    map
}

/// The fast counter's per-kind totals for edge `(i, j)`: runs the crate and sums
/// the typed counts per [`ExtendedGraphletType`] discriminant (decoding the kind
/// from the perfect hash).
#[must_use]
pub fn fast_per_kind_counts(graph: &OracleGraph, i: usize, j: usize) -> [u64; 12] {
    let counts = graph.get_heterogeneous_graphlet(i, j).unwrap();
    // The perfect hash uses base `number_of_labels + 1` (one reserved sentinel
    // digit), so the kind occupies the `base^4` place.
    let base = u64::from(graph.get_number_of_node_labels()) + 1;
    let base4 = base * base * base * base;
    let mut totals = [0u64; 12];
    for (hash, count) in &counts {
        let kind = (u64::from(*hash) / base4) as usize;
        totals[kind] += count;
    }
    totals
}

/// The crate's full typed output for edge `(i, j)`, decoded into a map from
/// `(kind, sorted node labels)` to count. The label decode is kind-aware and uses
/// the perfect hash's base of `n + 1`: 4-node graphlets carry four real labels,
/// while 3-node graphlets carry three real labels plus the `sentinel = n` digit,
/// which is removed before decoding.
#[must_use]
pub fn crate_typed_counts(
    graph: &OracleGraph,
    i: usize,
    j: usize,
) -> alloc::collections::BTreeMap<(u8, Vec<u8>), u64> {
    let counts = graph.get_heterogeneous_graphlet(i, j).unwrap();
    // The perfect hash uses a positional base of `n + 1`, where one digit value
    // (`= n`) is reserved as the sentinel that 3-node graphlets store in their
    // 4th position. Decoding mirrors that base, and the sentinel is stripped before
    // recovering the three real labels of a 3-node graphlet.
    let n = u32::from(graph.get_number_of_node_labels());
    let base = n + 1;
    let (base2, base3, base4) = (base * base, base * base * base, base * base * base * base);
    let mut map = alloc::collections::BTreeMap::new();
    for (hash, count) in &counts {
        let hash = *hash;
        let kind = (hash / base4) as u8;
        let rem = hash % base4;
        let mut labels: Vec<u8> = if kind <= 1 {
            // 3-node: rem = la*base^3 + lb*base^2 + lc*base + sentinel(n).
            let stripped = (rem - n) / base;
            alloc::vec![
                (stripped / base2) as u8,
                (stripped / base % base) as u8,
                (stripped % base) as u8
            ]
        } else {
            alloc::vec![
                (rem / base3) as u8,
                (rem / base2 % base) as u8,
                (rem / base % base) as u8,
                (rem % base) as u8,
            ]
        };
        labels.sort_unstable();
        *map.entry((kind, labels)).or_default() += count;
    }
    map
}

/// Paper-faithful typed counts for edge `(i, j)`: the same orbit enumeration as
/// [`paper_per_kind_counts`], but recording each occurrence keyed by its kind and
/// the sorted labels of its nodes, so the per-label (heterogeneous) counting can
/// be validated against the crate.
#[must_use]
pub fn paper_typed_counts(
    graph: &OracleGraph,
    i: usize,
    j: usize,
) -> alloc::collections::BTreeMap<(u8, Vec<u8>), u64> {
    let n = graph.get_number_of_nodes();
    let adj = |a: usize, b: usize| graph.iter_neighbours(a).any(|x| x == b);
    let lab = |x: usize| graph.get_node_label(x);
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
    let s_ij: Vec<usize> = s_i.iter().chain(&s_j).copied().collect();

    let mut map = alloc::collections::BTreeMap::new();
    let emit3 = |map: &mut alloc::collections::BTreeMap<(u8, Vec<u8>), u64>, kind: u8, w: usize| {
        let mut labels = alloc::vec![lab(i), lab(j), lab(w)];
        labels.sort_unstable();
        *map.entry((kind, labels)).or_default() += 1;
    };
    for &w in &s_i {
        emit3(&mut map, kind::TRIAD as u8, w);
    }
    for &w in &s_j {
        emit3(&mut map, kind::TRIAD as u8, w);
    }
    for &w in &t {
        emit3(&mut map, kind::TRIANGLE as u8, w);
    }

    let emit4 = |map: &mut alloc::collections::BTreeMap<(u8, Vec<u8>), u64>,
                 kind: usize,
                 a: usize,
                 b: usize| {
        let mut labels = alloc::vec![lab(i), lab(j), lab(a), lab(b)];
        labels.sort_unstable();
        *map.entry((kind as u8, labels)).or_default() += 1;
    };
    let product = |map: &mut _, kind, p: &[usize], q: &[usize], want_edge: bool| {
        for &a in p {
            for &b in q {
                if adj(a, b) == want_edge {
                    emit4(map, kind, a, b);
                }
            }
        }
    };
    product(&mut map, kind::FOUR_PATH_EDGE, &s_ij, &far, true);
    product(&mut map, kind::FOUR_PATH_CENTER, &s_i, &s_j, false);
    product(&mut map, kind::FOUR_CYCLE, &s_i, &s_j, true);
    product(&mut map, kind::TAILED_TRI_CENTER, &t, &far, true);
    product(&mut map, kind::TAILED_TRI_EDGE, &t, &s_ij, false);
    product(&mut map, kind::CHORDAL_CYCLE_EDGE, &t, &s_ij, true);

    let within = |map: &mut _, kind, s: &[usize], want_edge: bool| {
        for a in 0..s.len() {
            for b in (a + 1)..s.len() {
                if adj(s[a], s[b]) == want_edge {
                    emit4(map, kind, s[a], s[b]);
                }
            }
        }
    };
    within(&mut map, kind::FOUR_STAR, &s_i, false);
    within(&mut map, kind::FOUR_STAR, &s_j, false);
    within(&mut map, kind::TAILED_TRI_TAIL, &s_i, true);
    within(&mut map, kind::TAILED_TRI_TAIL, &s_j, true);
    within(&mut map, kind::CHORDAL_CYCLE_CENTER, &t, false);
    within(&mut map, kind::FOUR_CLIQUE, &t, true);

    map
}

/// Paper-faithful EDGE-typed counts for edge `(i, j)`: the same orbit enumeration
/// as [`paper_typed_counts`], but each occurrence is keyed by its kind together
/// with a canonical positional descriptor of the four node labels and the six
/// edge colours among the four nodes. This is the
/// independent ground truth for the edge-coloured counting.
#[must_use]
pub fn paper_edge_typed_counts(
    graph: &OracleGraph,
    i: usize,
    j: usize,
) -> alloc::collections::BTreeMap<(u8, [u8; 4], [u8; 6]), u64> {
    let n = graph.get_number_of_nodes();
    let adj = |a: usize, b: usize| graph.iter_neighbours(a).any(|x| x == b);
    let lab = |x: usize| graph.get_node_label(x);
    let node_sentinel = graph.get_number_of_node_labels();
    let edge_sentinel = graph.get_number_of_edge_labels();
    // Colour of edge (a, b) if present, else the absent-edge sentinel.
    let col = |a: usize, b: usize| {
        if adj(a, b) {
            graph.get_edge_colour(a, b)
        } else {
            edge_sentinel
        }
    };
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
    let s_ij: Vec<usize> = s_i.iter().chain(&s_j).copied().collect();

    let mut map = alloc::collections::BTreeMap::new();
    let emit3 = |map: &mut alloc::collections::BTreeMap<(u8, [u8; 4], [u8; 6]), u64>,
                 kind: u8,
                 w: usize| {
        let nodes = [lab(i), lab(j), lab(w), node_sentinel];
        let edges = [
            col(i, j),
            col(i, w),
            edge_sentinel,
            col(j, w),
            edge_sentinel,
            edge_sentinel,
        ];
        let (cn, ce) = canonical_descriptor(nodes, edges);
        *map.entry((kind, cn, ce)).or_default() += 1;
    };
    for &w in &s_i {
        emit3(&mut map, kind::TRIAD as u8, w);
    }
    for &w in &s_j {
        emit3(&mut map, kind::TRIAD as u8, w);
    }
    for &w in &t {
        emit3(&mut map, kind::TRIANGLE as u8, w);
    }

    let emit4 = |map: &mut alloc::collections::BTreeMap<(u8, [u8; 4], [u8; 6]), u64>,
                 kind: usize,
                 a: usize,
                 b: usize| {
        let nodes = [lab(i), lab(j), lab(a), lab(b)];
        let edges = [
            col(i, j),
            col(i, a),
            col(i, b),
            col(j, a),
            col(j, b),
            col(a, b),
        ];
        let (cn, ce) = canonical_descriptor(nodes, edges);
        *map.entry((kind as u8, cn, ce)).or_default() += 1;
    };
    let product = |map: &mut _, kind, p: &[usize], q: &[usize], want_edge: bool| {
        for &a in p {
            for &b in q {
                if adj(a, b) == want_edge {
                    emit4(map, kind, a, b);
                }
            }
        }
    };
    product(&mut map, kind::FOUR_PATH_EDGE, &s_ij, &far, true);
    product(&mut map, kind::FOUR_PATH_CENTER, &s_i, &s_j, false);
    product(&mut map, kind::FOUR_CYCLE, &s_i, &s_j, true);
    product(&mut map, kind::TAILED_TRI_CENTER, &t, &far, true);
    product(&mut map, kind::TAILED_TRI_EDGE, &t, &s_ij, false);
    product(&mut map, kind::CHORDAL_CYCLE_EDGE, &t, &s_ij, true);

    let within = |map: &mut _, kind, s: &[usize], want_edge: bool| {
        for a in 0..s.len() {
            for b in (a + 1)..s.len() {
                if adj(s[a], s[b]) == want_edge {
                    emit4(map, kind, s[a], s[b]);
                }
            }
        }
    };
    within(&mut map, kind::FOUR_STAR, &s_i, false);
    within(&mut map, kind::FOUR_STAR, &s_j, false);
    within(&mut map, kind::TAILED_TRI_TAIL, &s_i, true);
    within(&mut map, kind::TAILED_TRI_TAIL, &s_j, true);
    within(&mut map, kind::CHORDAL_CYCLE_CENTER, &t, false);
    within(&mut map, kind::FOUR_CLIQUE, &t, true);

    map
}

/// Paper-faithful counts of the two 4-path orbits for edge `(i, j)`, following
/// the Table 3 definitions of Rossi et al.: `g3` (4-path edge, the queried edge
/// is an end edge) and `g4` (4-path center). Returns `(g3, g4)`.
#[must_use]
pub fn paper_four_path_counts<G: Graph>(graph: &G, i: usize, j: usize) -> (u64, u64) {
    let n = graph.get_number_of_nodes();
    let adjacent = |a: usize, b: usize| graph.iter_neighbours(a).any(|x| x == b);

    // S_i / S_j: exclusive neighbours. far (I): adjacent to neither i nor j.
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
            // Edges among {i, j, k, l}, where (i, j) is always present.
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
        // Cycle 0-1-3-2-0, where edge (0,1) is a cycle edge.
        assert_single(&[(0, 1), (1, 3), (3, 2), (2, 0)], 4, 0, 1, kind::FOUR_CYCLE);
    }

    #[test]
    fn calibrate_four_star() {
        // Star centered at 0 with leaves 1,2,3, where edge (0,1) is a spoke.
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
        // Deterministic random graphs, tallying per-orbit crate vs paper totals to
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

    /// Strategy generating an arbitrary small graph with three node labels, used
    /// to exercise the typed (heterogeneous) counting.
    fn arbitrary_typed_graph() -> impl Strategy<Value = OracleGraph> {
        (2usize..=7)
            .prop_flat_map(|num_nodes| {
                (
                    proptest::collection::vec(
                        (0..num_nodes, 0..num_nodes),
                        0..=num_nodes * num_nodes,
                    ),
                    proptest::collection::vec(0u8..3, num_nodes),
                )
            })
            .prop_map(|(edges, labels)| {
                let num_nodes = labels.len();
                // Three real labels (0..=2) with NO spare label, so the full label
                // range is exercised, including the all-maximum-label 3-node
                // orbits whose `dummy = num_labels` sentinel must still encode and
                // decode without colliding with another graphlet.
                OracleGraph::new(num_nodes, &edges, &labels, 3)
            })
    }

    /// Strategy generating an arbitrary small graph with three node labels and a
    /// given number of edge colours, used to exercise edge-coloured counting.
    fn arbitrary_edge_typed_graph(num_edge_labels: u8) -> impl Strategy<Value = OracleGraph> {
        (2usize..=7)
            .prop_flat_map(|num_nodes| {
                (
                    proptest::collection::vec(
                        (0..num_nodes, 0..num_nodes),
                        0..=num_nodes * num_nodes,
                    ),
                    proptest::collection::vec(0u8..3, num_nodes),
                )
            })
            .prop_flat_map(move |(edges, labels)| {
                let edge_count = edges.len();
                (
                    Just(edges),
                    Just(labels),
                    proptest::collection::vec(0u8..num_edge_labels.max(1), edge_count),
                )
            })
            .prop_map(move |(edges, labels, edge_colours)| {
                let num_nodes = labels.len();
                // Three node colours and `num_edge_labels` edge colours, neither
                // with a spare value, so both sentinel digits (node = 3 and
                // absent-edge = num_edge_labels) are exercised.
                OracleGraph::new_edge_typed(
                    num_nodes,
                    &edges,
                    &edge_colours,
                    &labels,
                    3,
                    num_edge_labels,
                )
            })
    }

    /// Strategy yielding an edge-coloured graph together with a permutation of its
    /// node indices, used to relabel nodes and check invariance. The permutation is
    /// the argsort of random keys, so it is always a bijection of `0..n` (no
    /// isomorphism detection is involved: the relabelling is known by construction).
    fn arbitrary_edge_typed_graph_with_permutation(
    ) -> impl Strategy<Value = (OracleGraph, Vec<usize>)> {
        arbitrary_edge_typed_graph(2)
            .prop_flat_map(|graph| {
                let n = graph.get_number_of_nodes();
                (Just(graph), proptest::collection::vec(any::<u64>(), n))
            })
            .prop_map(|(graph, keys)| {
                let mut order: Vec<usize> = (0..keys.len()).collect();
                order.sort_by_key(|&i| keys[i]);
                let mut permutation = alloc::vec![0usize; keys.len()];
                for (new_index, &old_index) in order.iter().enumerate() {
                    permutation[old_index] = new_index;
                }
                (graph, permutation)
            })
    }

    /// Rebuilds `graph` with its node indices remapped by `permutation` (node `u`
    /// becomes node `permutation[u]`), preserving node labels and edge colours. The
    /// result is the same graph up to a known node renaming.
    fn relabel_nodes(graph: &OracleGraph, permutation: &[usize]) -> OracleGraph {
        let n = graph.get_number_of_nodes();
        let mut edge_pairs = Vec::new();
        let mut edge_colours = Vec::new();
        for (a, b) in graph.edges() {
            edge_pairs.push((permutation[a], permutation[b]));
            edge_colours.push(graph.get_edge_colour(a, b));
        }
        let mut node_labels = alloc::vec![0u8; n];
        for u in 0..n {
            node_labels[permutation[u]] = graph.get_node_label(u);
        }
        OracleGraph::new_edge_typed(
            n,
            &edge_pairs,
            &edge_colours,
            &node_labels,
            graph.get_number_of_node_labels(),
            EdgeTypedGraph::get_number_of_edge_labels(graph),
        )
    }

    /// Aggregates the crate's edge-coloured counts over every edge into a single
    /// canonical-key histogram for the whole graph.
    fn aggregate_edge_typed(
        graph: &OracleGraph,
    ) -> alloc::collections::BTreeMap<(u8, [u8; 4], [u8; 6]), u64> {
        let mut total = alloc::collections::BTreeMap::new();
        for (i, j) in graph.edges() {
            for (key, value) in crate_edge_typed_counts(graph, i, j) {
                *total.entry(key).or_insert(0u64) += value;
            }
        }
        total
    }

    /// Aggregates a per-edge typed-count function over every edge of `graph`.
    fn total_typed_counts(
        graph: &OracleGraph,
        per_edge: impl Fn(
            &OracleGraph,
            usize,
            usize,
        ) -> alloc::collections::BTreeMap<(u8, Vec<u8>), u64>,
    ) -> alloc::collections::BTreeMap<(u8, Vec<u8>), u64> {
        let mut total = alloc::collections::BTreeMap::new();
        for (i, j) in graph.edges() {
            for (key, value) in per_edge(graph, i, j) {
                *total.entry(key).or_insert(0u64) += value;
            }
        }
        total
    }

    /// Paper-reference totals per graphlet-kind NAME for an edge: the typed
    /// reference aggregated by kind name, summing over every colour combination.
    /// This is what the convenience reporting methods (`to_graphlet_names`,
    /// `get_report`) are expected to produce.
    fn paper_kind_name_totals(
        graph: &OracleGraph,
        i: usize,
        j: usize,
    ) -> alloc::collections::BTreeMap<alloc::string::String, u64> {
        let mut totals = alloc::collections::BTreeMap::new();
        for ((kind, _labels), count) in paper_typed_counts(graph, i, j) {
            let name = alloc::format!("{}", ExtendedGraphletType::from(kind));
            *totals.entry(name).or_insert(0u64) += count;
        }
        totals
    }

    // Per-orbit count of the distinct typed keys the algorithm actually emits
    // (its edge-centric hash granularity), as a function of the colour count `c`.
    // These are the formulas documented in the catalog figure and the README.
    fn cube(c: u64) -> u64 {
        c * c * c
    }
    fn fourth(c: u64) -> u64 {
        c * c * c * c
    }
    fn half(c: u64) -> u64 {
        // c^2 * binomial(c+1, 2) = c^3 (c+1) / 2
        c * c * c * (c + 1) / 2
    }

    #[test]
    #[allow(clippy::type_complexity)]
    fn edge_centric_typed_key_counts_match_formula() {
        use hashbrown::HashSet;
        // (name, kind, num_nodes, edges, counted edge, distinct-key formula).
        let orbits: [(
            &str,
            usize,
            usize,
            &[(usize, usize)],
            (usize, usize),
            fn(u64) -> u64,
        ); 12] = [
            ("Triad", kind::TRIAD, 3, &[(0, 1), (0, 2)], (0, 1), cube),
            (
                "Triangle",
                kind::TRIANGLE,
                3,
                &[(0, 1), (1, 2), (0, 2)],
                (0, 1),
                cube,
            ),
            (
                "FourPathEdge",
                kind::FOUR_PATH_EDGE,
                4,
                &[(0, 1), (1, 2), (2, 3)],
                (0, 1),
                fourth,
            ),
            (
                "FourPathCenter",
                kind::FOUR_PATH_CENTER,
                4,
                &[(0, 1), (1, 2), (2, 3)],
                (1, 2),
                half,
            ),
            (
                "FourStar",
                kind::FOUR_STAR,
                4,
                &[(0, 1), (0, 2), (0, 3)],
                (0, 1),
                half,
            ),
            (
                "FourCycle",
                kind::FOUR_CYCLE,
                4,
                &[(0, 1), (1, 2), (2, 3), (3, 0)],
                (0, 1),
                half,
            ),
            (
                "TailedTriTail",
                kind::TAILED_TRI_TAIL,
                4,
                &[(0, 1), (0, 2), (1, 2), (2, 3)],
                (2, 3),
                half,
            ),
            (
                "TailedTriCenter",
                kind::TAILED_TRI_CENTER,
                4,
                &[(0, 1), (0, 2), (1, 2), (2, 3)],
                (0, 1),
                fourth,
            ),
            (
                "TailedTriEdge",
                kind::TAILED_TRI_EDGE,
                4,
                &[(0, 1), (0, 2), (1, 2), (2, 3)],
                (0, 2),
                half,
            ),
            (
                "ChordalCycleEdge",
                kind::CHORDAL_CYCLE_EDGE,
                4,
                &[(0, 1), (1, 2), (2, 3), (3, 0), (1, 3)],
                (0, 1),
                half,
            ),
            (
                "ChordalCycleCenter",
                kind::CHORDAL_CYCLE_CENTER,
                4,
                &[(0, 1), (1, 2), (2, 3), (3, 0), (1, 3)],
                (1, 3),
                half,
            ),
            (
                "FourClique",
                kind::FOUR_CLIQUE,
                4,
                &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
                (0, 1),
                half,
            ),
        ];
        for (name, kind_index, num_nodes, edges, (i, j), formula) in orbits {
            for c in 1u8..=5 {
                let base = u32::from(c) + 1;
                let base4 = base * base * base * base;
                let mut keys: HashSet<u32> = HashSet::new();
                let total = (c as usize).pow(num_nodes as u32);
                for mask in 0..total {
                    let mut labels = alloc::vec![0u8; num_nodes];
                    let mut m = mask;
                    for slot in &mut labels {
                        *slot = (m % c as usize) as u8;
                        m /= c as usize;
                    }
                    let g = OracleGraph::new(num_nodes, edges, &labels, c);
                    for (hash, count) in &g.get_heterogeneous_graphlet(i, j).unwrap() {
                        if *count > 0 && (hash / base4) as usize == kind_index {
                            keys.insert(*hash);
                        }
                    }
                }
                assert_eq!(
                    keys.len() as u64,
                    formula(u64::from(c)),
                    "{name} with c={c}: distinct keys"
                );
            }
        }
    }

    #[test]
    fn typed_counts_distinguish_max_label_triangle_from_four_path_edge() {
        // A Triangle whose three nodes all carry the maximum label and an
        // all-zero-label FourPathEdge hash to the same value (2*n^4) under the
        // base-n perfect hash, because the 3-node `dummy = n` units digit carries
        // through every position. The crate therefore merges their counts. The
        // independent reference keeps them apart, and so must the crate.
        //
        // Triangle 0-1-2 (label 1) plus path 3-4-5-6 (label 0), with n = 2 labels.
        let graph = OracleGraph::new(
            7,
            &[(0, 1), (1, 2), (0, 2), (3, 4), (4, 5), (5, 6)],
            &[1, 1, 1, 0, 0, 0, 0],
            2,
        );
        assert_eq!(
            total_typed_counts(&graph, crate_typed_counts),
            total_typed_counts(&graph, paper_typed_counts),
        );
    }

    #[test]
    fn typed_counts_distinguish_max_label_triad_from_triangle() {
        // The companion collision: a Triad (3-star) whose three nodes all carry
        // the maximum label hashes to n^4, aliasing the all-zero Triangle region.
        // Star centred at 0 (label 1) with leaves 1, 2 (label 1), plus an all-zero
        // triangle 3-4-5, with n = 2 labels.
        let graph = OracleGraph::new(
            6,
            &[(0, 1), (0, 2), (3, 4), (4, 5), (3, 5)],
            &[1, 1, 1, 0, 0, 0],
            2,
        );
        assert_eq!(
            total_typed_counts(&graph, crate_typed_counts),
            total_typed_counts(&graph, paper_typed_counts),
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        /// The fast counter's per-kind totals must match the paper-faithful
        /// reference for every edge of every generated graph. On failure proptest
        /// shrinks to a minimal counterexample graph.
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

        /// The crate's full typed output (per kind and label multiset) must match
        /// the paper-faithful typed reference, validating the heterogeneous
        /// per-label counting.
        #[test]
        fn typed_counts_match_paper_reference(graph in arbitrary_typed_graph()) {
            for (i, j) in graph.edges() {
                prop_assert_eq!(
                    crate_typed_counts(&graph, i, j),
                    paper_typed_counts(&graph, i, j),
                    "edge ({}, {})", i, j
                );
            }
        }

        /// `to_graphlet_names` must report, per graphlet-kind name, the total over
        /// all colour combinations, matching the paper reference. Exercises the
        /// public reporting layer that the raw-count proptests skip.
        #[test]
        fn graphlet_names_match_paper_reference(graph in arbitrary_typed_graph()) {
            for (i, j) in graph.edges() {
                let number_of_labels = graph.get_number_of_node_labels();
                let counts = graph.get_heterogeneous_graphlet(i, j).unwrap();
                let got: alloc::collections::BTreeMap<alloc::string::String, u64> = counts
                    .to_graphlet_names::<ExtendedGraphletType, u8>(number_of_labels)
                    .into_iter()
                    .collect();
                prop_assert_eq!(
                    got,
                    paper_kind_name_totals(&graph, i, j),
                    "edge ({}, {})", i, j
                );
            }
        }

        /// `get_report` must contain exactly one line per graphlet kind present,
        /// each carrying that kind's total count, matching the paper reference.
        #[test]
        fn get_report_matches_paper_reference(graph in arbitrary_typed_graph()) {
            for (i, j) in graph.edges() {
                let number_of_labels = graph.get_number_of_node_labels();
                let counts = graph.get_heterogeneous_graphlet(i, j).unwrap();
                let report = counts.get_report::<ExtendedGraphletType, u8>(number_of_labels);
                let mut got: alloc::collections::BTreeMap<alloc::string::String, u64> =
                    alloc::collections::BTreeMap::new();
                let mut line_count = 0usize;
                for line in report.lines() {
                    if let Some((name, value)) = line.rsplit_once(": ") {
                        if let Ok(value) = value.parse::<u64>() {
                            *got.entry(name.into()).or_insert(0u64) += value;
                            line_count += 1;
                        }
                    }
                }
                let expected = paper_kind_name_totals(&graph, i, j);
                let expected_kinds = expected.len();
                prop_assert_eq!(got, expected, "edge ({}, {})", i, j);
                // No duplicate kind names: exactly one report line per kind.
                prop_assert_eq!(line_count, expected_kinds, "edge ({}, {})", i, j);
            }
        }

        /// Counting an edge is orientation-independent: `(i, j)` and `(j, i)` must
        /// give identical per-kind totals, and identical typed counts once each
        /// graphlet's node colours are taken as an unordered multiset (which the
        /// sorted-label decode does). Expected to hold, but verified not assumed.
        #[test]
        fn counts_are_symmetric_under_edge_orientation(graph in arbitrary_typed_graph()) {
            for (i, j) in graph.edges() {
                prop_assert_eq!(
                    fast_per_kind_counts(&graph, i, j),
                    fast_per_kind_counts(&graph, j, i),
                    "per-kind ({}, {})", i, j
                );
                prop_assert_eq!(
                    crate_typed_counts(&graph, i, j),
                    crate_typed_counts(&graph, j, i),
                    "typed ({}, {})", i, j
                );
            }
        }

        /// Property 1 (edge-colour collapse): summing the edge-typed oracle over
        /// all edge-colour tuples, and reducing the node labels to the sorted
        /// multiset the node-typed oracle uses, must reproduce `paper_typed_counts`
        /// exactly. This validates the new edge-typed oracle as a faithful
        /// refinement of the already-trusted node-typed oracle.
        #[test]
        fn edge_typed_oracle_collapses_to_node_typed(graph in arbitrary_edge_typed_graph(2)) {
            let node_sentinel = graph.get_number_of_node_labels();
            for (i, j) in graph.edges() {
                let mut collapsed: alloc::collections::BTreeMap<(u8, Vec<u8>), u64> =
                    alloc::collections::BTreeMap::new();
                for ((kind, nodes, _edges), count) in paper_edge_typed_counts(&graph, i, j) {
                    let mut multiset: Vec<u8> =
                        nodes.iter().copied().filter(|&l| l != node_sentinel).collect();
                    multiset.sort_unstable();
                    *collapsed.entry((kind, multiset)).or_insert(0) += count;
                }
                prop_assert_eq!(collapsed, paper_typed_counts(&graph, i, j), "edge ({}, {})", i, j);
            }
        }

        /// Property 2 (single-edge-colour degeneracy): with one edge colour the
        /// only edge digits are the present colour 0 and the absent sentinel 1, so
        /// this exercises the sentinel boundary. Collapsing the edge-typed oracle
        /// (strip edge colours, reduce node labels to the sorted multiset) must
        /// still reproduce the node-typed oracle exactly.
        #[test]
        fn single_edge_colour_collapses_to_node_typed(graph in arbitrary_edge_typed_graph(1)) {
            let node_sentinel = graph.get_number_of_node_labels();
            for (i, j) in graph.edges() {
                let mut collapsed: alloc::collections::BTreeMap<(u8, Vec<u8>), u64> =
                    alloc::collections::BTreeMap::new();
                for ((kind, nodes, _edges), count) in paper_edge_typed_counts(&graph, i, j) {
                    let mut multiset: Vec<u8> =
                        nodes.iter().copied().filter(|&l| l != node_sentinel).collect();
                    multiset.sort_unstable();
                    *collapsed.entry((kind, multiset)).or_insert(0) += count;
                }
                prop_assert_eq!(collapsed, paper_typed_counts(&graph, i, j), "edge ({}, {})", i, j);
            }
        }

        /// Property 3 (primary differential gate): the fast edge-coloured counter
        /// must agree with the independent edge-coloured oracle, per edge, at full
        /// canonical-key granularity. This is the only check that catches per-colour
        /// canonicalisation bugs.
        #[test]
        fn fast_edge_typed_matches_paper_reference(graph in arbitrary_edge_typed_graph(2)) {
            for (i, j) in graph.edges() {
                prop_assert_eq!(
                    crate_edge_typed_counts(&graph, i, j),
                    paper_edge_typed_counts(&graph, i, j),
                    "edge ({}, {})", i, j
                );
            }
        }

        /// Property 4 (tie-back): collapsing the fast edge-coloured output over edge
        /// colours (and reducing node labels to the sorted multiset) reproduces the
        /// existing, independently-implemented node-typed crate output. This ties
        /// the new direct-enumeration path to the validated O(1) node-typed path.
        #[test]
        fn fast_edge_typed_collapses_to_node_typed_crate(graph in arbitrary_edge_typed_graph(2)) {
            let node_sentinel = graph.get_number_of_node_labels();
            for (i, j) in graph.edges() {
                let mut collapsed: alloc::collections::BTreeMap<(u8, Vec<u8>), u64> =
                    alloc::collections::BTreeMap::new();
                for ((kind, nodes, _edges), count) in crate_edge_typed_counts(&graph, i, j) {
                    let mut multiset: Vec<u8> =
                        nodes.iter().copied().filter(|&l| l != node_sentinel).collect();
                    multiset.sort_unstable();
                    *collapsed.entry((kind, multiset)).or_insert(0) += count;
                }
                prop_assert_eq!(collapsed, crate_typed_counts(&graph, i, j), "edge ({}, {})", i, j);
            }
        }

        /// Property 5 (single-edge-colour degeneracy, fast path): with one edge
        /// colour the fast edge-coloured output collapses to the node-typed crate
        /// output, exercising the sentinel boundary in the encoded keys.
        #[test]
        fn single_edge_colour_fast_collapses_to_node_typed(graph in arbitrary_edge_typed_graph(1)) {
            let node_sentinel = graph.get_number_of_node_labels();
            for (i, j) in graph.edges() {
                let mut collapsed: alloc::collections::BTreeMap<(u8, Vec<u8>), u64> =
                    alloc::collections::BTreeMap::new();
                for ((kind, nodes, _edges), count) in crate_edge_typed_counts(&graph, i, j) {
                    let mut multiset: Vec<u8> =
                        nodes.iter().copied().filter(|&l| l != node_sentinel).collect();
                    multiset.sort_unstable();
                    *collapsed.entry((kind, multiset)).or_insert(0) += count;
                }
                prop_assert_eq!(collapsed, crate_typed_counts(&graph, i, j), "edge ({}, {})", i, j);
            }
        }

        /// Property 6 (edge-orientation symmetry): the canonical key is invariant
        /// under swapping the focal endpoints, so the full edge-coloured key map for
        /// `(i, j)` equals that for `(j, i)`.
        #[test]
        fn edge_typed_counts_are_symmetric_under_orientation(graph in arbitrary_edge_typed_graph(2)) {
            for (i, j) in graph.edges() {
                prop_assert_eq!(
                    crate_edge_typed_counts(&graph, i, j),
                    crate_edge_typed_counts(&graph, j, i),
                    "edge ({}, {})", i, j
                );
            }
        }

        /// Property 7 (node-relabelling invariance): the canonical key is a true
        /// isomorphism invariant, so renaming the graph's node indices leaves the
        /// whole-graph canonical-key histogram unchanged. This pins the
        /// canonicalisation as COMPLETE (in particular the non-focal-pair swap),
        /// which the fast-vs-oracle gate cannot, since both share the canonical form.
        #[test]
        fn edge_typed_counts_are_node_relabelling_invariant(
            (graph, permutation) in arbitrary_edge_typed_graph_with_permutation()
        ) {
            let relabelled = relabel_nodes(&graph, &permutation);
            prop_assert_eq!(aggregate_edge_typed(&graph), aggregate_edge_typed(&relabelled));
        }

        /// Property 9 (conservation): summing the fast edge-coloured counts over all
        /// keys per kind reproduces the label-free brute-force reference, an
        /// independent counter that never goes through the perfect hash.
        #[test]
        fn edge_typed_per_kind_matches_label_free_reference(graph in arbitrary_edge_typed_graph(2)) {
            for (i, j) in graph.edges() {
                let mut per_kind = [0u64; 12];
                for ((kind, _nodes, _edges), count) in crate_edge_typed_counts(&graph, i, j) {
                    per_kind[kind as usize] += count;
                }
                prop_assert_eq!(
                    per_kind,
                    reference_per_kind_counts(&graph, i, j),
                    "edge ({}, {})", i, j
                );
            }
        }
    }

    #[test]
    fn edge_typed_hash_capacity_accepts_largest_fitting() {
        // The edge-coloured bound is 12 * (c+1)^4 * (d+1)^6. With one node colour
        // (node base 2) and 15 edge colours (edge base 16): 12 * 16 * 16^6 =
        // 3_221_225_472 <= u32::MAX, so a `u32` key suffices and the call returns Ok.
        let graph = OracleGraph::new_edge_typed(2, &[(0, 1)], &[0], &[0, 0], 1, 15);
        assert!(graph.get_edge_typed_graphlet(0, 1).is_ok());
    }

    #[test]
    fn edge_typed_hash_capacity_rejects_one_colour_too_many() {
        // One more edge colour (16, edge base 17): 12 * 16 * 17^6 = 4_634_413_248
        // > u32::MAX, so the encodability check must return an error.
        let graph = OracleGraph::new_edge_typed(2, &[(0, 1)], &[0], &[0, 0], 1, 16);
        assert!(matches!(
            graph.get_edge_typed_graphlet(0, 1),
            Err(GraphletError::EdgeGraphletKeyTooSmall { .. })
        ));
    }
}
