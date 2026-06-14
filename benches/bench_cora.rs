#![allow(
    missing_docs,
    missing_debug_implementations,
    unreachable_pub,
    clippy::unwrap_used,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::must_use_candidate
)]

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use hashbrown::HashMap;

use heterogeneous_graphlets::prelude::*;
use rayon::prelude::*;

/// Compressed Sparse Row Graph
pub struct CSRGraph {
    /// The number of nodes in the graph.
    number_of_nodes: usize,
    /// The number of edges in the graph.
    number_of_edges: usize,
    /// The number of node labels in the graph.
    number_of_node_labels: u8,
    /// The number of edge colours used by the synthetic edge-coloured benchmark.
    number_of_edge_labels: u8,
    /// The node labels of the graph.
    node_labels: Vec<u8>,
    /// The offsets of the graph.
    offsets: Vec<usize>,
    /// The edges of the graph.
    edges: Vec<usize>,
}

fn read_csv(path: &str) -> Result<Vec<Vec<usize>>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)
        .map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|e| e.to_string())?;
        result.push(
            record
                .into_iter()
                .map(|value| value.parse::<usize>().map_err(|e| e.to_string()))
                .collect::<Result<Vec<usize>, String>>()?,
        );
    }
    Ok(result)
}

impl CSRGraph {
    /// Create a new `CSRGraph` from the provided node list and edge list.
    ///
    /// # Arguments
    /// * `node_list_path` - The path to the node list.
    /// * `edge_list_path` - The path to the edge list.
    ///
    /// # Implementation details
    /// We expect the node list to be a csv file containing a single column
    /// with the NUMERIC node labels. The length of the column must be equal
    /// to the number of nodes in the graph. An example of the node list is
    /// the following:
    ///
    /// ```csv
    /// 0
    /// 1
    /// 2
    /// 1
    /// 4
    /// ```
    ///
    /// The edge list must be a csv file containing two columns with the NUMERIC
    /// source and destination node IDs. The length of the columns must be equal
    /// to the number of edges in the graph. An example of the edge list is the
    /// following:
    ///
    /// ```csv
    /// 0,1
    /// 1,2
    /// 2,3
    /// 1,4
    /// 4,5
    /// ```
    ///
    pub fn from_csv(node_list_path: &str, edge_list_path: &str) -> Result<Self, String> {
        let mut edge_list = read_csv(edge_list_path)?;
        edge_list.sort_unstable();

        let number_of_edges = edge_list.len();

        let node_labels = read_csv(node_list_path)?
            .into_iter()
            .map(|node_label| {
                assert_eq!(node_label.len(), 1);
                node_label[0] as u8
            })
            .collect::<Vec<u8>>();
        let number_of_nodes = node_labels.len();
        let mut offsets = Vec::with_capacity(number_of_nodes + 1);
        let mut edges = Vec::with_capacity(number_of_edges);

        let mut current_offset = 0;
        let mut current_node = 0;
        offsets.push(current_offset);

        for edge in edge_list {
            let src = edge[0];
            let dst = edge[1];
            assert!(
                src < number_of_nodes,
                "src: {src}, number_of_nodes: {number_of_nodes}"
            );
            assert!(
                dst < number_of_nodes,
                "dst: {dst}, number_of_nodes: {number_of_nodes}"
            );
            assert!(src != dst, "Self-loops are not supported.");
            if src != current_node {
                current_node = src;
                offsets.push(current_offset);
            }
            current_offset += 1;
            edges.push(dst);
        }

        while offsets.len() <= number_of_nodes {
            offsets.push(current_offset);
        }

        Ok(Self {
            number_of_nodes,
            number_of_edges,
            number_of_node_labels: node_labels.iter().max().unwrap() + 1,
            number_of_edge_labels: 1,
            node_labels,
            offsets,
            edges,
        })
    }

    /// Sets the number of synthetic edge colours used by the edge-coloured
    /// benchmark (the colour of edge `(a, b)` is `(a + b) % number_of_edge_labels`).
    #[must_use]
    pub fn with_edge_labels(mut self, number_of_edge_labels: u8) -> Self {
        self.number_of_edge_labels = number_of_edge_labels.max(1);
        self
    }

    /// Iterates in parallel over the edges.
    pub fn par_iter_edges(&self) -> impl ParallelIterator<Item = (usize, usize)> + '_ {
        (0..self.number_of_nodes)
            .into_par_iter()
            .flat_map(move |node| {
                let src_offset = self.offsets[node];
                let dst_offset = self.offsets[node + 1];
                self.edges[src_offset..dst_offset]
                    .par_iter()
                    .map(move |dst| (node, *dst))
            })
    }

    /// Iterates over the edges.
    pub fn iter_edges(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        (0..self.number_of_nodes).flat_map(move |node| {
            let src_offset = self.offsets[node];
            let dst_offset = self.offsets[node + 1];
            self.edges[src_offset..dst_offset]
                .iter()
                .map(move |dst| (node, *dst))
        })
    }
}

impl Graph for CSRGraph {
    type NeighbourIter<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

    fn get_number_of_nodes(&self) -> usize {
        self.number_of_nodes
    }

    fn get_number_of_edges(&self) -> usize {
        self.number_of_edges
    }

    fn iter_neighbours(&self, node: usize) -> Self::NeighbourIter<'_> {
        let src_offset = self.offsets[node];
        let dst_offset = self.offsets[node + 1];
        self.edges[src_offset..dst_offset].iter().copied()
    }
}

