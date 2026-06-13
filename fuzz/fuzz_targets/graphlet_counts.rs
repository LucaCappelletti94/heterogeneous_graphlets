#![no_main]

use arbitrary::Arbitrary;
use hashbrown::HashMap;
use heterogeneous_graphlets::prelude::*;
use libfuzzer_sys::fuzz_target;
use std::collections::BTreeSet;

/// Arbitrary description of a small typed, undirected graph.
#[derive(Arbitrary, Debug)]
struct Input {
    num_labels: u8,
    node_labels: Vec<u8>,
    edges: Vec<(u8, u8)>,
}

/// Minimal CSR graph with sorted neighbour lists, built so that the algorithm's
/// preconditions (undirected, sorted neighbours, no self-loops) always hold.
struct FuzzGraph {
    offsets: Vec<usize>,
    edges: Vec<usize>,
    labels: Vec<u8>,
    num_labels: u8,
}

impl FuzzGraph {
    fn build(input: &Input) -> Self {
        // 2..=16 nodes, 1..=4 labels.
        let num_nodes = input.node_labels.len().clamp(2, 16);
        let num_labels = (input.num_labels % 4) + 1;
        let labels: Vec<u8> = (0..num_nodes)
            .map(|i| input.node_labels.get(i).copied().unwrap_or(0) % num_labels)
            .collect();

        let mut adjacency: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); num_nodes];
        for &(a, b) in &input.edges {
            let a = (a as usize) % num_nodes;
            let b = (b as usize) % num_nodes;
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

        Self {
            offsets,
            edges,
            labels,
            num_labels,
        }
    }
}

impl Graph for FuzzGraph {
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

impl TypedGraph for FuzzGraph {
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

impl HeterogeneousGraphlets<u128, u64> for FuzzGraph {
    type GraphLetCounter = HashMap<u128, u64>;
}

fuzz_target!(|input: Input| {
    let graph = FuzzGraph::build(&input);

    // Count graphlets for every (sorted) edge. With debug assertions enabled the
    // crate's internal oracle verifies each edge's counts, and overflow checks
    // catch arithmetic mistakes, so any incorrect result becomes a crash.
    for src in 0..graph.get_number_of_nodes() {
        for dst in graph.iter_neighbours(src) {
            let _: HashMap<u128, u64> = graph.get_heterogeneous_graphlet(src, dst);
        }
    }
});
