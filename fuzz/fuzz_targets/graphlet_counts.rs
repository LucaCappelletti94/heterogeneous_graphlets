#![no_main]

use arbitrary::Arbitrary;
use heterogeneous_graphlets::oracle::{fast_per_kind_counts, paper_per_kind_counts, OracleGraph};
use libfuzzer_sys::fuzz_target;

/// Arbitrary description of a small undirected graph.
#[derive(Arbitrary, Debug)]
struct Input {
    num_nodes: u8,
    edges: Vec<(u8, u8)>,
}

fuzz_target!(|input: Input| {
    // 2..=33 nodes. A single label is used so the per-kind decode is exact, so the
    // bug surface being fuzzed is the structural orbit enumeration.
    let num_nodes = usize::from(input.num_nodes % 32) + 2;
    let edges: Vec<(usize, usize)> = input
        .edges
        .iter()
        .map(|&(a, b)| (usize::from(a), usize::from(b)))
        .collect();
    let graph = OracleGraph::new(num_nodes, &edges, &[], 2);

    // For every edge, the fast counter (whose internal differential oracle also
    // runs under the fuzz profile's debug assertions) must agree with the
    // independent paper-faithful reference.
    for (i, j) in graph.edges() {
        assert_eq!(
            fast_per_kind_counts(&graph, i, j),
            paper_per_kind_counts(&graph, i, j),
            "fast vs paper mismatch on edge ({i}, {j})"
        );
    }
});
