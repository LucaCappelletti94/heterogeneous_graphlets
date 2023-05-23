use std::fmt::Debug;

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
    type NodeLabel: Eq + Debug;

    /// Returns the number of node labels in the graph.
    fn get_number_of_node_labels(&self) -> Self::NodeLabel;

    /// Returns the number of node labels in the graph as usize
    fn get_number_of_node_labels_usize(&self) -> usize;

    /// Returns the node label curresponding to the provided label index.
    fn get_number_of_node_label_from_usize(&self, label_index: usize) -> Self::NodeLabel;

    /// Returns the node label index curresponding to the provided node label:
    fn get_number_of_node_label_index(&self, label: Self::NodeLabel) -> usize;

    /// Returns the node label of the given node.
    /// 
    /// # Arguments
    /// 
    /// * `node` - The node whose label should be returned.
    fn get_node_label(&self, node: usize) -> Self::NodeLabel;
}