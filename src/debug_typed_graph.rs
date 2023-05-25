use crate::graph::TypedGraph;

/// This trait is only used for debugging purposes.
/// It is exclusively used for assertions.
pub(crate) struct DebugTypedGraph<G> {
    graph: G,
}

impl<'a, G> From<&'a G> for DebugTypedGraph<&'a G> {
    fn from(graph: &'a G) -> Self {
        Self { graph }
    }
}

impl<'a, G> DebugTypedGraph<&'a G>
where
    G: TypedGraph,
{
    /// Iterate the neighbours of a node solely of the given label.
    ///
    /// # Arguments
    /// * `node` - The node whose neighbours should be iterated over.
    /// * `label` - The label of the neighbours to iterate over.
    pub(crate) fn iter_neighbours_of_label(
        &self,
        node: usize,
        label: G::NodeLabel,
    ) -> impl Iterator<Item = usize> + '_ {
        self.graph.iter_neighbours(node).filter(move |neighbour| {
            debug_assert!(
                node != *neighbour,
                "A node cannot be neighbour of itself, but {} is neighbour of {}",
                node,
                neighbour
            );
            self.graph.get_node_label(*neighbour) == label
        })
    }

    /// Returns the subtraction of the neighbours of two given nodes.
    ///
    /// # Arguments
    /// * `first_node` - The first node whose neighbours should be from.
    /// * `second_node` - The second node whose neighbours should be subtracted.
    ///
    /// # Implementation details
    /// We assume that the provided node neighbours are sorted.
    pub(crate) fn get_subtraction_of_neighbours(
        &self,
        first_node: usize,
        second_node: usize,
    ) -> impl Iterator<Item = usize> + '_ {
        let mut first_node_neighbours = self.graph.iter_neighbours(first_node);
        let mut second_node_neighbours = self.graph.iter_neighbours(second_node);

        let mut result = Vec::new();

        let mut first_node_neighbour = first_node_neighbours.next();
        let mut second_node_neighbour = second_node_neighbours.next();

        while let (Some(first_node_neighbour_value), Some(second_node_neighbour_value)) =
            (first_node_neighbour, second_node_neighbour)
        {
            if first_node_neighbour_value == second_node {
                first_node_neighbour = first_node_neighbours.next();
                continue;
            }
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
            if first_node_neighbour_value == second_node {
                first_node_neighbour = first_node_neighbours.next();
                continue;
            }
            result.push(first_node_neighbour_value);
            first_node_neighbour = first_node_neighbours.next();
        }

        result.into_iter()
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
    pub(crate) fn get_subtraction_of_neighbours_of_label(
        &self,
        first_node: usize,
        second_node: usize,
        label: G::NodeLabel,
    ) -> impl Iterator<Item = usize> + '_ {
        let mut first_node_neighbours = self.iter_neighbours_of_label(first_node, label);
        let mut second_node_neighbours = self.iter_neighbours_of_label(second_node, label);

        let mut result = Vec::new();

        let mut first_node_neighbour = first_node_neighbours.next();
        let mut second_node_neighbour = second_node_neighbours.next();

        while let (Some(first_node_neighbour_value), Some(second_node_neighbour_value)) =
            (first_node_neighbour, second_node_neighbour)
        {
            if first_node_neighbour_value == second_node || first_node_neighbour_value == first_node
            {
                first_node_neighbour = first_node_neighbours.next();
                continue;
            }
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
            if first_node_neighbour_value == second_node || first_node_neighbour_value == first_node
            {
                first_node_neighbour = first_node_neighbours.next();
                continue;
            }
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
        label: G::NodeLabel,
    ) -> impl Iterator<Item = usize> + '_ {
        let mut first_node_neighbours = self.iter_neighbours_of_label(first_node, label);
        let mut second_node_neighbours = self.iter_neighbours_of_label(second_node, label);

        let mut result = Vec::new();

        let mut first_node_neighbour = first_node_neighbours.next();
        let mut second_node_neighbour = second_node_neighbours.next();

        while let (Some(first_node_neighbour_value), Some(second_node_neighbour_value)) =
            (first_node_neighbour, second_node_neighbour)
        {
            if first_node_neighbour_value == second_node || first_node_neighbour_value == first_node
            {
                first_node_neighbour = first_node_neighbours.next();
                continue;
            }
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

    pub(crate) fn get_intersection_size_of_label(
        &self,
        src: usize,
        dst: usize,
        label: G::NodeLabel,
    ) -> usize {
        self.get_intersection_of_neighbours_of_label(src, dst, label)
            .count()
    }
}
