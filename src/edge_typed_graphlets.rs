#[cfg(debug_assertions)]
use alloc::vec::Vec;
use core::fmt::Debug;
use core::ops::{Add, AddAssign, Div, Mul, Rem, Sub};

use crate::graphlet_set::{ExtendedGraphletType, GraphletSet, ReducedGraphletType};
use crate::numbers::Two;
use crate::orbits::{
    get_heterogeneously_typed_chordal_cycle_center_orbit_count,
    get_heterogeneously_typed_four_path_orbit_count,
    get_heterogeneously_typed_four_star_orbit_count,
    get_heterogeneously_typed_tailed_triangle_tri_edge_orbit_count,
    get_homogeneously_typed_chordal_cycle_center_orbit_count,
    get_homogeneously_typed_four_path_orbit_count, get_homogeneously_typed_four_star_orbit_count,
    get_homogeneously_typed_tailed_triangle_tri_edge_orbit_count,
};
use crate::{
    graphlet_counter::GraphLetCounter, perfect_graphlet_hash::PerfectGraphletHash, prelude::*,
};
use num_traits::{AsPrimitive, Bounded, One, Zero};

#[cfg(debug_assertions)]
use crate::debug_typed_graph::DebugTypedGraph;

/// Counting of typed 4-node graphlet orbits incident to each edge of a
/// [`TypedGraph`].
///
/// `Graphlet` is the integer type used for the perfect-hash key of a typed
/// graphlet, and `Count` is the integer type used to tally occurrences.
pub trait HeterogeneousGraphlets<Graphlet, Count>: TypedGraph
where
    Count: Debug
        + Copy
        + 'static
        + Ord
        + One
        + Two
        + Zero
        + AddAssign
        + Add<Count, Output = Count>
        + Sub<Count, Output = Count>
        + Div<Count, Output = Count>
        + Mul<Count, Output = Count>
        + Rem<Count, Output = Count>,
    usize: AsPrimitive<Count>,
    Self: Sized,
    Graphlet: Copy
        + Debug
        + 'static
        + Bounded
        + AsPrimitive<u128>
        + From<ReducedGraphletType>
        + From<ExtendedGraphletType>
        + Mul<Output = Graphlet>
        + Add<Output = Graphlet>
        + Div<Output = Graphlet>
        + Rem<Output = Graphlet>
        + Sub<Output = Graphlet>
        + One
        + Zero
        + Ord,
    Self::NodeLabel: Ord
        + One
        + Zero
        + 'static
        + AsPrimitive<Graphlet>
        + AsPrimitive<u128>
        + Mul<Self::NodeLabel, Output = Self::NodeLabel>
        + Add<Self::NodeLabel, Output = Self::NodeLabel>
        + Div<Self::NodeLabel, Output = Self::NodeLabel>
        + Rem<Self::NodeLabel, Output = Self::NodeLabel>
        + Copy,
    ReducedGraphletType: GraphletSet<Graphlet> + From<Graphlet>,
    ExtendedGraphletType: GraphletSet<Graphlet> + From<Graphlet>,
    (
        Self::NodeLabel,
        Self::NodeLabel,
        Self::NodeLabel,
        Self::NodeLabel,
    ): PerfectGraphletHash<Graphlet, Self::NodeLabel> + Sized,
{
    /// The accumulator used to collect graphlet counts for an edge.
    type GraphLetCounter: GraphLetCounter<Graphlet, Count>;

    #[inline(always)]
    /// Returns the number of graphlets of the provided edge.
    ///
    /// # Arguments
    /// * `src` - The source node of the edge.
    /// * `dst` - The destination node of the edge.
    ///
    fn get_heterogeneous_graphlet(&self, src: usize, dst: usize) -> Self::GraphLetCounter {
        // We verify that the chosen Graphlet integer type is wide enough to hold
        // every possible graphlet hash for this graph. The bound is computed in
        // u128 (rather than in the Graphlet type, whose own arithmetic could
        // overflow and defeat the check) and asserted on every build, not just
        // in debug: a violation would otherwise silently produce wrong counts
        // through integer wraparound in release. The bound depends only on the
        // label count and the chosen types, so it is constant across edges.
        let number_of_labels: u128 = self.get_number_of_node_labels().as_();
        let maximal_hash_as_u128: u128 =
            <ExtendedGraphletType as GraphletSet<u128>>::get_number_of_graphlets()
                * number_of_labels.pow(4)
                + number_of_labels.pow(4)
                + number_of_labels.pow(3)
                + number_of_labels.pow(2)
                + number_of_labels;
        let maximal_graphlet_as_u128: u128 = Graphlet::max_value().as_();
        assert!(
            maximal_hash_as_u128 <= maximal_graphlet_as_u128,
            concat!(
                "The maximal hash value of the provided graphlet type is larger than the ",
                "maximum value of the graphlet type. This means that the graphlet type ",
                "cannot be encoded in the provided graphlet type. Specifically, the ",
                "maximum hash value is {:?}, while the maximum graphlet value is {:?}."
            ),
            maximal_hash_as_u128,
            maximal_graphlet_as_u128
        );

        // We allocate the graphlet set for the unique rare graphlets.
        let mut graphlet_counter =
            <Self::GraphLetCounter>::with_number_of_elements(self.get_number_of_node_labels());

        // We get the iterator of the neighbours of the source and destination nodes.
        // We observe that the iterators are sorted.
        let mut src_iter = self.iter_neighbours(src).peekable();
        let mut dst_iter = self.iter_neighbours(dst).peekable();

        // We get the node labels of the source and destination nodes.
        let src_node_type = self.get_node_label(src);
        let dst_node_type = self.get_node_label(dst);

        // We allocate counters for the node labels of triangles:
        let mut triangle_labels_counts =
            vec![Count::zero(); self.get_number_of_node_labels_usize()];
        // Similarly, we allocate counters for the node labels of the source and destination neighbours
        // that are solely neighbours of the source or destination nodes.
        let mut src_neighbour_labels_counts =
            vec![Count::zero(); self.get_number_of_node_labels_usize()];
        let mut dst_neighbour_labels_counts =
            vec![Count::zero(); self.get_number_of_node_labels_usize()];

        // We define here the function used to handle the cases for the typed paths, as it will be
        // necessary to invoce such function multiple times.
        let handle_src_rooted_typed_paths =
            |root: usize,
             graphlet_counter: &mut Self::GraphLetCounter,
             src_neighbour_labels_counts: &mut [Count]| {
                // We increment the counter of the node label of the source neighbour.
                src_neighbour_labels_counts
                    [self.get_node_label_index(self.get_node_label(root))] += Count::one();

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
                        .encode_with_graphlet::<ExtendedGraphletType>(
                            ExtendedGraphletType::Triad,
                            self.get_number_of_node_labels(),
                        ),
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
                // We classify every neighbour of the root node by whether it is
                // also a neighbour of the source and/or destination nodes. All
                // four neighbour lists are sorted, so we advance the source and
                // destination cursors monotonically to test membership in O(degree).
                let second_order_iterator = self.iter_neighbours(root);
                let mut src_second_order_iterator = self.iter_neighbours(src).peekable();
                let mut dst_second_order_iterator = self.iter_neighbours(dst).peekable();

                for second_order_neighbour in second_order_iterator {
                    // We skip the second order neighbour if it is the source or destination node.
                    if second_order_neighbour == src || second_order_neighbour == dst {
                        continue;
                    }

                    // We advance the source/destination cursors to the second order
                    // neighbour and check whether it is one of their neighbours.
                    while src_second_order_iterator
                        .peek()
                        .is_some_and(|&x| x < second_order_neighbour)
                    {
                        src_second_order_iterator.next();
                    }
                    let is_src_neighbour =
                        src_second_order_iterator.peek() == Some(&second_order_neighbour);
                    while dst_second_order_iterator
                        .peek()
                        .is_some_and(|&x| x < second_order_neighbour)
                    {
                        dst_second_order_iterator.next();
                    }
                    let is_dst_neighbour =
                        dst_second_order_iterator.peek() == Some(&second_order_neighbour);

                    if !is_src_neighbour && !is_dst_neighbour {
                        // The second order neighbour is a neighbour of neither the
                        // source nor the destination: the induced subgraph on
                        // {dst, src, root, second_order_neighbour} is a 4-path with
                        // edge (src, dst) at the end, i.e. a typed 4-path-edge orbit.
                        graphlet_counter.insert(
                            (
                                src_node_type,
                                dst_node_type,
                                self.get_node_label(second_order_neighbour),
                                self.get_node_label(root),
                            )
                                .encode_with_graphlet::<ExtendedGraphletType>(
                                    ExtendedGraphletType::FourPathEdge,
                                    self.get_number_of_node_labels(),
                                ),
                        );
                    } else if is_src_neighbour
                        && !is_dst_neighbour
                        && second_order_neighbour <= root
                    {
                        // The second order neighbour is a neighbour of solely the
                        // source node: it forms the triangle {src, root,
                        // second_order_neighbour} whose tail is edge (src, dst),
                        // i.e. a typed tailed-triangle tail-edge orbit. The
                        // `<= root` guard counts each such triangle once.
                        graphlet_counter.insert(
                            (
                                src_node_type,
                                dst_node_type,
                                self.get_node_label(second_order_neighbour),
                                self.get_node_label(root),
                            )
                                .encode_with_graphlet::<ExtendedGraphletType>(
                                    ExtendedGraphletType::TailedTriTail,
                                    self.get_number_of_node_labels(),
                                ),
                        );
                    }
                }
            };
        let handle_dst_rooted_typed_paths =
            |root: usize,
             graphlet_counter: &mut Self::GraphLetCounter,
             dst_neighbour_labels_counts: &mut [Count]| {
                // We increment the counter of the node label of the destination neighbour.
                dst_neighbour_labels_counts
                    [self.get_node_label_index(self.get_node_label(root))] += Count::one();

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
                        .encode_with_graphlet::<ExtendedGraphletType>(
                            ExtendedGraphletType::Triad,
                            self.get_number_of_node_labels(),
                        ),
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
                // As in the source-rooted case, classify every neighbour of the
                // root node by membership in the source/destination neighbourhoods
                // using monotonic cursors over the sorted lists.
                let second_order_iterator = self.iter_neighbours(root);
                let mut src_second_order_iterator = self.iter_neighbours(src).peekable();
                let mut dst_second_order_iterator = self.iter_neighbours(dst).peekable();

                for second_order_neighbour in second_order_iterator {
                    if second_order_neighbour == src || second_order_neighbour == dst {
                        continue;
                    }

                    while src_second_order_iterator
                        .peek()
                        .is_some_and(|&x| x < second_order_neighbour)
                    {
                        src_second_order_iterator.next();
                    }
                    let is_src_neighbour =
                        src_second_order_iterator.peek() == Some(&second_order_neighbour);
                    while dst_second_order_iterator
                        .peek()
                        .is_some_and(|&x| x < second_order_neighbour)
                    {
                        dst_second_order_iterator.next();
                    }
                    let is_dst_neighbour =
                        dst_second_order_iterator.peek() == Some(&second_order_neighbour);

                    if !is_src_neighbour && !is_dst_neighbour {
                        // Neighbour of neither endpoint: a typed 4-path-edge orbit
                        // (edge (src, dst) at the end of the path).
                        graphlet_counter.insert(
                            (
                                src_node_type,
                                dst_node_type,
                                self.get_node_label(second_order_neighbour),
                                self.get_node_label(root),
                            )
                                .encode_with_graphlet::<ExtendedGraphletType>(
                                    ExtendedGraphletType::FourPathEdge,
                                    self.get_number_of_node_labels(),
                                ),
                        );
                    } else if is_dst_neighbour
                        && !is_src_neighbour
                        && second_order_neighbour <= root
                    {
                        // Neighbour of solely the destination: the triangle
                        // {dst, root, second_order_neighbour} with tail (src, dst),
                        // a typed tailed-triangle tail-edge orbit (counted once via
                        // the `<= root` guard).
                        graphlet_counter.insert(
                            (
                                src_node_type,
                                dst_node_type,
                                self.get_node_label(second_order_neighbour),
                                self.get_node_label(root),
                            )
                                .encode_with_graphlet::<ExtendedGraphletType>(
                                    ExtendedGraphletType::TailedTriTail,
                                    self.get_number_of_node_labels(),
                                ),
                        );
                    } else if is_src_neighbour && !is_dst_neighbour {
                        // Neighbour of solely the source: the induced subgraph
                        // {src, dst, root, second_order_neighbour} is a 4-cycle
                        // (src - dst - root - second_order_neighbour - src). Each
                        // 4-cycle has a single destination-exclusive node (the
                        // root) so it is counted exactly once here.
                        graphlet_counter.insert(
                            (
                                src_node_type,
                                dst_node_type,
                                self.get_node_label(second_order_neighbour),
                                self.get_node_label(root),
                            )
                                .encode_with_graphlet::<ExtendedGraphletType>(
                                    ExtendedGraphletType::FourCycle,
                                    self.get_number_of_node_labels(),
                                ),
                        );
                    }
                }
            };

        // We start to iterate over the neighbours of the source and destination nodes.
        while let (Some(&src_neighbour), Some(&dst_neighbour)) = (src_iter.peek(), dst_iter.peek())
        {
            // We skip the neighbours if they are the same as the source or destination nodes.
            if src_neighbour == src || src_neighbour == dst {
                src_iter.next();
                continue;
            }

            if dst_neighbour == src || dst_neighbour == dst {
                dst_iter.next();
                continue;
            }

            match src_neighbour.cmp(&dst_neighbour) {
                // If the two neighbours are the same, we have identified a triangle.
                core::cmp::Ordering::Equal => {
                    // We get the node labels of the source only, as both have
                    // necessarily the same node label.
                    let node_neighbour_type = self.get_node_label(src_neighbour);

                    // We increase the counter of the node label of the triangle.
                    triangle_labels_counts[self.get_node_label_index(node_neighbour_type)] +=
                        Count::one();

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
                            .encode_with_graphlet::<ExtendedGraphletType>(
                                ExtendedGraphletType::Triangle,
                                self.get_number_of_node_labels(),
                            ),
                    );

                    // We iterate over the neighbours of the triangle node.
                    // These nodes will be second-order neighbours of the source and destination nodes.
                    // We classify every neighbour of the triangle node by its
                    // membership in the source and destination neighbourhoods,
                    // advancing monotonic cursors over the sorted neighbour lists.
                    let second_order_iterator = self.iter_neighbours(src_neighbour);
                    let mut src_second_order_iterator = self.iter_neighbours(src).peekable();
                    let mut dst_second_order_iterator = self.iter_neighbours(dst).peekable();

                    for second_order_neighbour in second_order_iterator {
                        if second_order_neighbour == src || second_order_neighbour == dst {
                            continue;
                        }

                        while src_second_order_iterator
                            .peek()
                            .is_some_and(|&x| x < second_order_neighbour)
                        {
                            src_second_order_iterator.next();
                        }
                        let is_src_neighbour =
                            src_second_order_iterator.peek() == Some(&second_order_neighbour);
                        while dst_second_order_iterator
                            .peek()
                            .is_some_and(|&x| x < second_order_neighbour)
                        {
                            dst_second_order_iterator.next();
                        }
                        let is_dst_neighbour =
                            dst_second_order_iterator.peek() == Some(&second_order_neighbour);

                        if is_src_neighbour && is_dst_neighbour {
                            // Common neighbour of both endpoints and of the triangle
                            // node: the four nodes form a 4-clique. The `<= src_neighbour`
                            // guard counts each clique exactly once.
                            if second_order_neighbour <= src_neighbour {
                                graphlet_counter.insert(
                                    (
                                        src_node_type,
                                        dst_node_type,
                                        node_neighbour_type,
                                        self.get_node_label(second_order_neighbour),
                                    )
                                        .encode_with_graphlet::<ExtendedGraphletType>(
                                            ExtendedGraphletType::FourClique,
                                            self.get_number_of_node_labels(),
                                        ),
                                );
                            }
                        } else if is_src_neighbour || is_dst_neighbour {
                            // Neighbour of the triangle node and exactly one endpoint:
                            // the induced subgraph is a diamond with edge (src, dst) as
                            // a rim edge, i.e. a typed chordal-cycle edge orbit.
                            graphlet_counter.insert(
                                (
                                    src_node_type,
                                    dst_node_type,
                                    node_neighbour_type,
                                    self.get_node_label(second_order_neighbour),
                                )
                                    .encode_with_graphlet::<ExtendedGraphletType>(
                                        ExtendedGraphletType::ChordalCycleEdge,
                                        self.get_number_of_node_labels(),
                                    ),
                            );
                        } else {
                            // Neighbour of the triangle node but of neither endpoint:
                            // the triangle {src, dst, triangle node} with a tail at the
                            // triangle node, i.e. a typed tailed-triangle center orbit.
                            graphlet_counter.insert(
                                (
                                    src_node_type,
                                    dst_node_type,
                                    node_neighbour_type,
                                    self.get_node_label(second_order_neighbour),
                                )
                                    .encode_with_graphlet::<ExtendedGraphletType>(
                                        ExtendedGraphletType::TailedTriCenter,
                                        self.get_number_of_node_labels(),
                                    ),
                            );
                        }
                    }
                    // We can now advance the two iterators of the source and destination nodes.
                    src_iter.next();
                    dst_iter.next();
                }
                // Otherwise, if the two neighbours are not the same, both
                // may compose a 3-path with the source and destination nodes.
                // Since we are iterating over sorted iterators, we can check
                // which one of the two nodes is the smallest. It may be the case that
                // the larger node will also appear in the other iterator, but because
                // of the sorted nature of the iterators we are sure that the smaller
                // will never appear in the other iterator.
                core::cmp::Ordering::Less => {
                    // If the source neighbour is smaller than the destination neighbour,
                    // it forms a 3-path with the source and destination nodes.
                    handle_src_rooted_typed_paths(
                        src_neighbour,
                        &mut graphlet_counter,
                        &mut src_neighbour_labels_counts,
                    );

                    // We update the iterator with the lesser of the two nodes, which
                    // in this case is the source iterator:
                    src_iter.next();
                }
                core::cmp::Ordering::Greater => {
                    // If the destination neighbour is smaller than the source neighbour,
                    // it forms a 3-path with the source and destination nodes.
                    handle_dst_rooted_typed_paths(
                        dst_neighbour,
                        &mut graphlet_counter,
                        &mut dst_neighbour_labels_counts,
                    );

                    // We update the iterator with the lesser of the two nodes, which
                    // in this case is the destination iterator:
                    dst_iter.next();
                }
            }
        }
        // Finally, we need to check whether both iterators are finished. If this is not the case,
        // the source or destination neighbours are surely not present in each other's iterator
        // and they form a 3-path with the source and destination nodes.
        for src_neighbour in src_iter {
            // We need to check that the source neighbour is not equal to the destination node.
            // If this is the case, we need to skip it.
            if src_neighbour == dst || src_neighbour == src {
                continue;
            }

            handle_src_rooted_typed_paths(
                src_neighbour,
                &mut graphlet_counter,
                &mut src_neighbour_labels_counts,
            );
        }

        for dst_neighbour in dst_iter {
            // We need to check that the destination neighbour is not equal to the source node.
            // If this is the case, we need to skip it.
            if dst_neighbour == src || dst_neighbour == dst {
                continue;
            }

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
            let number_of_triangles_with_row_label = triangle_labels_counts[rows_label];

            #[cfg(debug_assertions)]
            debug_assert_eq!(
                number_of_triangles_with_row_label,
                <usize as AsPrimitive<Count>>::as_(
                    DebugTypedGraph::from(self).get_intersection_size_of_label(
                        src,
                        dst,
                        self.get_node_label_from_usize(rows_label)
                    )
                ),
                concat!(
                    "The number of triangles with the label {:?} is not equal to the number ",
                    "of neighbours of the source and destination nodes with the same label. ",
                    "We expected {:?} but found {:?}. The count vector is {:?}."
                ),
                self.get_node_label(rows_label),
                number_of_triangles_with_row_label,
                DebugTypedGraph::from(self).get_intersection_size_of_label(
                    src,
                    dst,
                    self.get_node_label(rows_label)
                ),
                triangle_labels_counts
            );

            let number_of_src_neighbours_with_row_label = src_neighbour_labels_counts[rows_label];

            #[cfg(debug_assertions)]
            debug_assert_eq!(
                number_of_src_neighbours_with_row_label,
                <usize as AsPrimitive<Count>>::as_(DebugTypedGraph::from(self).get_subtraction_of_neighbours_of_label(src, dst, self.get_node_label_from_usize(rows_label))
                    .count()),
                concat!(
                    "The number of neighbours of the source node with the label {:?} is not equal to the number ",
                    "of neighbours of the source node with the same label. ",
                    "We expected {:?} but found {:?}. The count vector is {:?}. ",
                    "The neighbours of source of the current label are {:?} and the neighbours of destination of the current label are {:?}."
                ),
                self.get_node_label_from_usize(rows_label),
                number_of_src_neighbours_with_row_label,
                DebugTypedGraph::from(self).get_subtraction_of_neighbours_of_label(src, dst, self.get_node_label_from_usize(rows_label))
                    .count(),
                src_neighbour_labels_counts,
                DebugTypedGraph::from(self).iter_neighbours_of_label(src, self.get_node_label_from_usize(rows_label))
                    .collect::<Vec<_>>(),
                DebugTypedGraph::from(self).iter_neighbours_of_label(dst, self.get_node_label_from_usize(rows_label)).collect::<Vec<_>>()
            );

            let number_of_dst_neighbours_with_row_label = dst_neighbour_labels_counts[rows_label];

            #[cfg(debug_assertions)]
            debug_assert_eq!(
                number_of_dst_neighbours_with_row_label,
                <usize as AsPrimitive<Count>>::as_(DebugTypedGraph::from(self).get_subtraction_of_neighbours_of_label(dst, src, self.get_node_label_from_usize(rows_label))
                    .count()),
                concat!(
                    "The number of neighbours of the destination node with the label {:?} is not equal to the number ",
                    "of neighbours of the destination node with the same label. ",
                    "We expected {:?} but found {:?}. The count vector is {:?}. ",
                    "The neighbours of source of the current label are {:?} and the neighbours of destination of the current label are {:?}. ",
                    "The subtraction of the destination neighbours minus the source neighbours is {:?}."
                ),
                self.get_node_label_from_usize(rows_label),
                number_of_dst_neighbours_with_row_label,
                DebugTypedGraph::from(self).get_subtraction_of_neighbours_of_label(dst, src, self.get_node_label_from_usize(rows_label))
                    .count(),
                dst_neighbour_labels_counts,
                DebugTypedGraph::from(self).iter_neighbours_of_label(src, self.get_node_label_from_usize(rows_label))
                    .collect::<Vec<_>>(),
                DebugTypedGraph::from(self).iter_neighbours_of_label(dst, self.get_node_label_from_usize(rows_label)).collect::<Vec<_>>(),
                DebugTypedGraph::from(self).get_subtraction_of_neighbours_of_label(dst, src, self.get_node_label_from_usize(rows_label))
                    .collect::<Vec<_>>()
            );

            // Additionaly, it should hold that the number of triangles with the label
            // plus the number of neighbours EXCLUSIVELY of the source node with the label
            // should be equal to the number of neighbours of the source node with the label.
            #[cfg(debug_assertions)]
            debug_assert_eq!(
                number_of_triangles_with_row_label + number_of_src_neighbours_with_row_label,
                <usize as AsPrimitive<Count>>::as_(DebugTypedGraph::from(self).iter_neighbours_of_label(src, self.get_node_label_from_usize(rows_label))
                    .filter(|node| {*node != dst})
                    .count()),
                concat!(
                    "The number of triangles with the label {:?} plus the number of neighbours EXCLUSIVELY of the source node with the label {:?} ",
                    "is not equal to the number of neighbours of the source node with the label. ",
                    "The current edge is ({:?}, {:?}). ",
                    "We expected {:?} but found {:?}. The count vector is {:?}. ",
                    "The neighbours of source of the current label are {:?} and the neighbours of destination of the current label are {:?}."
                ),
                self.get_node_label(rows_label),
                self.get_node_label_from_usize(rows_label),
                src, dst,
                number_of_triangles_with_row_label + number_of_src_neighbours_with_row_label,
                DebugTypedGraph::from(self).iter_neighbours_of_label(src, self.get_node_label_from_usize(rows_label))
                .filter(|node| {*node != dst})
                    .count(),
                src_neighbour_labels_counts,
                DebugTypedGraph::from(self).iter_neighbours_of_label(src, self.get_node_label_from_usize(rows_label))
                .filter(|node| {*node != dst})
                    .collect::<Vec<_>>(),
                DebugTypedGraph::from(self).iter_neighbours_of_label(dst, self.get_node_label_from_usize(rows_label)).filter(|node| {*node != src}).collect::<Vec<_>>()
            );

            #[cfg(debug_assertions)]
            // We do the same check for the destination node.
            debug_assert_eq!(
                number_of_triangles_with_row_label + number_of_dst_neighbours_with_row_label,
                <usize as AsPrimitive<Count>>::as_(DebugTypedGraph::from(self).iter_neighbours_of_label(dst, self.get_node_label_from_usize(rows_label))
                    .filter(|node| {*node != src})
                    .count()),
                concat!(
                    "The number of triangles with the label {:?} plus the number of neighbours EXCLUSIVELY of the destination node with the label {:?} ",
                    "is not equal to the number of neighbours of the destination node with the label. ",
                    "The current edge is ({:?}, {:?}). ",
                    "We expected {:?} but found {:?}. The count vector is {:?}. ",
                    "The neighbours of source of the current label are {:?} and the neighbours of destination of the current label are {:?}."
                ),
                self.get_node_label(rows_label),
                self.get_node_label_from_usize(rows_label),
                src, dst,
                number_of_triangles_with_row_label + number_of_dst_neighbours_with_row_label,
                DebugTypedGraph::from(self).iter_neighbours_of_label(dst, self.get_node_label_from_usize(rows_label))
                    .filter(|node| {*node != src})
                    .count(),
                dst_neighbour_labels_counts,
                DebugTypedGraph::from(self).iter_neighbours_of_label(src, self.get_node_label_from_usize(rows_label))
                    .filter(|node| {*node != dst})
                    .collect::<Vec<_>>(),
                DebugTypedGraph::from(self).iter_neighbours_of_label(dst, self.get_node_label_from_usize(rows_label)).filter(|node| {*node != src}).collect::<Vec<_>>()
            );

            // We need to retrieve the number of graphlets for the combination of labels
            // (source node label, destination node label, rows label, columns label),
            // for the four cycles, tailed-tri-tail, chord-cycle-edge and four-clique orbits.
            let number_of_homogenously_typed_four_cycles: Count = graphlet_counter
                .get_number_of_graphlets(
                    (
                        src_node_type,
                        dst_node_type,
                        self.get_node_label_from_usize(rows_label),
                        self.get_node_label_from_usize(rows_label),
                    )
                        .encode_with_graphlet::<ExtendedGraphletType>(
                            ExtendedGraphletType::FourCycle,
                            self.get_number_of_node_labels(),
                        ),
                );
            let number_of_homogenously_typed_tailed_tri_tails: Count = graphlet_counter
                .get_number_of_graphlets(
                    (
                        src_node_type,
                        dst_node_type,
                        self.get_node_label_from_usize(rows_label),
                        self.get_node_label_from_usize(rows_label),
                    )
                        .encode_with_graphlet::<ExtendedGraphletType>(
                            ExtendedGraphletType::TailedTriTail,
                            self.get_number_of_node_labels(),
                        ),
                );
            let number_of_homogenously_typed_chordal_cycle_edges: Count = graphlet_counter
                .get_number_of_graphlets(
                    (
                        src_node_type,
                        dst_node_type,
                        self.get_node_label_from_usize(rows_label),
                        self.get_node_label_from_usize(rows_label),
                    )
                        .encode_with_graphlet::<ExtendedGraphletType>(
                            ExtendedGraphletType::ChordalCycleEdge,
                            self.get_number_of_node_labels(),
                        ),
                );

            // We can verify whether the value of chordal cycle edges is self-consistent
            // with the other computed values. Namely, if there is a non-zero number of
            // chordal cycle edges, then there should be a non-zero number of triangles
            // of the same rows_label and columns_label, and the number of neighbours
            // exclusively associated to the source and destination nodes should be non-zero
            debug_assert!(
                number_of_homogenously_typed_chordal_cycle_edges == Count::zero()
                    || (number_of_triangles_with_row_label > Count::zero()
                        && (number_of_src_neighbours_with_row_label > Count::zero()
                            || number_of_dst_neighbours_with_row_label > Count::zero())),
                concat!(
                    "The number of chordal cycle edges is non-zero, but the number of triangles ",
                    "or the number of neighbours of the source and destination nodes is zero. ",
                    "The current edge is ({:?}, {:?}). "
                ),
                src,
                dst
            );

            let number_of_homogenously_typed_four_cliques = graphlet_counter
                .get_number_of_graphlets(
                    (
                        src_node_type,
                        dst_node_type,
                        self.get_node_label_from_usize(rows_label),
                        self.get_node_label_from_usize(rows_label),
                    )
                        .encode_with_graphlet::<ExtendedGraphletType>(
                            ExtendedGraphletType::FourClique,
                            self.get_number_of_node_labels(),
                        ),
                );

            // Now we have all ingredients to compute the number of graphlets for the
            // graphlets (4), (5), (9) and (11), which are four-path center orbits,
            // four-star orbits, tailed tri-edge orbits and chordal cycle center orbits.

            // We start with the four-path center orbits.
            let number_homogeneously_of_four_path_center_orbits: Count =
                get_homogeneously_typed_four_path_orbit_count(
                    number_of_homogenously_typed_four_cycles,
                    number_of_src_neighbours_with_row_label,
                    number_of_dst_neighbours_with_row_label,
                );

            // We update the graphlet counter with the number of four-path center orbits.
            graphlet_counter.insert_count(
                (
                    src_node_type,
                    dst_node_type,
                    self.get_node_label_from_usize(rows_label),
                    self.get_node_label_from_usize(rows_label),
                )
                    .encode_with_graphlet::<ExtendedGraphletType>(
                        ExtendedGraphletType::FourPathCenter,
                        self.get_number_of_node_labels(),
                    ),
                number_homogeneously_of_four_path_center_orbits,
            );

            // We continue with the four-star orbits.
            let number_of_homogeneously_typed_four_star_orbits: Count =
                get_homogeneously_typed_four_star_orbit_count(
                    number_of_homogenously_typed_tailed_tri_tails,
                    number_of_src_neighbours_with_row_label,
                    number_of_dst_neighbours_with_row_label,
                );

            // We update the graphlet counter with the number of four-star orbits.
            graphlet_counter.insert_count(
                (
                    src_node_type,
                    dst_node_type,
                    self.get_node_label_from_usize(rows_label),
                    self.get_node_label_from_usize(rows_label),
                )
                    .encode_with_graphlet::<ExtendedGraphletType>(
                        ExtendedGraphletType::FourStar,
                        self.get_number_of_node_labels(),
                    ),
                number_of_homogeneously_typed_four_star_orbits,
            );

            // We continue with the tailed tri-edge orbits.
            let number_of_homogeneously_tailed_tri_edge_orbits: Count =
                get_homogeneously_typed_tailed_triangle_tri_edge_orbit_count(
                    number_of_homogenously_typed_chordal_cycle_edges,
                    number_of_triangles_with_row_label,
                    number_of_src_neighbours_with_row_label,
                    number_of_dst_neighbours_with_row_label,
                );

            // We update the graphlet counter with the number of tailed tri-edge orbits.
            graphlet_counter.insert_count(
                (
                    src_node_type,
                    dst_node_type,
                    self.get_node_label_from_usize(rows_label),
                    self.get_node_label_from_usize(rows_label),
                )
                    .encode_with_graphlet::<ExtendedGraphletType>(
                        ExtendedGraphletType::TailedTriEdge,
                        self.get_number_of_node_labels(),
                    ),
                number_of_homogeneously_tailed_tri_edge_orbits,
            );

            // We continue with the chordal cycle center orbits.
            let number_of_homogeneously_chordal_cycle_center_orbits =
                get_homogeneously_typed_chordal_cycle_center_orbit_count(
                    number_of_homogenously_typed_four_cliques,
                    number_of_triangles_with_row_label,
                );

            // We update the graphlet counter with the number of chordal cycle center orbits.
            graphlet_counter.insert_count(
                (
                    src_node_type,
                    dst_node_type,
                    self.get_node_label_from_usize(rows_label),
                    self.get_node_label_from_usize(rows_label),
                )
                    .encode_with_graphlet::<ExtendedGraphletType>(
                        ExtendedGraphletType::ChordalCycleCenter,
                        self.get_number_of_node_labels(),
                    ),
                number_of_homogeneously_chordal_cycle_center_orbits,
            );

            // We iterate on the upper triangular matrix of the triangle labels counts.
            for columns_label in (rows_label + 1)..self.get_number_of_node_labels_usize() {
                let number_of_triangles_with_column_label = triangle_labels_counts[columns_label];
                let number_of_src_neighbours_with_column_label: Count =
                    src_neighbour_labels_counts[columns_label];
                let number_of_dst_neighbours_with_column_label: Count =
                    dst_neighbour_labels_counts[columns_label];

                #[cfg(debug_assertions)]
                // We write three debug assert tests very similar to the ones
                // done for the row labels:
                debug_assert_eq!(
                    number_of_triangles_with_column_label,
                    <usize as AsPrimitive<Count>>::as_(
                        DebugTypedGraph::from(self).get_intersection_size_of_label(
                            src,
                            dst,
                            self.get_node_label_from_usize(columns_label)
                        )
                    ),
                    concat!(
                        "The number of triangles with the label {:?} is not equal to the number ",
                        "of neighbours of the source and destination nodes with the same label. ",
                        "We expected {:?} but found {:?}. The count vector is {:?}."
                    ),
                    self.get_node_label(columns_label),
                    number_of_triangles_with_column_label,
                    DebugTypedGraph::from(self).get_intersection_size_of_label(
                        src,
                        dst,
                        self.get_node_label_from_usize(columns_label)
                    ),
                    triangle_labels_counts
                );

                #[cfg(debug_assertions)]
                debug_assert_eq!(
                    number_of_src_neighbours_with_column_label,
                    <usize as AsPrimitive<Count>>::as_(DebugTypedGraph::from(self).get_subtraction_of_neighbours_of_label(
                        src,
                        dst,
                        self.get_node_label_from_usize(columns_label)
                    )
                    .count()),
                    concat!(
                        "The number of neighbours of the source node with the label {:?} is not equal to the number ",
                        "of neighbours of the source node with the same label. ",
                        "We expected {:?} but found {:?}. The count vector is {:?}. ",
                        "The neighbours of source of the current label are {:?} and the neighbours of destination of the current label are {:?}."
                    ),
                    self.get_node_label_from_usize(columns_label),
                    number_of_src_neighbours_with_column_label,
                    DebugTypedGraph::from(self).get_subtraction_of_neighbours_of_label(
                        src,
                        dst,
                        self.get_node_label_from_usize(columns_label)
                    )
                    .count(),
                    src_neighbour_labels_counts,
                    DebugTypedGraph::from(self).iter_neighbours_of_label(src, self.get_node_label_from_usize(columns_label))
                        .collect::<Vec<_>>(),
                    DebugTypedGraph::from(self).iter_neighbours_of_label(dst, self.get_node_label_from_usize(columns_label)).collect::<Vec<_>>()
                );

                #[cfg(debug_assertions)]
                debug_assert_eq!(
                    number_of_dst_neighbours_with_column_label,
                    <usize as AsPrimitive<Count>>::as_(DebugTypedGraph::from(self).get_subtraction_of_neighbours_of_label(
                        dst,
                        src,
                        self.get_node_label_from_usize(columns_label)
                    )
                    .count()),
                    concat!(
                        "The number of neighbours of the destination node with the label {:?} is not equal to the number ",
                        "of neighbours of the destination node with the same label. ",
                        "The edge currently being processed is ({:?}, {:?}). ",
                        "We expected {:?} but found {:?}. The count vector is {:?}. ",
                        "The neighbours of source of the current label are {:?} and the neighbours of destination of the current label are {:?}. ",
                        "The subtraction of the destination neighbours minus the source neighbours is {:?}."
                    ),
                    self.get_node_label_from_usize(columns_label),
                    src, dst,
                    number_of_dst_neighbours_with_column_label,
                    DebugTypedGraph::from(self).get_subtraction_of_neighbours_of_label(
                        dst,
                        src,
                        self.get_node_label_from_usize(columns_label)
                    )
                    .count(),
                    dst_neighbour_labels_counts,
                    DebugTypedGraph::from(self).iter_neighbours_of_label(src, self.get_node_label_from_usize(columns_label))
                        .collect::<Vec<_>>(),
                    DebugTypedGraph::from(self).iter_neighbours_of_label(dst, self.get_node_label_from_usize(columns_label)).collect::<Vec<_>>(),
                    DebugTypedGraph::from(self).get_subtraction_of_neighbours_of_label(
                        dst,
                        src,
                        self.get_node_label_from_usize(columns_label)
                    )
                    .collect::<Vec<_>>()
                );

                #[cfg(debug_assertions)]
                // As done for the row labels, we check that the number of triangles with the label
                // plus the number of neighbours EXCLUSIVELY of the source node with the label
                // should be equal to the number of neighbours of the source node with the label.
                debug_assert_eq!(
                    number_of_triangles_with_column_label + number_of_src_neighbours_with_column_label,
                    <usize as AsPrimitive<Count>>::as_(DebugTypedGraph::from(self).iter_neighbours_of_label(src, self.get_node_label_from_usize(columns_label))
                        .filter(|node| {*node != dst})
                        .count()),
                    concat!(
                        "The number of triangles with the label {:?} plus the number of neighbours EXCLUSIVELY of the source node with the label {:?} ",
                        "is not equal to the number of neighbours of the source node with the label. ",
                        "The current edge is ({:?}, {:?}). ",
                        "We expected {:?} but found {:?}. The count vector is {:?}. ",
                        "The neighbours of source of the current label are {:?} and the neighbours of destination of the current label are {:?}."
                    ),
                    self.get_node_label(columns_label),
                    self.get_node_label_from_usize(columns_label),
                    src, dst,
                    number_of_triangles_with_column_label + number_of_src_neighbours_with_column_label,
                    DebugTypedGraph::from(self).iter_neighbours_of_label(src, self.get_node_label_from_usize(columns_label))
                        .filter(|node| {*node != dst})
                        .count(),
                    src_neighbour_labels_counts,
                    DebugTypedGraph::from(self).iter_neighbours_of_label(src, self.get_node_label_from_usize(columns_label))
                        .filter(|node| {*node != dst})
                        .collect::<Vec<_>>(),
                    DebugTypedGraph::from(self).iter_neighbours_of_label(dst, self.get_node_label_from_usize(columns_label)).filter(|node| {*node != src}).collect::<Vec<_>>()
                );

                #[cfg(debug_assertions)]
                // We do the same check for the destination node.
                debug_assert_eq!(
                    number_of_triangles_with_column_label + number_of_dst_neighbours_with_column_label,
                    <usize as AsPrimitive<Count>>::as_(DebugTypedGraph::from(self).iter_neighbours_of_label(dst, self.get_node_label_from_usize(columns_label))
                        .filter(|node| {*node != src})
                        .count()),
                    concat!(
                        "The number of triangles with the label {:?} plus the number of neighbours EXCLUSIVELY of the destination node with the label {:?} ",
                        "is not equal to the number of neighbours of the destination node with the label. ",
                        "The current edge is ({:?}, {:?}). ",
                        "We expected {:?} but found {:?}. The count vector is {:?}. ",
                        "The neighbours of source of the current label are {:?} and the neighbours of destination of the current label are {:?}."
                    ),
                    self.get_node_label(columns_label),
                    self.get_node_label_from_usize(columns_label),
                    src, dst,
                    number_of_triangles_with_column_label + number_of_dst_neighbours_with_column_label,
                    DebugTypedGraph::from(self).iter_neighbours_of_label(dst, self.get_node_label_from_usize(columns_label))
                        .filter(|node| {*node != src})
                        .count(),
                    dst_neighbour_labels_counts,
                    DebugTypedGraph::from(self).iter_neighbours_of_label(src, self.get_node_label_from_usize(columns_label))
                        .filter(|node| {*node != dst})
                        .collect::<Vec<_>>(),
                    DebugTypedGraph::from(self).iter_neighbours_of_label(dst, self.get_node_label_from_usize(columns_label)).filter(|node| {*node != src}).collect::<Vec<_>>()
                );

                // We need to retrieve the number of graphlets for the combination of labels
                // (source node label, destination node label, rows label, columns label),
                // for the four cycles, tailed-tri-tail, chord-cycle-edge and four-clique orbits.
                let number_of_heterogenously_typed_four_cycles: Count = graphlet_counter
                    .get_number_of_graphlets(
                        (
                            src_node_type,
                            dst_node_type,
                            self.get_node_label_from_usize(rows_label),
                            self.get_node_label_from_usize(columns_label),
                        )
                            .encode_with_graphlet::<ExtendedGraphletType>(
                                ExtendedGraphletType::FourCycle,
                                self.get_number_of_node_labels(),
                            ),
                    );
                let number_of_heterogenously_typed_tailed_tri_tails: Count = graphlet_counter
                    .get_number_of_graphlets(
                        (
                            src_node_type,
                            dst_node_type,
                            self.get_node_label_from_usize(rows_label),
                            self.get_node_label_from_usize(columns_label),
                        )
                            .encode_with_graphlet::<ExtendedGraphletType>(
                                ExtendedGraphletType::TailedTriTail,
                                self.get_number_of_node_labels(),
                            ),
                    );
                let number_of_heterogenously_typed_chordal_cycle_edges: Count = graphlet_counter
                    .get_number_of_graphlets(
                        (
                            src_node_type,
                            dst_node_type,
                            self.get_node_label_from_usize(rows_label),
                            self.get_node_label_from_usize(columns_label),
                        )
                            .encode_with_graphlet::<ExtendedGraphletType>(
                                ExtendedGraphletType::ChordalCycleEdge,
                                self.get_number_of_node_labels(),
                            ),
                    );

                // We can verify whether the value of chordal cycle edges is self-consistent
                // with the other computed values. Namely, if there is a non-zero number of
                // chordal cycle edges, then there should be a non-zero number of triangles
                // of the same rows_label and columns_label, and the number of neighbours
                // exclusively associated to the source and destination nodes should be non-zero
                debug_assert!(
                    number_of_heterogenously_typed_chordal_cycle_edges == Count::zero() || rows_label != columns_label
                        || (number_of_triangles_with_row_label > Count::zero()
                            && number_of_triangles_with_column_label > Count::zero()
                            && (number_of_src_neighbours_with_row_label > Count::zero()
                            && number_of_src_neighbours_with_column_label > Count::zero()
                            || number_of_dst_neighbours_with_row_label > Count::zero()
                            && number_of_dst_neighbours_with_column_label > Count::zero())),
                    concat!(
                        "The number of chordal cycle edges is non-zero, but the number of triangles ",
                        "or the number of neighbours of the source and destination nodes is zero. ",
                        "The current edge is ({:?}, {:?}). ",
                        "The number of chordal cycle edges is {:?}, the number of triangles with the rows label {:?} is {:?}, ",
                        "the number of exclusive neighbours of the source node with the rows label {:?} is {:?}, ",
                        "the number of exclusive neighbours of the source node with the columns label {:?} is {:?}, ",
                        "the number of exclusive neighbours of the destination node with the rows label {:?} is {:?}, ",
                    ),
                    src, dst,
                    number_of_heterogenously_typed_chordal_cycle_edges,
                    self.get_node_label(rows_label),
                    number_of_triangles_with_row_label,
                    self.get_node_label(rows_label),
                    number_of_src_neighbours_with_row_label,
                    self.get_node_label(columns_label),
                    number_of_src_neighbours_with_column_label,
                    self.get_node_label(rows_label),
                    number_of_dst_neighbours_with_column_label
                );

                let number_of_heterogenously_typed_four_cliques = graphlet_counter
                    .get_number_of_graphlets(
                        (
                            src_node_type,
                            dst_node_type,
                            self.get_node_label_from_usize(rows_label),
                            self.get_node_label_from_usize(columns_label),
                        )
                            .encode_with_graphlet::<ExtendedGraphletType>(
                                ExtendedGraphletType::FourClique,
                                self.get_number_of_node_labels(),
                            ),
                    );

                // Now we have all ingredients to compute the number of graphlets for the
                // graphlets (4), (5), (9) and (11), which are four-path center orbits,
                // four-star orbits, tailed tri-edge orbits and chordal cycle center orbits.

                // We start with the four-path center orbits.
                let number_of_heterogenously_of_four_path_center_orbits: Count =
                    get_heterogeneously_typed_four_path_orbit_count(
                        number_of_heterogenously_typed_four_cycles,
                        number_of_src_neighbours_with_row_label,
                        number_of_dst_neighbours_with_row_label,
                        number_of_src_neighbours_with_column_label,
                        number_of_dst_neighbours_with_column_label,
                    );

                // We update the graphlet counter with the number of four-path center orbits.
                graphlet_counter.insert_count(
                    (
                        src_node_type,
                        dst_node_type,
                        self.get_node_label_from_usize(rows_label),
                        self.get_node_label_from_usize(columns_label),
                    )
                        .encode_with_graphlet::<ExtendedGraphletType>(
                            ExtendedGraphletType::FourPathCenter,
                            self.get_number_of_node_labels(),
                        ),
                    number_of_heterogenously_of_four_path_center_orbits,
                );

                // We continue with the four-star orbits.
                let number_of_heterogeneously_four_star_orbits: Count =
                    get_heterogeneously_typed_four_star_orbit_count(
                        number_of_heterogenously_typed_tailed_tri_tails,
                        number_of_src_neighbours_with_row_label,
                        number_of_dst_neighbours_with_row_label,
                        number_of_src_neighbours_with_column_label,
                        number_of_dst_neighbours_with_column_label,
                    );

                // We update the graphlet counter with the number of four-star orbits.
                graphlet_counter.insert_count(
                    (
                        src_node_type,
                        dst_node_type,
                        self.get_node_label_from_usize(rows_label),
                        self.get_node_label_from_usize(columns_label),
                    )
                        .encode_with_graphlet::<ExtendedGraphletType>(
                            ExtendedGraphletType::FourStar,
                            self.get_number_of_node_labels(),
                        ),
                    number_of_heterogeneously_four_star_orbits,
                );

                // We continue with the tailed tri-edge orbits.
                let number_of_heterogeneously_tailed_tri_edge_orbits: Count =
                    get_heterogeneously_typed_tailed_triangle_tri_edge_orbit_count(
                        number_of_heterogenously_typed_chordal_cycle_edges,
                        number_of_triangles_with_row_label,
                        number_of_triangles_with_column_label,
                        number_of_src_neighbours_with_row_label,
                        number_of_dst_neighbours_with_row_label,
                        number_of_src_neighbours_with_column_label,
                        number_of_dst_neighbours_with_column_label,
                    );

                // We update the graphlet counter with the number of tailed tri-edge orbits.
                graphlet_counter.insert_count(
                    (
                        src_node_type,
                        dst_node_type,
                        self.get_node_label_from_usize(rows_label),
                        self.get_node_label_from_usize(columns_label),
                    )
                        .encode_with_graphlet::<ExtendedGraphletType>(
                            ExtendedGraphletType::TailedTriEdge,
                            self.get_number_of_node_labels(),
                        ),
                    number_of_heterogeneously_tailed_tri_edge_orbits,
                );

                // We continue with the chordal cycle center orbits.
                let number_of_heterogeneously_typed_chordal_cycle_center_orbits =
                    get_heterogeneously_typed_chordal_cycle_center_orbit_count(
                        number_of_heterogenously_typed_four_cliques,
                        number_of_triangles_with_row_label,
                        number_of_triangles_with_column_label,
                    );

                // We update the graphlet counter with the number of chordal cycle center orbits.
                graphlet_counter.insert_count(
                    (
                        src_node_type,
                        dst_node_type,
                        self.get_node_label_from_usize(rows_label),
                        self.get_node_label_from_usize(columns_label),
                    )
                        .encode_with_graphlet::<ExtendedGraphletType>(
                            ExtendedGraphletType::ChordalCycleCenter,
                            self.get_number_of_node_labels(),
                        ),
                    number_of_heterogeneously_typed_chordal_cycle_center_orbits,
                );
            }
        }
        // We return the graphlet counter.
        graphlet_counter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hashbrown::HashMap;

    /// Minimal graph whose only purpose is to report a node-label count large
    /// enough to overflow a deliberately undersized `Graphlet` type.
    struct TinyGraph;

    impl Graph for TinyGraph {
        type NeighbourIter<'a> = core::iter::Empty<usize>;

        fn get_number_of_nodes(&self) -> usize {
            4
        }

        fn get_number_of_edges(&self) -> usize {
            0
        }

        fn iter_neighbours(&self, _node: usize) -> Self::NeighbourIter<'_> {
            core::iter::empty()
        }
    }

    impl TypedGraph for TinyGraph {
        type NodeLabel = u8;

        fn get_number_of_node_labels(&self) -> u8 {
            3
        }

        fn get_number_of_node_labels_usize(&self) -> usize {
            3
        }

        fn get_node_label_from_usize(&self, label_index: usize) -> u8 {
            label_index as u8
        }

        fn get_node_label_index(&self, label: u8) -> usize {
            label as usize
        }

        fn get_node_label(&self, _node: usize) -> u8 {
            0
        }
    }

    impl HeterogeneousGraphlets<u8, u32> for TinyGraph {
        type GraphLetCounter = HashMap<u8, u32>;
    }

    #[test]
    #[should_panic(expected = "cannot be encoded")]
    fn undersized_graphlet_type_panics() {
        // With 3 labels the maximal hash is 12 * 3^4 + 3^4 + 3^3 + 3^2 + 3 =
        // 1092, which does not fit in a u8 (max 255), so the encodability
        // assert must fire instead of silently producing wrong counts.
        let graph = TinyGraph;
        let _ = graph.get_heterogeneous_graphlet(0, 1);
    }
}
