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
    type NodeLabel: Eq + Debug + Copy;

    /// Returns the number of node labels in the graph.
    fn get_number_of_node_labels(&self) -> Self::NodeLabel;

    /// Returns the number of node labels in the graph as usize
    fn get_number_of_node_labels_usize(&self) -> usize;

    /// Returns the node label curresponding to the provided label index.
    fn get_number_of_node_label_from_usize(&self, label_index: usize) -> Self::NodeLabel;

    /// Returns the node label index curresponding to the provided node label:
    fn get_number_of_node_label_index(&self, label: Self::NodeLabel) -> usize;

    /// Iterate the node labels of the neighbours of a given node.
    ///
    /// # Arguments
    /// * `node` - The node whose neighbours' labels should be iterated over.
    fn iter_neighbour_labels(&self, node: usize) -> impl Iterator<Item = Self::NodeLabel> + '_ {
        self.iter_neighbours(node)
            .map(move |neighbour| self.get_node_label(neighbour))
    }

    /// Iterate the neighbours of a node solely of the given label.
    ///
    /// # Arguments
    /// * `node` - The node whose neighbours should be iterated over.
    /// * `label` - The label of the neighbours to iterate over.
    fn iter_neighbours_of_label(
        &self,
        node: usize,
        label: Self::NodeLabel,
    ) -> impl Iterator<Item = usize> + '_ {
        self.iter_neighbours(node)
            .filter(move |neighbour| self.get_node_label(*neighbour) == label)
    }

    /// Returns the subtraction of the neighbours of two given nodes and a given label.
    /// 
    /// # Arguments
    /// * `first_node` - The first node whose neighbours should be from.
    /// * `second_node` - The second node whose neighbours should be subtracted.
    /// * `label` - The label of the neighbours to subtract.
    /// 
    /// # Implementation details
    /// We assume that the provided node neighbours are sorted.
    fn get_subtraction_of_neighbours_of_label(
        &self,
        first_node: usize,
        second_node: usize,
        label: Self::NodeLabel,
    ) -> impl Iterator<Item = usize> + '_ {
        let mut first_node_neighbours = self.iter_neighbours_of_label(first_node, label);
        let mut second_node_neighbours = self.iter_neighbours_of_label(second_node, label);

        let mut result = Vec::new();

        let mut first_node_neighbour = first_node_neighbours.next();
        let mut second_node_neighbour = second_node_neighbours.next();

        while let (Some(first_node_neighbour_value), Some(second_node_neighbour_value)) =
            (first_node_neighbour, second_node_neighbour)
        {
            if first_node_neighbour_value == second_node_neighbour_value {
                first_node_neighbour = first_node_neighbours.next();
                second_node_neighbour = second_node_neighbours.next();
            } else if first_node_neighbour_value < second_node_neighbour_value {
                result.push(first_node_neighbour_value);
                first_node_neighbour = first_node_neighbours.next();
            } else {
                second_node_neighbour = second_node_neighbours.next();
            }
        }

        // We need to add the remaining neighbours of the first node.
        while let Some(first_node_neighbour_value) = first_node_neighbour {
            result.push(first_node_neighbour_value);
            first_node_neighbour = first_node_neighbours.next();
        }

        result.into_iter()
    }

    /// Returns the intersection of the neighbours of two given nodes and a given label.
    /// 
    /// # Arguments
    /// * `first_node` - The first node whose neighbours should be intersected.
    /// * `second_node` - The second node whose neighbours should be intersected.
    /// * `label` - The label of the neighbours to intersect.
    /// 
    /// # Implementation details
    /// We assume that the provided node neighbours are sorted.
    fn get_intersection_of_neighbours_of_label(
        &self,
        first_node: usize,
        second_node: usize,
        label: Self::NodeLabel,
    ) -> impl Iterator<Item = usize> + '_ {
        let mut first_node_neighbours = self.iter_neighbours_of_label(first_node, label);
        let mut second_node_neighbours = self.iter_neighbours_of_label(second_node, label);

        let mut result = Vec::new();

        let mut first_node_neighbour = first_node_neighbours.next();
        let mut second_node_neighbour = second_node_neighbours.next();

        while let (Some(first_node_neighbour_value), Some(second_node_neighbour_value)) =
            (first_node_neighbour, second_node_neighbour)
        {
            if first_node_neighbour_value == second_node_neighbour_value {
                result.push(first_node_neighbour_value);
                first_node_neighbour = first_node_neighbours.next();
                second_node_neighbour = second_node_neighbours.next();
            } else if first_node_neighbour_value < second_node_neighbour_value {
                first_node_neighbour = first_node_neighbours.next();
            } else {
                second_node_neighbour = second_node_neighbours.next();
            }
        }

        result.into_iter()
    }

    /// Returns the node label of the given node.
    ///
    /// # Arguments
    ///
    /// * `node` - The node whose label should be returned.
    fn get_node_label(&self, node: usize) -> Self::NodeLabel;
}
