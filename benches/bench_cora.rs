#![feature(test)]
extern crate test;
use test::{black_box, Bencher};

use std::collections::HashMap;

use heterogeneous_graphlets::prelude::*;
use rayon::prelude::*;

/// Compressed Sparse Row Graph
pub struct CSRGraph {
    /// The number of nodes in the graph.
    number_of_nodes: usize,
    /// The number of edges in the graph.
    number_of_edges: usize,
    /// The number of node labels in the graph.
    number_of_node_labels: usize,
    /// The node labels of the graph.
    node_labels: Vec<usize>,
    /// The offsets of the graph.
    offsets: Vec<usize>,
    /// The edges of the graph.
    edges: Vec<usize>,
}

unsafe impl Send for CSRGraph {}
unsafe impl Sync for CSRGraph {}

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
    /// Create a new CSRGraph from the provided node list and edge list.
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
                assert!(node_label.len() == 1);
                node_label[0]
            })
            .collect::<Vec<usize>>();
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
                "src: {}, number_of_nodes: {}",
                src,
                number_of_nodes
            );
            assert!(
                dst < number_of_nodes,
                "dst: {}, number_of_nodes: {}",
                dst,
                number_of_nodes
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
            node_labels,
            offsets,
            edges,
        })
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
        (0..self.number_of_nodes).into_iter().flat_map(move |node| {
            let src_offset = self.offsets[node];
            let dst_offset = self.offsets[node + 1];
            self.edges[src_offset..dst_offset]
                .iter()
                .map(move |dst| (node, *dst))
        })
    }
}

impl Graph for CSRGraph {
    type Node = usize;
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
    type NodeLabel = usize;

    fn get_number_of_node_labels(&self) -> usize {
        self.number_of_node_labels
    }

    fn get_number_of_node_labels_usize(&self) -> usize {
        self.number_of_node_labels
    }

    fn get_number_of_node_label_from_usize(&self, label_index: usize) -> usize {
        label_index
    }

    fn get_number_of_node_label_index(&self, label: usize) -> usize {
        label
    }

    fn get_node_label(&self, node: usize) -> usize {
        self.node_labels[node]
    }
}

impl HeterogeneousGraphlets for CSRGraph {
    type GraphLetCounter = HashMap<usize, usize>;
}

#[bench]
fn bench_cora(b: &mut Bencher) {
    let graph = CSRGraph::from_csv(
        "tests/data/cora/node_list.csv",
        "tests/data/cora/edge_list.csv",
    )
    .unwrap();
    b.iter(|| {
        // Inner closure, the actual test
        black_box({
            graph
                .iter_edges()
                .filter(|(src, dst)| src < dst)
                .for_each(|(src, dst)| {
                    graph.get_heterogeneous_graphlet(src, dst);
                });
        });
    });
}

#[bench]
fn bench_citeseer(b: &mut Bencher) {
    let graph = CSRGraph::from_csv(
        "tests/data/citeseer/node_list.csv",
        "tests/data/citeseer/edge_list.csv",
    )
    .unwrap();
    b.iter(|| {
        // Inner closure, the actual test
        black_box({
            graph
                .iter_edges()
                .filter(|(src, dst)| src < dst)
                .for_each(|(src, dst)| {
                    graph.get_heterogeneous_graphlet(src, dst);
                });
        });
    });
}

#[bench]
fn bench_par_cora(b: &mut Bencher) {
    let graph = CSRGraph::from_csv(
        "tests/data/cora/node_list.csv",
        "tests/data/cora/edge_list.csv",
    )
    .unwrap();
    b.iter(|| {
        // Inner closure, the actual test
        black_box({
            graph
                .par_iter_edges()
                .filter(|(src, dst)| src < dst)
                .for_each(|(src, dst)| {
                    graph.get_heterogeneous_graphlet(src, dst);
                });
        });
    });
}

#[bench]
fn bench_par_citeseer(b: &mut Bencher) {
    let graph = CSRGraph::from_csv(
        "tests/data/citeseer/node_list.csv",
        "tests/data/citeseer/edge_list.csv",
    )
    .unwrap();
    b.iter(|| {
        // Inner closure, the actual test
        black_box({
            graph
                .par_iter_edges()
                .filter(|(src, dst)| src < dst)
                .for_each(|(src, dst)| {
                    graph.get_heterogeneous_graphlet(src, dst);
                });
        });
    });
}