impl TypedGraph for CSRGraph {
    type NodeLabel = u8;

    fn get_number_of_node_labels(&self) -> Self::NodeLabel {
        self.number_of_node_labels
    }

    fn get_number_of_node_labels_usize(&self) -> usize {
        self.number_of_node_labels as usize
    }

    fn get_node_label_from_usize(&self, label_index: usize) -> Self::NodeLabel {
        label_index as u8
    }

    fn get_node_label_index(&self, label: Self::NodeLabel) -> usize {
        label as usize
    }

    fn get_node_label(&self, node: usize) -> Self::NodeLabel {
        self.node_labels[node]
    }
}

impl HeterogeneousGraphlets<u16, u32> for CSRGraph {
    type GraphLetCounter = HashMap<u16, u32>;
}

impl EdgeTypedGraph for CSRGraph {
    type EdgeLabel = u8;

    fn get_number_of_edge_labels(&self) -> u8 {
        self.number_of_edge_labels
    }

    fn get_number_of_edge_labels_usize(&self) -> usize {
        self.number_of_edge_labels as usize
    }

    fn get_edge_label_from_usize(&self, label_index: usize) -> u8 {
        label_index as u8
    }

    fn get_edge_label_index(&self, label: u8) -> usize {
        label as usize
    }

    fn get_edge_label(&self, src: usize, dst: usize) -> u8 {
        ((src + dst) % usize::from(self.number_of_edge_labels)) as u8
    }
}

impl EdgeTypedGraphlets<u64, u64> for CSRGraph {
    type GraphLetCounter = HashMap<u64, u64>;
}

/// Counts the graphlets of every edge of the graph on a single thread.
fn count_single_thread(graph: &CSRGraph) {
    graph
        .iter_edges()
        .filter(|(src, dst)| src < dst)
        .for_each(|(src, dst)| {
            black_box(graph.get_heterogeneous_graphlet(src, dst).unwrap());
        });
}

/// Counts the graphlets of every edge of the graph in parallel.
fn count_multi_thread(graph: &CSRGraph) {
    graph
        .par_iter_edges()
        .filter(|(src, dst)| src < dst)
        .for_each(|(src, dst)| {
            black_box(graph.get_heterogeneous_graphlet(src, dst).unwrap());
        });
}

/// Counts the edge-coloured graphlets of every edge on a single thread.
fn count_single_thread_edge_typed(graph: &CSRGraph) {
    graph
        .iter_edges()
        .filter(|(src, dst)| src < dst)
        .for_each(|(src, dst)| {
            black_box(graph.get_edge_typed_graphlet(src, dst).unwrap());
        });
}

/// Counts the edge-coloured graphlets of every edge in parallel.
fn count_multi_thread_edge_typed(graph: &CSRGraph) {
    graph
        .par_iter_edges()
        .filter(|(src, dst)| src < dst)
        .for_each(|(src, dst)| {
            black_box(graph.get_edge_typed_graphlet(src, dst).unwrap());
        });
}

fn bench_graphlets(c: &mut Criterion) {
    let cora = CSRGraph::from_csv(
        "tests/data/cora/node_list.csv",
        "tests/data/cora/edge_list.csv",
    )
    .unwrap();
    let citeseer = CSRGraph::from_csv(
        "tests/data/citeseer/node_list.csv",
        "tests/data/citeseer/edge_list.csv",
    )
    .unwrap();

    c.bench_function("single_thread_cora", |b| {
        b.iter(|| count_single_thread(&cora));
    });
    c.bench_function("single_thread_citeseer", |b| {
        b.iter(|| count_single_thread(&citeseer));
    });
    c.bench_function("multi_thread_cora", |b| {
        b.iter(|| count_multi_thread(&cora));
    });
    c.bench_function("multi_thread_citeseer", |b| {
        b.iter(|| count_multi_thread(&citeseer));
    });

    // Edge-coloured benchmarks, at one and three synthetic edge colours, to
    // quantify the direct-enumeration cost against the node-only path. Grouped
    // with a small sample size because the current direct-enumeration path is
    // slow per iteration, so the default sample count would make `cargo bench`
    // impractical.
    let mut group = c.benchmark_group("edge_typed");
    group.sample_size(10);
    for &num_edge_labels in &[1u8, 3] {
        let cora = CSRGraph::from_csv(
            "tests/data/cora/node_list.csv",
            "tests/data/cora/edge_list.csv",
        )
        .unwrap()
        .with_edge_labels(num_edge_labels);
        let citeseer = CSRGraph::from_csv(
            "tests/data/citeseer/node_list.csv",
            "tests/data/citeseer/edge_list.csv",
        )
        .unwrap()
        .with_edge_labels(num_edge_labels);

        group.bench_function(format!("single_thread_cora_d{num_edge_labels}"), |b| {
            b.iter(|| count_single_thread_edge_typed(&cora));
        });
        group.bench_function(format!("single_thread_citeseer_d{num_edge_labels}"), |b| {
            b.iter(|| count_single_thread_edge_typed(&citeseer));
        });
        group.bench_function(format!("multi_thread_cora_d{num_edge_labels}"), |b| {
            b.iter(|| count_multi_thread_edge_typed(&cora));
        });
        group.bench_function(format!("multi_thread_citeseer_d{num_edge_labels}"), |b| {
            b.iter(|| count_multi_thread_edge_typed(&citeseer));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_graphlets);
criterion_main!(benches);
