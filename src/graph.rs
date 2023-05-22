pub trait Graph {
    type Node;

    /// Returns the number of nodes in the graph.
    fn get_number_of_nodes(&self) -> usize;

    /// Returns the number of edges in the graph.
    fn get_number_of_edges(&self) -> usize;

    /// Iterates over neighbours of the given node.
    /// 
    /// # Arguments
    /// * `node` - The node whose neighbours should be iterated over.
    fn iter_neighbours(&self, node: usize) -> impl Iterator<Item = usize> + '_;
}

pub trait TypedGraph: Graph {
    type NodeLabel: Eq;
}