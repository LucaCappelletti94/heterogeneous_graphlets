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
    graphlet_counter::GraphLetCounter,
    perfect_graphlet_hash::{canonical_descriptor, encode_edge_typed, PerfectGraphletHash},
    prelude::*,
};
use alloc::vec::Vec;
use num_traits::{AsPrimitive, Bounded, One, Zero};

/// A deliberately conservative upper bound on the largest hash any typed
/// graphlet can encode to for `number_of_node_labels` labels, computed in `u128`
/// so the intermediate arithmetic cannot itself overflow.
///
/// The perfect hash uses base `number_of_node_labels + 1` (one digit value is
/// reserved as the 3-node sentinel). This bound exceeds the true maximum by about
/// one whole `base^4` term, so its low-order terms sit well inside that slack:
/// the exact value is not behaviour-relevant, only that it is at least the true
/// maximum. (It is therefore excluded from mutation testing. The encodability
/// boundary itself is pinned by dedicated tests.)
fn maximal_possible_hash(number_of_node_labels: u128) -> u128 {
    let base = number_of_node_labels + 1;
    <ExtendedGraphletType as GraphletSet<u128>>::get_number_of_graphlets() * base.pow(4)
        + base.pow(4)
        + base.pow(3)
        + base.pow(2)
        + base
}

/// Whether the chosen graphlet key type, whose maximum representable value is
/// `maximal_graphlet`, is too small to hold `maximal_hash`, the largest hash any
/// graphlet could encode to for the graph's label count.
///
/// Flipping the comparison only changes behaviour when the two values are exactly
/// equal, which no key type and label count ever produce (the bound is never a
/// power of two minus one), so this is its own function to mark it as an
/// equivalent-mutant boundary (excluded from mutation testing). The encodability
/// boundary itself is pinned by dedicated tests.
#[inline]
fn graphlet_key_too_small(maximal_hash: u128, maximal_graphlet: u128) -> bool {
    maximal_hash > maximal_graphlet
}

/// A conservative upper bound on the largest edge-coloured graphlet hash for
/// `number_of_node_labels` node labels and `number_of_edge_labels` edge colours,
/// computed in `u128`. The key is `kind * node_base^4 * edge_base^6 + ...` with
/// `node_base = c + 1` and `edge_base = d + 1`, and `kind < number_of_graphlets`,
/// so every key is strictly below this product.
fn maximal_possible_edge_typed_hash(
    number_of_node_labels: u128,
    number_of_edge_labels: u128,
) -> u128 {
    let node_base = number_of_node_labels + 1;
    let edge_base = number_of_edge_labels + 1;
    <ExtendedGraphletType as GraphletSet<u128>>::get_number_of_graphlets()
        * node_base.pow(4)
        * edge_base.pow(6)
}

