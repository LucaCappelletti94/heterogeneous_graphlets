use heterogeneous_graphlets::prelude::*;
use indicatif::ProgressIterator;

#[test]
fn test_random_graph() {
    for random_state in (50..1000).progress() {
        let graph = RandomGraph::new(random_state, 100, 10, 8);
        graph.iter_edges().for_each(|(src, dst)| {
            let _graphlet_count = graph.get_heterogeneous_graphlet(src, dst);
        });
    }
}
