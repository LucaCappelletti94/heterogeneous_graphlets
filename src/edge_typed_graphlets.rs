use std::{
    collections::HashMap,
    ops::{Add, Div, Mul, Rem},
};

use crate::{
    graphlet_counter::GraphLetCounter, perfect_hash::PerfectHash, prelude::*,
    utils::NumericalConstants,
};

const NOT_UPDATED: usize = usize::MAX;

pub trait HeterogeneousGraphlets: Orbit
where
    Self::NodeLabel: NumericalConstants
        + Mul<Self::NodeLabel, Output = Self::NodeLabel>
        + Add<Self::NodeLabel, Output = Self::NodeLabel>
        + Div<Self::NodeLabel, Output = Self::NodeLabel>
        + Rem<Self::NodeLabel, Output = Self::NodeLabel>
        + Copy,
    (
        Self::NodeLabel,
        Self::NodeLabel,
        Self::NodeLabel,
        Self::NodeLabel,
    ): PerfectHash<Self::NodeLabel> + Sized,
{
    type GraphLetCounter: GraphLetCounter<Self::NodeLabel>;
    const FOUR_CLIQUE: Self::NodeLabel = Self::NodeLabel::TWELVE;
    const CHORDAL_CYCLE_CENTER_ORBIT: Self::NodeLabel = Self::NodeLabel::ELEVEN;
    const CHORDAL_CYCLE_EDGE_ORBIT: Self::NodeLabel = Self::NodeLabel::TEN;
    const TAILED_TRIANGLE_TRI_EDGE_ORBIT: Self::NodeLabel = Self::NodeLabel::NINE;
    const TAILED_TRI_CENTER_ORBIT: Self::NodeLabel = Self::NodeLabel::EIGHT;
    const TAILED_TRI_TAIL_ORBIT: Self::NodeLabel = Self::NodeLabel::SEVEN;
    const FOUR_CYCLE: Self::NodeLabel = Self::NodeLabel::SIX;
    const FOUR_STAR_ORBIT: Self::NodeLabel = Self::NodeLabel::FIVE;
    const FOUR_PATH_CENTER_ORBIT: Self::NodeLabel = Self::NodeLabel::FOUR;
    const FOUR_PATH_EDGE_ORBIT: Self::NodeLabel = Self::NodeLabel::THREE;
    const TRIANGLE: Self::NodeLabel = Self::NodeLabel::TWO;
    const TRIAD: Self::NodeLabel = Self::NodeLabel::ONE;

    /// Returns the number of graphlets of the provided edge.
    ///
    /// # Arguments
    /// * `src` - The source node of the edge.
    /// * `dst` - The destination node of the edge.
    ///
    /// # Example
    /// We use the random graph object to test some executions of the function.
    ///
    /// ```
    /// use heterogeneous_graphlets::prelude::*;
    ///
    /// let graph = RandomGraph::new(42, 10, 2, 5);
    ///
    /// graph.iter_edges().for_each(|(src, dst)| {
    ///     let graphlet_count = graph.get_heterogeneous_graphlet(src, dst);
    /// });
    ///
    /// ```
    ///
    fn get_heterogeneous_graphlet(&self, src: usize, dst: usize) -> Self::GraphLetCounter {
        // We allocate the graphlet set for the unique rare graphlets.
        let mut graphlet_counter = <Self::GraphLetCounter>::default();

        // We get the iterator of the neighbours of the source and destination nodes.
        // We observe that the iterators are sorted.
        let mut src_iter = self.iter_neighbours(src).peekable();
        let mut dst_iter = self.iter_neighbours(dst).peekable();

        // We get the node labels of the source and destination nodes.
        let src_node_type = self.get_node_label(src);
        let dst_node_type = self.get_node_label(dst);

        // We allocate counters for the node labels of triangles:
        let mut triangle_labels_counts = vec![0; self.get_number_of_node_labels_usize()];
        // Similarly, we allocate counters for the node labels of the source and destination neighbours
        // that are solely neighbours of the source or destination nodes.
        let mut src_neighbour_labels_counts = vec![0; self.get_number_of_node_labels_usize()];
        let mut dst_neighbour_labels_counts = vec![0; self.get_number_of_node_labels_usize()];

        // We define here the function used to handle the cases for the typed paths, as it will be
        // necessary to invoce such function multiple times.
        let handle_src_rooted_typed_paths =
            |root: usize,
             graphlet_counter: &mut Self::GraphLetCounter,
             src_neighbour_labels_counts: &mut [usize]| {
                // We increment the counter of the node label of the source neighbour.
                src_neighbour_labels_counts
                    [self.get_number_of_node_label_index(self.get_node_label(root))] += 1;

                // We have found a 3-path, which can also be called a 3-star.
                // We compute the hash associated to the 3-star graphlet and insert it into the graphlet counter.
                graphlet_counter.insert(
                    (
                        src_node_type,
                        dst_node_type,
                        self.get_node_label(root),
                        // A 3-star has only 3 possible node types characterizing it.
                        // Thus, we can use the last node label as a dummy value.
                        self.get_number_of_node_labels(),
                    )
                        .encode(Self::TRIAD, self.get_number_of_node_labels()),
                );

                // We start to iterate over the neighbours of the provided root node.
                // The neighbouring nodes must not be equal to the source or destination nodes.
                // Since we need to intersect these second-order neighbours with the
                // neighbours of the source and destination nodes, we need to do a secondary
                // internal iteration on the neighbours of source and destination nodes.
                // The conditions we will be checking are the two following:
                // 1. The second order neighbour is NOT a neighbour of either the source or destination nodes.
                //    We will check this condition by exploiting the sorted nature of the iterators, as any time
                //    we will encounter a second order neighbour that is smaller than the source or destination neighbouring
                //    nodes, we will know that it will never appear again in the source or destination iterators.
                //    When this condition is true, we will have identified a typed 4-path-edge orbit.
                // 2. The second order neighbour is a neighbour of SOLELY THE SOURCE NODE and NOT of the destination node and
                //    it is lower or equal to the provided root node. We will check this condition by exploiting the sorted
                //    nature of the iterators, as any time we will encounter a second order neighbour that is smaller than
                //    the source neighbouring node, we will know that it will never appear again in the source iterator.
                //    In order to check that the second order neighbour is not a neighbour of the destination node, we will
                //    only enter in condition (2) if the second order neighbour is smaller than the destination neighbouring node.
                //    When this condition is true, we will have identified a typed tailed-tri-tail orbit.
                let mut second_order_iterator = self.iter_neighbours(root).peekable();
                let mut src_second_order_iterator = self.iter_neighbours(src).peekable();
                let mut dst_second_order_iterator = self.iter_neighbours(dst).peekable();

                // To check for the first condition, we need to know the last seen values of the source and destination iterators
                // as the iterators are sorted. This is necessary because the first condition requires for the second order neighbour
                // to NOT appear in the source or destination iterators, and if the value is lower than the value of the source or
                // destination iterators, it will never appear again, and thus it will never appear in the source or destination iterators.

                // These values are surely updated immediately, so for better code clarity we initialize them with a value that is
                // quite clear instead of using any dummy value.
                let mut last_src_neighbour = NOT_UPDATED;
                let mut last_dst_neighbour = NOT_UPDATED;

                // We iterate over the second order neighbours of the root node.
                while let Some(&second_order_neighbour) = second_order_iterator.peek() {
                    // We skip the second order neighbour if it is the same as the source or destination nodes.
                    if second_order_neighbour == src || second_order_neighbour == dst {
                        second_order_iterator.advance_by(1).unwrap();
                        continue;
                    }

                    // If the second order neighbour is larger than the source node,
                    // we increase the iterator of the source node.
                    if let Some(&second_order_src) = src_second_order_iterator.peek() {
                        if second_order_neighbour > second_order_src {
                            src_second_order_iterator.advance_by(1).unwrap();
                            continue;
                        }
                        last_src_neighbour = second_order_src;
                    }

                    // Similarly, if the second order neighbour is larger than the destination node,
                    // we increase the iterator of the destination node.
                    if let Some(&second_order_dst) = dst_second_order_iterator.peek() {
                        if second_order_neighbour > second_order_dst {
                            dst_second_order_iterator.advance_by(1).unwrap();
                            continue;
                        }
                        last_dst_neighbour = second_order_dst;
                    }

                    // If the second order neighbour is smaller than both the source and destination neighbouring nodes,
                    // it means that it is not a neighbour of either the source or destination nodes as the iterators are sorted
                    // and the second order neighbour will never appear again in the source or destination iterators.
                    if second_order_neighbour < last_src_neighbour
                        && second_order_neighbour < last_dst_neighbour
                    {
                        // We compute the hash associated to the 4-path-edge orbit
                        // and insert it into the graphlet counter.
                        graphlet_counter.insert(
                            (
                                src_node_type,
                                dst_node_type,
                                self.get_node_label(second_order_neighbour),
                                self.get_node_label(root),
                            )
                                .encode(
                                    Self::FOUR_PATH_EDGE_ORBIT,
                                    self.get_number_of_node_labels(),
                                ),
                        );

                        // Now we can increase the iterator of the second order neighbours.
                        second_order_iterator.advance_by(1).unwrap();

                        continue;
                    }

                    // Alternatively, if the second order neighbour is EQUAL TO the source neighbouring node and
                    // SMALLER THAN the destination neighbouring node, it means that it is a neighbour of SOLELY
                    // THE SOURCE NODE and NOT of the destination node.
                    if second_order_neighbour == last_src_neighbour
                        && second_order_neighbour < last_dst_neighbour
                        && second_order_neighbour <= root
                    {
                        // We compute the hash associated to the tailed-tri-tail orbit
                        // and insert it into the graphlet counter.
                        graphlet_counter.insert(
                            (
                                src_node_type,
                                dst_node_type,
                                self.get_node_label(second_order_neighbour),
                                self.get_node_label(root),
                            )
                                .encode(
                                    Self::TAILED_TRI_TAIL_ORBIT,
                                    self.get_number_of_node_labels(),
                                ),
                        );

                        // Now we can increase the iterator of the second order neighbours
                        // and the source second order neighbours.
                        src_second_order_iterator.advance_by(1).unwrap();
                        second_order_iterator.advance_by(1).unwrap();

                        continue;
                    }

                    second_order_iterator.advance_by(1).unwrap();
                }
            };
        let handle_dst_rooted_typed_paths =
            |root: usize,
             graphlet_counter: &mut Self::GraphLetCounter,
             dst_neighbour_labels_counts: &mut [usize]| {
                // We increment the counter of the node label of the destination neighbour.
                dst_neighbour_labels_counts
                    [self.get_number_of_node_label_index(self.get_node_label(root))] += 1;

                // We have found a 3-path, which can also be called a 3-star.
                // We compute the hash associated to the 3-star graphlet and insert it into the graphlet counter.
                graphlet_counter.insert(
                    (
                        src_node_type,
                        dst_node_type,
                        self.get_node_label(root),
                        // A 3-star has only 3 possible node types characterizing it.
                        // Thus, we can use the last node label as a dummy value.
                        self.get_number_of_node_labels(),
                    )
                        .encode(Self::TRIAD, self.get_number_of_node_labels()),
                );

                // We start to iterate over the neighbours of the provided root node.
                // The neighbouring nodes must not be equal to the source or destination nodes.
                // Since we need to intersect these second-order neighbours with the
                // neighbours of the source and destination nodes, we need to do a secondary
                // internal iteration on the neighbours of source and destination nodes.
                // The conditions we will be checking, similar but in part complementary to the ones described in the
                // source node rooted function, are the three following:
                // 1. This condition is identical to the one described in the source node rooted function.
                // 2. Complementarily to the condition (2) described in the source node rooted function, we will check
                //    that the second order neighbour is a neighbour of SOLELY THE DESTINATION NODE and NOT of the source node and
                //    it is lower or equal to the provided root node. We will check this condition by exploiting the sorted
                //    nature of the iterators, as any time we will encounter a second order neighbour that is smaller than
                //    the destination neighbouring node, we will know that it will never appear again in the destination iterator.
                //    In order to check that the second order neighbour is not a neighbour of the source node, we will
                //    only enter in condition (2) if the second order neighbour is smaller than the source neighbouring node.
                //    When this condition is true, we will have identified a typed tailed-tri-tail orbit.
                // 3. This third condition only appears in the destination node rooted function. We will check that the second
                //    order neighbour is a neighbour of SOLELY THE SOURCE NODE and NOT of the destination node. Since we are executing
                //    the symmetric check in the condition (2) of this function, we will only enter in this condition if the second
                //    order neighbour is smaller than the destination neighbouring node and equal to the source neighbouring node.
                //    When this condition is true, we will have identified a typed 4-cycle.
                let mut second_order_iterator = self.iter_neighbours(root).peekable();
                let mut src_second_order_iterator = self.iter_neighbours(src).peekable();
                let mut dst_second_order_iterator = self.iter_neighbours(dst).peekable();

                // To check for the first condition, we need to know the last seen values of the source and destination iterators
                // as the iterators are sorted. This is necessary because the first condition requires for the second order neighbour
                // to NOT appear in the source or destination iterators, and if the value is lower than the value of the source or
                // destination iterators, it will never appear again, and thus it will never appear in the source or destination iterators.

                // These values are surely updated immediately, so for better code clarity we initialize them with a value that is
                // quite clear instead of using any dummy value.

                let mut last_src_neighbour = NOT_UPDATED;
                let mut last_dst_neighbour = NOT_UPDATED;

                // We iterate over the second order neighbours of the root node.

                // We iterate over the second order neighbours of the root node.
                while let Some(&second_order_neighbour) = second_order_iterator.peek() {
                    // We skip the second order neighbour if it is the same as the source or destination nodes.
                    if second_order_neighbour == src || second_order_neighbour == dst {
                        second_order_iterator.advance_by(1).unwrap();
                        continue;
                    }

                    // If the second order neighbour is larger than the source node,
                    // we increase the iterator of the source node.
                    if let Some(&second_order_src) = src_second_order_iterator.peek() {
                        if second_order_neighbour > second_order_src {
                            src_second_order_iterator.advance_by(1).unwrap();
                            continue;
                        }
                        last_src_neighbour = second_order_src;
                    }

                    // Similarly, if the second order neighbour is larger than the destination node,
                    // we increase the iterator of the destination node.
                    if let Some(&second_order_dst) = dst_second_order_iterator.peek() {
                        if second_order_neighbour > second_order_dst {
                            dst_second_order_iterator.advance_by(1).unwrap();
                            continue;
                        }
                        last_dst_neighbour = second_order_dst;
                    }

                    // If the second order neighbour is smaller than both the source and destination neighbouring nodes,
                    // it means that it is not a neighbour of either the source or destination nodes as the iterators are sorted
                    // and the second order neighbour will never appear again in the source or destination iterators.
                    if second_order_neighbour < last_src_neighbour
                        && second_order_neighbour < last_dst_neighbour
                    {
                        // We compute the hash associated to the 4-path-edge orbit
                        // and insert it into the graphlet counter.
                        graphlet_counter.insert(
                            (
                                src_node_type,
                                dst_node_type,
                                self.get_node_label(second_order_neighbour),
                                self.get_node_label(root),
                            )
                                .encode(
                                    Self::FOUR_PATH_EDGE_ORBIT,
                                    self.get_number_of_node_labels(),
                                ),
                        );

                        // Now we can increase the iterator of the second order neighbours.
                        second_order_iterator.advance_by(1).unwrap();

                        continue;
                    }

                    // Alternatively, if the second order neighbour is EQUAL TO the destination neighbouring node and
                    // SMALLER THAN the source neighbouring node, it means that it is a neighbour of SOLELY
                    // THE DESTINATION NODE and NOT of the source node.
                    if second_order_neighbour == last_dst_neighbour
                        && second_order_neighbour < last_src_neighbour
                        && second_order_neighbour <= root
                    {
                        // We compute the hash associated to the tailed-tri-tail orbit
                        // and insert it into the graphlet counter.
                        graphlet_counter.insert(
                            (
                                src_node_type,
                                dst_node_type,
                                self.get_node_label(second_order_neighbour),
                                self.get_node_label(root),
                            )
                                .encode(
                                    Self::TAILED_TRI_TAIL_ORBIT,
                                    self.get_number_of_node_labels(),
                                ),
                        );

                        // Now we can increase the iterator of the second order neighbours
                        // and the source second order neighbours.
                        dst_second_order_iterator.advance_by(1).unwrap();
                        second_order_iterator.advance_by(1).unwrap();

                        continue;
                    }

                    // Finally, for the third option that is solely present in the destination node rooted function,
                    // we check that the second order neighbour is a neighbour of SOLELY THE SOURCE NODE and NOT of the destination node.

                    if second_order_neighbour == last_src_neighbour
                        && second_order_neighbour < last_dst_neighbour
                    {
                        // We compute the hash associated to the 4-cycle
                        graphlet_counter.insert(
                            (
                                src_node_type,
                                dst_node_type,
                                self.get_node_label(second_order_neighbour),
                                self.get_node_label(root),
                            )
                                .encode(Self::FOUR_CYCLE, self.get_number_of_node_labels()),
                        );

                        // Now we can increase the iterator of the second order neighbours
                        // and the source second order neighbours.
                        src_second_order_iterator.advance_by(1).unwrap();
                        second_order_iterator.advance_by(1).unwrap();

                        continue;
                    }

                    second_order_iterator.advance_by(1).unwrap();
                }
            };

        // We start to iterate over the neighbours of the source and destination nodes.
        while let (Some(&src_neighbour), Some(&dst_neighbour)) = (src_iter.peek(), dst_iter.peek())
        {
            // If the two neighbours are the same, we have identified a triangle.
            // TODO! PROPERLY HANDLE SELF-LOOP!
            if src_neighbour == dst_neighbour {
                // We get the node labels of the source only, as both have
                // necessarily the same node label.
                let node_neighbour_type = self.get_node_label(src_neighbour);

                // We increase the counter of the node label of the triangle.
                triangle_labels_counts[self.get_number_of_node_label_index(node_neighbour_type)] +=
                    1;

                // We insert the triangle into the graphlet counter.
                graphlet_counter.insert(
                    (
                        src_node_type,
                        dst_node_type,
                        node_neighbour_type,
                        // A triangle has only 3 possible node types characterizing it.
                        // Thus, we can use the last node label as a dummy value.
                        self.get_number_of_node_labels(),
                    )
                        .encode(Self::TRIANGLE, self.get_number_of_node_labels()),
                );

                // We iterate over the neighbours of the triangle node.
                // These nodes will be second-order neighbours of the source and destination nodes.
                let mut second_order_iterator = self.iter_neighbours(src_neighbour).peekable();
                // In order to check the following conditions, it is necessary to do a secondary
                // internal iteration on the neighbours of source and destination nodes.
                // Specifically, the conditions are as follows:
                //
                // 1. The second order neighbour is ALSO a neighbour of the source and destination nodes,
                //    that is, it also forms a triangle with the source and destination nodes.
                // 2. The second order neighbour DOES NOT form a triangle with the source and destination nodes
                //    but is a neighbour of source or destination nevertheless.
                // 3. The second order neighbour is NOT a neighbour of source or destination nodes.
                //
                // To check these conditions, we iterate over the neighbours of the source and destination nodes.
                let mut src_second_order_iterator = self.iter_neighbours(src).peekable();
                let mut dst_second_order_iterator = self.iter_neighbours(dst).peekable();

                // To check for the last condition, we need to know the last seen values of the source and destination iterators
                // as the iterators are sorted. This is necessary because the third condition requires for the second order neighbour
                // to NOT appear in the source or destination iterators, and if the value is lower than the value of the source or
                // destination iterators, it will never appear again, and thus it will never appear in the source or destination iterators.

                // These values are surely updated immediately, so for better code clarity we initialize them with a value that is
                // quite clear instead of using any dummy value.
                let mut last_src_neighbour = NOT_UPDATED;
                let mut last_dst_neighbour = NOT_UPDATED;

                // We iterate over the second order neighbours of the triangle node.
                while let Some(&second_order_neighbour) = second_order_iterator.peek() {
                    // We skip the second order neighbour if it is the same as the source or destination nodes.
                    if second_order_neighbour == src || second_order_neighbour == dst {
                        second_order_iterator.advance_by(1).unwrap();
                        continue;
                    }

                    // If the second order neighbour is less or equal to the triangle node,
                    if second_order_neighbour <= src_neighbour {
                        // we check the first condition, that is, if the second order neighbour is
                        // also a neighbour of the source and destination nodes and forms a triangle with them.
                        if let (Some(&second_order_src), Some(&second_order_dst)) = (
                            src_second_order_iterator.peek(),
                            dst_second_order_iterator.peek(),
                        ) {
                            // We update the last seen values of the source and destination iterators.
                            last_src_neighbour = second_order_src;
                            last_dst_neighbour = second_order_dst;

                            if second_order_neighbour == second_order_src
                                && second_order_neighbour == second_order_dst
                            {
                                // We compute the hash associated to the 4-clique graphlet
                                // and insert it into the graphlet counter.
                                graphlet_counter.insert(
                                    (
                                        src_node_type,
                                        dst_node_type,
                                        node_neighbour_type,
                                        self.get_node_label(second_order_src),
                                    )
                                        .encode(
                                            Self::FOUR_CLIQUE,
                                            self.get_number_of_node_labels(),
                                        ),
                                );

                                // Now we can update all involved iterators with the next value.
                                src_second_order_iterator.advance_by(1).unwrap();
                                dst_second_order_iterator.advance_by(1).unwrap();
                                second_order_iterator.advance_by(1).unwrap();

                                continue;
                            }
                        }
                    }

                    // Otherwise, we proceed with the second condition, that is, if the second order neighbour
                    // does not form a triangle with the source and destination nodes but is a neighbour of
                    // source or destination nevertheless.
                    if let Some(&second_order_src) = src_second_order_iterator.peek() {
                        // We update the last seen value of the source iterator.
                        last_src_neighbour = second_order_src;

                        if second_order_neighbour == second_order_src {
                            // In this case, we have identified a chord-cycle-edge orbit.
                            // We compute the hash associated to the chord-cycle-edge graphlet.
                            graphlet_counter.insert(
                                (
                                    src_node_type,
                                    dst_node_type,
                                    node_neighbour_type,
                                    self.get_node_label(second_order_src),
                                )
                                    .encode(
                                        Self::CHORDAL_CYCLE_EDGE_ORBIT,
                                        self.get_number_of_node_labels(),
                                    ),
                            );

                            // Now we can update all involved iterators with the next value.
                            src_second_order_iterator.advance_by(1).unwrap();
                            second_order_iterator.advance_by(1).unwrap();

                            continue;
                        } else {
                            src_second_order_iterator.advance_by(1).unwrap();
                        }
                    }
                    // We do the same for the destination node.
                    if let Some(&second_order_dst) = dst_second_order_iterator.peek() {
                        // We update the last seen value of the destination iterator.
                        last_dst_neighbour = second_order_dst;

                        if second_order_neighbour == second_order_dst {
                            // Again, in this case, we have identified a chord-cycle-edge orbit.
                            // We compute the hash associated to the chord-cycle-edge graphlet.
                            graphlet_counter.insert(
                                (
                                    src_node_type,
                                    dst_node_type,
                                    node_neighbour_type,
                                    self.get_node_label(second_order_dst),
                                )
                                    .encode(
                                        Self::CHORDAL_CYCLE_EDGE_ORBIT,
                                        self.get_number_of_node_labels(),
                                    ),
                            );

                            // Now we can update all involved iterators with the next value.
                            dst_second_order_iterator.advance_by(1).unwrap();
                            second_order_iterator.advance_by(1).unwrap();

                            continue;
                        } else {
                            dst_second_order_iterator.advance_by(1).unwrap();
                        }
                    }
                    // Otherwise, we proceed with the third condition, that is, if the second order neighbour
                    // is not a neighbour of source or destination nodes.
                    if second_order_neighbour < last_src_neighbour
                        && second_order_neighbour < last_dst_neighbour
                    {
                        // In this case, we have identified a tailed-triangle-center orbit.
                        // We compute the hash associated to the tailed-triangle-center graphlet.
                        graphlet_counter.insert(
                            (
                                src_node_type,
                                dst_node_type,
                                node_neighbour_type,
                                self.get_node_label(second_order_neighbour),
                            )
                                .encode(
                                    Self::TAILED_TRI_CENTER_ORBIT,
                                    self.get_number_of_node_labels(),
                                ),
                        );

                        // Now we can update all involved iterators with the next value.
                        second_order_iterator.advance_by(1).unwrap();

                        continue;
                    }
                }
                // We can now advance the two iterators of the source and destination nodes.
                src_iter.advance_by(1).unwrap();
                dst_iter.advance_by(1).unwrap();
            }
            // Otherwise, if the two neighbours are not the same, both
            // may compose a 3-path with the source and destination nodes.
            // Since we are iterating over sorted iterators, we can check
            // which one of the two nodes is the smallest. It may be the case that
            // the larger node will also appear in the other iterator, but because
            // of the sorted nature of the iterators we are sure that the smaller
            // will never appear in the other iterator.
            // TODO! CHECK IF THE NEIGHBOURS HAVE TO BE DIFFERENT FROM SOURCE AND DESTINATION NODES!
            else if src_neighbour < dst_neighbour {
                // If the source neighbour is smaller than the destination neighbour,
                // it forms a 3-path with the source and destination nodes.
                handle_src_rooted_typed_paths(
                    src_neighbour,
                    &mut graphlet_counter,
                    &mut src_neighbour_labels_counts,
                );

                // We update the iterator with the lesser of the two nodes, which
                // in this case is the source iterator:
                src_iter.advance_by(1).unwrap();
            } else if dst_neighbour < src_neighbour {
                // If the destination neighbour is smaller than the source neighbour,
                // it forms a 3-path with the source and destination nodes.
                handle_dst_rooted_typed_paths(
                    dst_neighbour,
                    &mut graphlet_counter,
                    &mut dst_neighbour_labels_counts,
                );

                // We update the iterator with the lesser of the two nodes, which
                // in this case is the destination iterator:
                dst_iter.advance_by(1).unwrap();
            } else {
                unreachable!(concat!(
                    "All the possible cases have been handled, this should never be reached. ",
                    "Please do open an issue on the GitHub repository if you see this message."
                ))
            }
        }
        // Finally, we need to check whether both iterators are finished. If this is not the case,
        // the source or destination neighbours are surely not present in each other's iterator
        // and they form a 3-path with the source and destination nodes.
        for src_neighbour in src_iter {
            handle_src_rooted_typed_paths(
                src_neighbour,
                &mut graphlet_counter,
                &mut src_neighbour_labels_counts,
            );
        }

        for dst_neighbour in dst_iter {
            handle_dst_rooted_typed_paths(
                dst_neighbour,
                &mut graphlet_counter,
                &mut dst_neighbour_labels_counts,
            );
        }

        // Now we are done with counting some of the triangle-based and path-based graphlets,
        // and we need to complete the process by counting the remaining graphlets with the
        // orbital counts as detailed in the "Heterogeneous Graphlets" paper, equations 19, 23, 26 and 30.

        // We start by iterating over the graph labels
        for rows_label in 0..self.get_number_of_node_labels_usize() {
            let number_of_triangles_with_rows_label = triangle_labels_counts[rows_label];

            debug_assert_eq!(
                number_of_triangles_with_rows_label,
                self.get_intersection_of_neighbours_of_label(
                    src,
                    dst,
                    self.get_number_of_node_label_from_usize(rows_label)
                )
                .count(),
                concat!(
                    "The number of triangles with the label {:?} is not equal to the number ",
                    "of neighbours of the source and destination nodes with the same label. ",
                    "We expected {:?} but found {:?}. The count vector is {:?}."
                ),
                self.get_node_label(rows_label),
                number_of_triangles_with_rows_label,
                self.get_intersection_of_neighbours_of_label(
                    src,
                    dst,
                    self.get_node_label(rows_label)
                )
                .count(),
                triangle_labels_counts
            );

            let number_of_src_neighbours_with_rows_label = src_neighbour_labels_counts[rows_label];

            debug_assert_eq!(
                number_of_src_neighbours_with_rows_label,
                self.get_subtraction_of_neighbours_of_label(src, dst, self.get_number_of_node_label_from_usize(rows_label))
                    .count(),
                concat!(
                    "The number of neighbours of the source node with the label {:?} is not equal to the number ",
                    "of neighbours of the source node with the same label. ",
                    "We expected {:?} but found {:?}. The count vector is {:?}. ",
                    "The neighbours of source of the current label are {:?} and the neighbours of destination of the current label are {:?}."
                ),
                self.get_number_of_node_label_from_usize(rows_label),
                number_of_src_neighbours_with_rows_label,
                self.get_subtraction_of_neighbours_of_label(src, dst, self.get_number_of_node_label_from_usize(rows_label))
                    .count(),
                src_neighbour_labels_counts,
                self.iter_neighbours_of_label(src, self.get_number_of_node_label_from_usize(rows_label))
                    .collect::<Vec<_>>(),
                self.iter_neighbours_of_label(dst, self.get_number_of_node_label_from_usize(rows_label)).collect::<Vec<_>>()
            );

            let number_of_dst_neighbours_with_rows_label = dst_neighbour_labels_counts[rows_label];

            println!("{}", rows_label);

            debug_assert_eq!(
                number_of_dst_neighbours_with_rows_label,
                self.get_subtraction_of_neighbours_of_label(dst, src, self.get_number_of_node_label_from_usize(rows_label))
                    .count(),
                concat!(
                    "The number of neighbours of the destination node with the label {:?} is not equal to the number ",
                    "of neighbours of the destination node with the same label. ",
                    "We expected {:?} but found {:?}. The count vector is {:?}. ",
                    "The neighbours of source of the current label are {:?} and the neighbours of destination of the current label are {:?}. ",
                    "The subtraction of the destination neighbours minus the source neighbours is {:?}."
                ),
                self.get_number_of_node_label_from_usize(rows_label),
                number_of_dst_neighbours_with_rows_label,
                self.get_subtraction_of_neighbours_of_label(dst, src, self.get_number_of_node_label_from_usize(rows_label))
                    .count(),
                dst_neighbour_labels_counts,
                self.iter_neighbours_of_label(src, self.get_number_of_node_label_from_usize(rows_label))
                    .collect::<Vec<_>>(),
                self.iter_neighbours_of_label(dst, self.get_number_of_node_label_from_usize(rows_label)).collect::<Vec<_>>(),
                self.get_subtraction_of_neighbours_of_label(dst, src, self.get_number_of_node_label_from_usize(rows_label))
                    .collect::<Vec<_>>()
            );

            // We iterate on the upper triangular matrix of the triangle labels counts.
            for columns_label in rows_label..self.get_number_of_node_labels_usize() {
                let number_of_triangles_with_columns_label = triangle_labels_counts[columns_label];
                let number_of_src_neighbours_with_columns_label =
                    src_neighbour_labels_counts[columns_label];
                let number_of_dst_neighbours_with_columns_label =
                    dst_neighbour_labels_counts[columns_label];

                // We need to retrieve the number of graphlets for the combination of labels
                // (source node label, destination node label, rows label, columns label),
                // for the four cycles, tailed-tri-tail, chord-cycle-edge and four-clique orbits.
                let number_of_four_cycles = graphlet_counter.get_number_of_graphlets(
                    (
                        src_node_type,
                        dst_node_type,
                        self.get_number_of_node_label_from_usize(rows_label),
                        self.get_number_of_node_label_from_usize(columns_label),
                    )
                        .encode(Self::FOUR_CYCLE, self.get_number_of_node_labels()),
                );
                let number_of_tailed_tri_tails = graphlet_counter.get_number_of_graphlets(
                    (
                        src_node_type,
                        dst_node_type,
                        self.get_number_of_node_label_from_usize(rows_label),
                        self.get_number_of_node_label_from_usize(columns_label),
                    )
                        .encode(
                            Self::TAILED_TRI_TAIL_ORBIT,
                            self.get_number_of_node_labels(),
                        ),
                );
                let number_of_chordal_cycle_edges = graphlet_counter.get_number_of_graphlets(
                    (
                        src_node_type,
                        dst_node_type,
                        self.get_number_of_node_label_from_usize(rows_label),
                        self.get_number_of_node_label_from_usize(columns_label),
                    )
                        .encode(
                            Self::CHORDAL_CYCLE_EDGE_ORBIT,
                            self.get_number_of_node_labels(),
                        ),
                );
                let number_of_four_cliques = graphlet_counter.get_number_of_graphlets(
                    (
                        src_node_type,
                        dst_node_type,
                        self.get_number_of_node_label_from_usize(rows_label),
                        self.get_number_of_node_label_from_usize(columns_label),
                    )
                        .encode(Self::FOUR_CLIQUE, self.get_number_of_node_labels()),
                );

                // Now we have all ingredients to compute the number of graphlets for the
                // graphlets (4), (5), (9) and (11), which are four-path center orbits,
                // four-star orbits, tailed triangle tri-edge orbits and chordal cycle center orbits.

                // We start with the four-path center orbits.
                let number_of_four_path_center_orbits = self.get_typed_four_path_orbit_count(
                    number_of_four_cycles,
                    self.get_number_of_node_label_from_usize(rows_label),
                    self.get_number_of_node_label_from_usize(columns_label),
                    number_of_src_neighbours_with_rows_label,
                    number_of_dst_neighbours_with_rows_label,
                    number_of_src_neighbours_with_columns_label,
                    number_of_dst_neighbours_with_columns_label,
                );

                // We update the graphlet counter with the number of four-path center orbits.
                graphlet_counter.insert_count(
                    (
                        src_node_type,
                        dst_node_type,
                        self.get_number_of_node_label_from_usize(rows_label),
                        self.get_number_of_node_label_from_usize(columns_label),
                    )
                        .encode(
                            Self::FOUR_PATH_CENTER_ORBIT,
                            self.get_number_of_node_labels(),
                        ),
                    number_of_four_path_center_orbits,
                );

                // We continue with the four-star orbits.
                let number_of_four_star_orbits = self.get_typed_four_star_orbit_count(
                    number_of_tailed_tri_tails,
                    self.get_number_of_node_label_from_usize(rows_label),
                    self.get_number_of_node_label_from_usize(columns_label),
                    number_of_src_neighbours_with_rows_label,
                    number_of_dst_neighbours_with_rows_label,
                    number_of_src_neighbours_with_columns_label,
                    number_of_dst_neighbours_with_columns_label,
                );

                // We update the graphlet counter with the number of four-star orbits.
                graphlet_counter.insert_count(
                    (
                        src_node_type,
                        dst_node_type,
                        self.get_number_of_node_label_from_usize(rows_label),
                        self.get_number_of_node_label_from_usize(columns_label),
                    )
                        .encode(Self::FOUR_STAR_ORBIT, self.get_number_of_node_labels()),
                    number_of_four_star_orbits,
                );

                // We continue with the tailed triangle tri-edge orbits.
                let number_of_tailed_tri_tri_edge_orbits = self
                    .get_typed_tailed_triangle_tri_edge_orbit_count(
                        number_of_chordal_cycle_edges,
                        self.get_number_of_node_label_from_usize(rows_label),
                        self.get_number_of_node_label_from_usize(columns_label),
                        number_of_triangles_with_rows_label,
                        number_of_triangles_with_columns_label,
                        number_of_src_neighbours_with_rows_label,
                        number_of_dst_neighbours_with_rows_label,
                        number_of_src_neighbours_with_columns_label,
                        number_of_dst_neighbours_with_columns_label,
                    );

                // We update the graphlet counter with the number of tailed triangle tri-edge orbits.
                graphlet_counter.insert_count(
                    (
                        src_node_type,
                        dst_node_type,
                        self.get_number_of_node_label_from_usize(rows_label),
                        self.get_number_of_node_label_from_usize(columns_label),
                    )
                        .encode(
                            Self::TAILED_TRIANGLE_TRI_EDGE_ORBIT,
                            self.get_number_of_node_labels(),
                        ),
                    number_of_tailed_tri_tri_edge_orbits,
                );

                // We continue with the chordal cycle center orbits.
                let number_of_chordal_cycle_center_orbits = self
                    .get_typed_chordal_cycle_center_orbit_count(
                        number_of_four_cliques,
                        self.get_number_of_node_label_from_usize(rows_label),
                        self.get_number_of_node_label_from_usize(columns_label),
                        number_of_triangles_with_rows_label,
                        number_of_triangles_with_columns_label,
                    );

                // We update the graphlet counter with the number of chordal cycle center orbits.
                graphlet_counter.insert_count(
                    (
                        src_node_type,
                        dst_node_type,
                        self.get_number_of_node_label_from_usize(rows_label),
                        self.get_number_of_node_label_from_usize(columns_label),
                    )
                        .encode(
                            Self::CHORDAL_CYCLE_CENTER_ORBIT,
                            self.get_number_of_node_labels(),
                        ),
                    number_of_chordal_cycle_center_orbits,
                );
            }
        }
        // We return the graphlet counter.
        graphlet_counter
    }
}

impl HeterogeneousGraphlets for RandomGraph {
    type GraphLetCounter = HashMap<usize, usize>;
}
