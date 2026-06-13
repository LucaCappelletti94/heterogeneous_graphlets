//! Graph abstractions over which graphlets are counted.

use core::fmt::Debug;

/// A graph whose nodes are identified by contiguous `usize` indices.
pub trait Graph {
    /// Iterator over the neighbours of a node, yielding their node indices.
    type NeighbourIter<'a>: Iterator<Item = usize> + 'a
    where
        Self: 'a;

    /// Returns the number of nodes in the graph.
    fn get_number_of_nodes(&self) -> usize;

    /// Returns the number of edges in the graph.
    fn get_number_of_edges(&self) -> usize;

    /// Iterates over neighbours of the given node.
    ///
    /// # Arguments
    /// * `node` - The node whose neighbours should be iterated over.
    fn iter_neighbours(&self, node: usize) -> Self::NeighbourIter<'_>;
}

/// A [`Graph`] whose nodes carry labels (types).
pub trait TypedGraph: Graph {
    /// The type used to represent a node label.
    type NodeLabel: Eq + Debug + Copy;

    /// Returns the number of node labels in the graph.
    fn get_number_of_node_labels(&self) -> Self::NodeLabel;

    /// Returns the number of node labels in the graph as usize
    fn get_number_of_node_labels_usize(&self) -> usize;

    /// Returns the node label curresponding to the provided label index.
    fn get_node_label_from_usize(&self, label_index: usize) -> Self::NodeLabel;

    /// Returns the node label index curresponding to the provided node label:
    fn get_node_label_index(&self, label: Self::NodeLabel) -> usize;

    /// Returns the node label of the given node.
    ///
    /// # Arguments
    /// * `node` - The node whose label should be returned.
    fn get_node_label(&self, node: usize) -> Self::NodeLabel;
}
