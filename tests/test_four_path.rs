#![feature(return_position_impl_trait_in_trait)]

use csr_graph::CSRGraph;
use heterogeneous_graphlets::prelude::*;
use rayon::prelude::*;
mod csr_graph;

#[test]
fn test_four_path() {
    let graph = CSRGraph::from_csv(
        "tests/data/four_path/node_list.csv",
        "tests/data/four_path/edge_list.csv",
    )
    .unwrap();

    let counts = graph
        .iter_edges()
        .map(|(src, dst)| graph.get_heterogeneous_graphlet(src, dst))
        .reduce(|mut left, right| {
            for (graphlet, count) in right.iter() {
                left.insert_count(*graphlet, *count);
            }
            left
        })
        .unwrap();
    println!(
        "{}",
        counts
            .get_report(graph.get_number_of_node_labels())
            .unwrap()
    );
}
