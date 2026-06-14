#![no_main]

use std::collections::BTreeMap;

use arbitrary::Arbitrary;
use heterogeneous_graphlets::oracle::{
    crate_edge_typed_counts, crate_typed_counts, paper_edge_typed_counts,
    reference_graphlet_counts, reference_per_kind_counts, OracleGraph,
};
use heterogeneous_graphlets::prelude::*;
use libfuzzer_sys::fuzz_target;

/// Arbitrary description of a small edge-coloured undirected graph: an edge list
/// of `(src, dst, colour)` triples plus per-node labels.
#[derive(Arbitrary, Debug)]
struct Input {
    num_nodes: u8,
    edges: Vec<(u8, u8, u8)>,
    node_labels: Vec<u8>,
}

fuzz_target!(|input: Input| {
    // 2..=17 nodes, with three node colours and two edge colours so both sentinel
    // digits (node = 3, absent-edge = 2) are exercised. The brute-force oracle is
    // the ground truth, and the fuzz profile enables debug assertions and overflow
    // checks, so a wrong count or an arithmetic slip crashes rather than corrupts.
    let num_nodes = usize::from(input.num_nodes % 16) + 2;
    let edge_pairs: Vec<(usize, usize)> = input
        .edges
        .iter()
        .map(|&(a, b, _)| (usize::from(a), usize::from(b)))
        .collect();
    let edge_colours: Vec<u8> = input.edges.iter().map(|&(_, _, colour)| colour).collect();
    let graph = OracleGraph::new_edge_typed(
        num_nodes,
        &edge_pairs,
        &edge_colours,
        &input.node_labels,
        3,
        2,
    );
    let node_sentinel = graph.get_number_of_node_labels();

    for (i, j) in graph.edges() {
        let crate_edge = crate_edge_typed_counts(&graph, i, j);

        // Property 3: the O(1) fast path matches the independent edge-coloured
        // oracle (this exercises the edge hash encode + decode + canonicalisation).
        assert_eq!(
            crate_edge,
            paper_edge_typed_counts(&graph, i, j),
            "fast vs edge-typed oracle on ({i}, {j})"
        );

        // Property 15: the O(1) path matches the direct enumeration it replaced.
        assert_eq!(
            graph.get_edge_typed_graphlet(i, j).unwrap(),
            graph.get_edge_typed_graphlet_direct(i, j).unwrap(),
            "O(1) vs direct on ({i}, {j})"
        );

        // Property 4: collapsing the edge colours (and reducing node labels to the
        // sorted multiset) reproduces the validated node-typed counts.
        let mut collapsed: BTreeMap<(u8, Vec<u8>), u64> = BTreeMap::new();
        for ((kind, nodes, _edges), count) in &crate_edge {
            let mut multiset: Vec<u8> = nodes
                .iter()
                .copied()
                .filter(|&label| label != node_sentinel)
                .collect();
            multiset.sort_unstable();
            *collapsed.entry((*kind, multiset)).or_insert(0) += *count;
        }
        assert_eq!(
            collapsed,
            crate_typed_counts(&graph, i, j),
            "edge-colour collapse on ({i}, {j})"
        );

        // Property 9: per-kind totals match the label-free brute-force reference.
        let mut per_kind = [0u64; 12];
        for ((kind, _nodes, _edges), count) in &crate_edge {
            per_kind[*kind as usize] += *count;
        }
        assert_eq!(
            per_kind,
            reference_per_kind_counts(&graph, i, j),
            "per-kind vs reference on ({i}, {j})"
        );
    }

    // Whole-graph exactness: the deduplicated edge-coloured counts equal the
    // brute-force occurrence counts over the entire graph (the decisive gate that
    // the full-automorphism canonicalisation and E_g division are exact).
    let dedup = graph.get_edge_typed_graphlet_counts().unwrap();
    assert_eq!(
        dedup,
        reference_graphlet_counts(&graph),
        "dedup vs brute-force reference"
    );

    // Whole-graph conservation: per reduced kind, the deduplicated total times the
    // graphlet edge count E_g equals the raw signature total (per-edge counts summed
    // over every edge), so the deduplication removes exactly the per-edge multiplicity.
    let mut signature_per_kind: BTreeMap<ReducedGraphletType, u64> = BTreeMap::new();
    for (i, j) in graph.edges() {
        for ((kind, _nodes, _edges), count) in crate_edge_typed_counts(&graph, i, j) {
            let reduced = ReducedGraphletType::from(ExtendedGraphletType::from(kind));
            *signature_per_kind.entry(reduced).or_insert(0) += count;
        }
    }
    let mut dedup_per_kind: BTreeMap<ReducedGraphletType, u64> = BTreeMap::new();
    for ((reduced, _nodes, _edges), count) in &dedup {
        *dedup_per_kind.entry(*reduced).or_insert(0) += *count;
    }
    for (reduced, signature_total) in signature_per_kind {
        let dedup_total = dedup_per_kind.get(&reduced).copied().unwrap_or(0);
        assert_eq!(
            dedup_total * reduced.number_of_edges() as u64,
            signature_total,
            "conservation for {reduced:?}"
        );
    }
});
