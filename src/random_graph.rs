use crate::graph::{Graph, TypedGraph};

/// Implicit graph fully rapresented by the provided random state and the number of nodes.
///
pub struct RandomGraph {
    random_state: usize,
    number_of_nodes: usize,
    maximal_node_degree: usize,
    number_of_labels: usize,
    rasterized_edges: Vec<(usize, usize)>,
}

impl RandomGraph {
    /// Create a new RandomGraph from the provided random state and number of nodes.
    ///
    /// # Arguments
    ///
    /// * `random_state` - The random state used to generate the graph.
    /// * `number_of_nodes` - The number of nodes in the graph.
    /// * `maximal_node_degree` - The maximal node degree in the graph.
    /// * `number_of_labels` - The number of labels in the graph.
    ///
    pub fn new(
        random_state: usize,
        number_of_nodes: usize,
        maximal_node_degree: usize,
        number_of_labels: usize,
    ) -> Self {
        let mut graph = Self {
            random_state,
            number_of_nodes,
            maximal_node_degree,
            number_of_labels,
            rasterized_edges: Vec::new(),
        };

        graph.rasterized_edges = (0..graph.number_of_nodes)
            .flat_map(move |node_id| {
                let mut counter = graph.random_state;
                (0..graph.maximal_node_degree)
                    .map(move |_| {
                        counter = counter.wrapping_mul(1103515245).wrapping_add(12345);
                        counter.wrapping_rem(graph.number_of_nodes)
                    })
                    .take_while(move |dst| *dst != node_id && (dst % (node_id + 1)) != 0)
                    .flat_map(move |dst| [(node_id, dst), (dst, node_id)])
            })
            .collect();
        graph.rasterized_edges.sort_unstable();
        graph.rasterized_edges.dedup();

        graph
    }

    pub fn iter_neighbours_from_node_id(&self, node_id: usize) -> impl Iterator<Item = usize> + '_ {
        self.iter_edges()
            .filter(move |(src, _)| *src == node_id)
            .map(|(_, dst)| dst)
    }

    pub fn iter_edges(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.rasterized_edges.iter().copied()
    }

    pub fn get_node_degree(&self, node_id: usize) -> usize {
        self.iter_neighbours_from_node_id(node_id).count()
    }
}

impl Graph for RandomGraph {
    type Node = usize;

    fn get_number_of_nodes(&self) -> usize {
        self.number_of_nodes
    }

    fn get_number_of_edges(&self) -> usize {
        self.rasterized_edges.len()
    }

    fn iter_neighbours(&self, node: usize) -> impl Iterator<Item = usize> + '_ {
        self.iter_neighbours_from_node_id(node)
    }
}

impl TypedGraph for RandomGraph {
    type NodeLabel = usize;

    fn get_number_of_node_labels(&self) -> Self::NodeLabel {
        self.number_of_labels
    }

    fn get_number_of_node_labels_usize(&self) -> usize {
        self.number_of_labels
    }

    fn get_number_of_node_label_from_usize(&self, label_index: usize) -> Self::NodeLabel {
        label_index
    }

    fn get_number_of_node_label_index(&self, label: Self::NodeLabel) -> usize {
        label
    }

    fn get_node_label(&self, node: usize) -> Self::NodeLabel {
        node.wrapping_mul(self.random_state) % self.number_of_labels
    }
}
