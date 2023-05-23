use heterogeneous_graphlets::prelude::*;

#[test]
fn test_random_graph() {
    let graph = RandomGraph::new(42, 100, 5, 8);

    println!(
        "The graph has {} nodes, {} edges and {:?} labels.",
        graph.get_number_of_nodes(),
        graph.get_number_of_edges(),
        graph.get_number_of_node_labels()
    );

    graph.iter_edges().for_each(|(src, dst)| {
        let _graphlet_count = graph.get_heterogeneous_graphlet(src, dst);
    });
}