/// Whether the second-order neighbour is the canonical representative at which a
/// triangle- or clique-based orbit is counted, so each is counted exactly once.
///
/// Flipping the comparison counts the complementary representative of the same
/// unordered pair and yields identical totals, so this is its own function to
/// mark it as an equivalent-mutant boundary (excluded from mutation testing).
#[inline]
fn counted_at_canonical_root(second_order_neighbour: usize, root: usize) -> bool {
    second_order_neighbour <= root
}

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
    /// Counts the typed graphlet orbits incident to the edge `(src, dst)`.
    ///
    /// # Arguments
    /// * `src` - The source node of the edge.
    /// * `dst` - The destination node of the edge.
    ///
    /// # Errors
    /// Returns [`GraphletError::GraphletKeyTooSmall`] if the chosen `Graphlet`
    /// key integer type cannot hold every graphlet hash for this graph's
    /// node-label count. The check depends only on the label count and the
    /// chosen types, so it succeeds or fails identically for every edge. Pick a
    /// wider `Graphlet` type (the crate documentation lists the per-type
    /// capacities). Returning an error rather than panicking keeps the counter
    /// safe to call on untrusted graphs.
    fn get_heterogeneous_graphlet(
        &self,
        src: usize,
        dst: usize,
    ) -> Result<Self::GraphLetCounter, GraphletError> {
        // We verify that the chosen Graphlet integer type is wide enough to hold
        // every possible graphlet hash for this graph. The bound is computed in
        // u128 (rather than in the Graphlet type, whose own arithmetic could
        // overflow and defeat the check). A violation would otherwise silently
        // produce wrong counts through integer wraparound. The bound depends only
        // on the label count and the chosen types, so it is constant across edges.
        let number_of_node_labels: u128 = self.get_number_of_node_labels().as_();
        let maximal_hash_as_u128: u128 = maximal_possible_hash(number_of_node_labels);
        let maximal_graphlet_as_u128: u128 = Graphlet::max_value().as_();
        if graphlet_key_too_small(maximal_hash_as_u128, maximal_graphlet_as_u128) {
            return Err(GraphletError::GraphletKeyTooSmall {
                number_of_node_labels,
                maximal_hash: maximal_hash_as_u128,
                maximal_graphlet: maximal_graphlet_as_u128,
            });
        }

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

        // The two non-edge nodes of the four-cycle, tailed-triangle-tail,
        // chordal-cycle-edge and four-clique orbits occupy interchangeable
        // positions, so their labels form an unordered pair. The second pass
        // derives the four-path-center, four-star, tailed-tri-edge and
        // chordal-cycle-center orbits (equations 19, 23, 26 and 30) by reading a
        // single base-orbit count from the cell `(rows_label, columns_label)`
        // with `rows_label <= columns_label`, summing both label arrangements
        // into that one upper-triangular cell. We therefore store those base
        // orbits with the smaller-index label first, so each unordered pair lands
        // in the cell the derivation reads. Without this, a base orbit could be
        // stored in the lower triangle and be missed entirely by the lookup,
        // corrupting the derived heterogeneous counts.
        let canonical_pair = |first: Self::NodeLabel, second: Self::NodeLabel| {
            if self.get_node_label_index(first) <= self.get_node_label_index(second) {
                (first, second)
            } else {
                (second, first)
            }
        };

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
                        && counted_at_canonical_root(second_order_neighbour, root)
                    {
                        // The second order neighbour is a neighbour of solely the
                        // source node: it forms the triangle {src, root,
                        // second_order_neighbour} whose tail is edge (src, dst),
                        // i.e. a typed tailed-triangle tail-edge orbit. The
                        // `<= root` guard counts each such triangle once.
                        let (first_label, second_label) = canonical_pair(
                            self.get_node_label(second_order_neighbour),
                            self.get_node_label(root),
                        );
                        graphlet_counter.insert(
                            (src_node_type, dst_node_type, first_label, second_label)
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
                        && counted_at_canonical_root(second_order_neighbour, root)
                    {
                        // Neighbour of solely the destination: the triangle
                        // {dst, root, second_order_neighbour} with tail (src, dst),
                        // a typed tailed-triangle tail-edge orbit (counted once via
                        // the `<= root` guard).
                        let (first_label, second_label) = canonical_pair(
                            self.get_node_label(second_order_neighbour),
                            self.get_node_label(root),
                        );
                        graphlet_counter.insert(
                            (src_node_type, dst_node_type, first_label, second_label)
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
                        let (first_label, second_label) = canonical_pair(
                            self.get_node_label(second_order_neighbour),
                            self.get_node_label(root),
                        );
                        graphlet_counter.insert(
                            (src_node_type, dst_node_type, first_label, second_label)
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
                            if counted_at_canonical_root(second_order_neighbour, src_neighbour) {
                                let (first_label, second_label) = canonical_pair(
                                    node_neighbour_type,
                                    self.get_node_label(second_order_neighbour),
                                );
                                graphlet_counter.insert(
                                    (src_node_type, dst_node_type, first_label, second_label)
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
                            let (first_label, second_label) = canonical_pair(
                                node_neighbour_type,
                                self.get_node_label(second_order_neighbour),
                            );
                            graphlet_counter.insert(
                                (src_node_type, dst_node_type, first_label, second_label)
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

            let number_of_src_neighbours_with_row_label = src_neighbour_labels_counts[rows_label];

            let number_of_dst_neighbours_with_row_label = dst_neighbour_labels_counts[rows_label];

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
        Ok(graphlet_counter)
    }
}

/// Counting of typed 4-node graphlet orbits incident to each edge of an
/// [`EdgeTypedGraph`], distinguishing graphlets by BOTH their node colours and
/// their edge colours.
///
/// This is the edge-coloured counterpart of [`HeterogeneousGraphlets`]. It
/// enumerates all twelve orbits directly (the correctness-first baseline), keying
/// each occurrence by the perfect hash of its canonical positional descriptor
/// (node labels plus edge colours). Dropping the six edge digits from a key recovers
/// the node-only key, so edge-coloured counts collapse exactly to the node-typed
/// counts of [`HeterogeneousGraphlets`].
pub trait EdgeTypedGraphlets<Graphlet, Count>: EdgeTypedGraph
where
    Count: Debug + Copy + 'static + Ord + One + Zero + AddAssign,
    Self: Sized,
    Graphlet: Copy
        + Debug
        + 'static
        + Bounded
        + AsPrimitive<u128>
        + From<ExtendedGraphletType>
        + Mul<Output = Graphlet>
        + Add<Output = Graphlet>
        + One
        + Ord,
    Self::NodeLabel: Ord + Copy + 'static + AsPrimitive<Graphlet> + AsPrimitive<u128>,
    Self::EdgeLabel: Ord + Copy + 'static + AsPrimitive<Graphlet> + AsPrimitive<u128>,
    ExtendedGraphletType: GraphletSet<Graphlet>,
{
    /// The accumulator used to collect edge-coloured graphlet counts for an edge.
    type GraphLetCounter: GraphLetCounter<Graphlet, Count>;

    /// Counts the edge-coloured typed graphlet orbits incident to the edge
    /// `(src, dst)`, distinguishing graphlets by node and edge colours.
    ///
    /// # Arguments
    /// * `src` - One endpoint of the focal edge.
    /// * `dst` - The other endpoint of the focal edge.
    ///
    /// # Errors
    /// Returns [`GraphletError::EdgeGraphletKeyTooSmall`] if the chosen `Graphlet`
    /// key integer type cannot hold every edge-coloured graphlet hash for this
    /// graph's node-label and edge-colour counts. The check depends only on those
    /// counts and the chosen types, so it succeeds or fails identically for every
    /// edge.
    fn get_edge_typed_graphlet(
        &self,
        src: usize,
        dst: usize,
    ) -> Result<Self::GraphLetCounter, GraphletError> {
        let number_of_node_labels: u128 = self.get_number_of_node_labels().as_();
        let number_of_edge_labels: u128 = self.get_number_of_edge_labels().as_();
        let maximal_hash =
            maximal_possible_edge_typed_hash(number_of_node_labels, number_of_edge_labels);
        let maximal_graphlet: u128 = Graphlet::max_value().as_();
        if graphlet_key_too_small(maximal_hash, maximal_graphlet) {
            return Err(GraphletError::EdgeGraphletKeyTooSmall {
                number_of_node_labels,
                number_of_edge_labels,
                maximal_hash,
                maximal_graphlet,
            });
        }

        let i = src;
        let j = dst;
        let n = self.get_number_of_nodes();
        let adj = |a: usize, b: usize| self.iter_neighbours(a).any(|x| x == b);
        let lab = |x: usize| self.get_node_label(x);
        let node_sentinel = self.get_number_of_node_labels();
        let edge_sentinel = self.get_number_of_edge_labels();
        // Colour of edge (a, b) if present, else the absent-edge sentinel.
        let col = |a: usize, b: usize| {
            if adj(a, b) {
                self.get_edge_label(a, b)
            } else {
                edge_sentinel
            }
        };
        let node_count: Graphlet = self.get_number_of_node_labels().as_();
        let edge_count: Graphlet = self.get_number_of_edge_labels().as_();
        let node_base: Graphlet = node_count + Graphlet::one();
        let edge_base: Graphlet = edge_count + Graphlet::one();

        let mut counter = Self::GraphLetCounter::with_number_of_elements(node_base);

        let emit = |counter: &mut Self::GraphLetCounter,
                    kind: ExtendedGraphletType,
                    nodes: [Self::NodeLabel; 4],
                    edges: [Self::EdgeLabel; 6]| {
            let (canonical_nodes, canonical_edges) = canonical_descriptor(nodes, edges);
            let key = encode_edge_typed::<Graphlet, Self::NodeLabel, Self::EdgeLabel>(
                Graphlet::from(kind),
                canonical_nodes,
                canonical_edges,
                node_base,
                edge_base,
            );
            counter.insert_count(key, Count::one());
        };
        let emit3 = |counter: &mut Self::GraphLetCounter, kind: ExtendedGraphletType, w: usize| {
            let nodes = [lab(i), lab(j), lab(w), node_sentinel];
            let edges = [
                col(i, j),
                col(i, w),
                edge_sentinel,
                col(j, w),
                edge_sentinel,
                edge_sentinel,
            ];
            emit(counter, kind, nodes, edges);
        };
        let emit4 = |counter: &mut Self::GraphLetCounter,
                     kind: ExtendedGraphletType,
                     a: usize,
                     b: usize| {
            let nodes = [lab(i), lab(j), lab(a), lab(b)];
            let edges = [
                col(i, j),
                col(i, a),
                col(i, b),
                col(j, a),
                col(j, b),
                col(a, b),
            ];
            emit(counter, kind, nodes, edges);
        };

        let s_i: Vec<usize> = (0..n)
            .filter(|&w| w != i && w != j && adj(i, w) && !adj(j, w))
            .collect();
        let s_j: Vec<usize> = (0..n)
            .filter(|&w| w != i && w != j && adj(j, w) && !adj(i, w))
            .collect();
        let t: Vec<usize> = (0..n)
            .filter(|&w| w != i && w != j && adj(i, w) && adj(j, w))
            .collect();
        let far: Vec<usize> = (0..n)
            .filter(|&w| w != i && w != j && !adj(i, w) && !adj(j, w))
            .collect();
        let s_ij: Vec<usize> = s_i.iter().chain(&s_j).copied().collect();

        for &w in &s_i {
            emit3(&mut counter, ExtendedGraphletType::Triad, w);
        }
        for &w in &s_j {
            emit3(&mut counter, ExtendedGraphletType::Triad, w);
        }
        for &w in &t {
            emit3(&mut counter, ExtendedGraphletType::Triangle, w);
        }

        let product = |counter: &mut Self::GraphLetCounter,
                       kind: ExtendedGraphletType,
                       p: &[usize],
                       q: &[usize],
                       want_edge: bool| {
            for &a in p {
                for &b in q {
                    if adj(a, b) == want_edge {
                        emit4(counter, kind, a, b);
                    }
                }
            }
        };
        product(
            &mut counter,
            ExtendedGraphletType::FourPathEdge,
            &s_ij,
            &far,
            true,
        );
        product(
            &mut counter,
            ExtendedGraphletType::FourPathCenter,
            &s_i,
            &s_j,
            false,
        );
        product(
            &mut counter,
            ExtendedGraphletType::FourCycle,
            &s_i,
            &s_j,
            true,
        );
        product(
            &mut counter,
            ExtendedGraphletType::TailedTriCenter,
            &t,
            &far,
            true,
        );
        product(
            &mut counter,
            ExtendedGraphletType::TailedTriEdge,
            &t,
            &s_ij,
            false,
        );
        product(
            &mut counter,
            ExtendedGraphletType::ChordalCycleEdge,
            &t,
            &s_ij,
            true,
        );

        let within = |counter: &mut Self::GraphLetCounter,
                      kind: ExtendedGraphletType,
                      s: &[usize],
                      want_edge: bool| {
            for a in 0..s.len() {
                for b in (a + 1)..s.len() {
                    if adj(s[a], s[b]) == want_edge {
                        emit4(counter, kind, s[a], s[b]);
                    }
                }
            }
        };
        within(&mut counter, ExtendedGraphletType::FourStar, &s_i, false);
        within(&mut counter, ExtendedGraphletType::FourStar, &s_j, false);
        within(
            &mut counter,
            ExtendedGraphletType::TailedTriTail,
            &s_i,
            true,
        );
        within(
            &mut counter,
            ExtendedGraphletType::TailedTriTail,
            &s_j,
            true,
        );
        within(
            &mut counter,
            ExtendedGraphletType::ChordalCycleCenter,
            &t,
            false,
        );
        within(&mut counter, ExtendedGraphletType::FourClique, &t, true);

        Ok(counter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hashbrown::HashMap;

    /// Minimal graph whose only purpose is to report a chosen node-label count,
    /// to probe the encodability assertion at the boundary of a `u8` graphlet key.
    struct TinyGraph {
        num_labels: u8,
    }

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
            self.num_labels
        }

        fn get_number_of_node_labels_usize(&self) -> usize {
            self.num_labels as usize
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
    fn largest_label_count_fitting_u8_is_accepted() {
        // With 1 label the hash base is 2, so the maximal hash is
        // 12 * 2^4 + 2^4 + 2^3 + 2^2 + 2 = 222, which fits a u8 (max 255). This
        // is the largest label count that fits, so the call must return Ok. A
        // mutation that grows the bound past 255 here would wrongly return an
        // error and fail this test.
        let graph = TinyGraph { num_labels: 1 };
        assert!(graph.get_heterogeneous_graphlet(0, 1).is_ok());
    }

    #[test]
    fn undersized_graphlet_type_is_rejected() {
        // One label more (2, base 3) overflows a u8: 12 * 3^4 + 3^4 + 3^3 + 3^2 +
        // 3 = 1092 > 255, so the encodability check must return an error instead
        // of silently miscounting. Sitting one past the boundary, a mutation that
        // shrinks the bound at or below 255 here would wrongly let this pass.
        let graph = TinyGraph { num_labels: 2 };
        assert!(matches!(
            graph.get_heterogeneous_graphlet(0, 1),
            Err(GraphletError::GraphletKeyTooSmall { .. })
        ));
    }
}
