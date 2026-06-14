//! Graph abstractions over which graphlets are counted.

use core::fmt::Debug;

/// A graph whose nodes are identified by contiguous `usize` indices in
/// `0..get_number_of_nodes()`.
///
/// # Contract
///
/// The counting algorithm assumes the following invariants of any
/// implementation. They are not checked at run time, so violating them produces
/// wrong counts or a panic rather than an error:
///
/// * [`iter_neighbours`](Graph::iter_neighbours) yields a node's neighbours as
///   strictly ascending node indices, with no duplicates and no self-loop (a
///   node is never listed as its own neighbour).
/// * The graph is undirected and symmetric: if `b` appears in the neighbours of
///   `a`, then `a` appears in the neighbours of `b`.
pub trait Graph {
    /// Iterator over the neighbours of a node, yielding their node indices in
    /// strictly ascending order (no duplicates, no self-loop).
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
///
/// # Contract
///
/// Labels are identified by an index in `0..get_number_of_node_labels_usize()`.
/// The implementation must satisfy, without these being checked at run time:
///
/// * [`get_node_label_index`](TypedGraph::get_node_label_index) and
///   [`get_node_label_from_usize`](TypedGraph::get_node_label_from_usize) are
///   mutual inverses over that index range.
/// * Every label returned by [`get_node_label`](TypedGraph::get_node_label) has
///   an index within that range.
pub trait TypedGraph: Graph {
    /// The type used to represent a node label.
    type NodeLabel: Eq + Debug + Copy;

    /// Returns the number of node labels in the graph.
    fn get_number_of_node_labels(&self) -> Self::NodeLabel;

    /// Returns the number of node labels in the graph as usize
    fn get_number_of_node_labels_usize(&self) -> usize;

    /// Returns the node label corresponding to the provided label index.
    fn get_node_label_from_usize(&self, label_index: usize) -> Self::NodeLabel;

    /// Returns the node label index corresponding to the provided node label.
    fn get_node_label_index(&self, label: Self::NodeLabel) -> usize;

    /// Returns the node label of the given node.
    ///
    /// # Arguments
    /// * `node` - The node whose label should be returned.
    fn get_node_label(&self, node: usize) -> Self::NodeLabel;
}

/// A [`TypedGraph`] whose edges also carry labels (colours).
///
/// # Contract
///
/// Edge labels are identified by an index in
/// `0..get_number_of_edge_labels_usize()`. The implementation must satisfy,
/// without these being checked at run time:
///
/// * [`get_edge_label`](EdgeTypedGraph::get_edge_label) is symmetric, that is
///   `get_edge_label(a, b) == get_edge_label(b, a)` for every edge `(a, b)`, and
///   is only queried for node pairs that are edges.
/// * [`get_edge_label_index`](EdgeTypedGraph::get_edge_label_index) and
///   [`get_edge_label_from_usize`](EdgeTypedGraph::get_edge_label_from_usize) are
///   mutual inverses over the index range, and every label returned by
///   [`get_edge_label`](EdgeTypedGraph::get_edge_label) has an index within it.
pub trait EdgeTypedGraph: TypedGraph {
    /// The type used to represent an edge label.
    type EdgeLabel: Eq + Debug + Copy;

    /// Returns the number of edge labels in the graph.
    fn get_number_of_edge_labels(&self) -> Self::EdgeLabel;

    /// Returns the number of edge labels in the graph as usize.
    fn get_number_of_edge_labels_usize(&self) -> usize;

    /// Returns the edge label corresponding to the provided label index.
    fn get_edge_label_from_usize(&self, label_index: usize) -> Self::EdgeLabel;

    /// Returns the edge label index corresponding to the provided edge label.
    fn get_edge_label_index(&self, label: Self::EdgeLabel) -> usize;

    /// Returns the edge label of the edge `(src, dst)`.
    ///
    /// # Arguments
    /// * `src` - One endpoint of the edge.
    /// * `dst` - The other endpoint of the edge.
    fn get_edge_label(&self, src: usize, dst: usize) -> Self::EdgeLabel;
}
