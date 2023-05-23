use crate::prelude::*;
use crate::utils::binomial_two;

pub trait Orbit: TypedGraph {
    #[inline(always)]
    /// Returns the number of 4-paths orbit associated to the provided edge.
    ///
    /// # References
    /// The formula reported in this code is taken from the "Heterogeneous Graphlets" paper
    /// and specifically from the equation 19.
    fn get_typed_four_path_orbit_count(
        &self,
        typed_four_cycle_count: usize,
        first_node_type: Self::NodeLabel,
        second_node_type: Self::NodeLabel,
        number_of_src_neighbours_with_first_type: usize,
        number_of_dst_neighbours_with_first_type: usize,
        number_of_src_neighbours_with_second_type: usize,
        number_of_dst_neighbours_with_second_type: usize,
    ) -> usize {
        if first_node_type == second_node_type {
            number_of_src_neighbours_with_first_type * number_of_dst_neighbours_with_first_type
                - typed_four_cycle_count
        } else {
            number_of_src_neighbours_with_first_type * number_of_dst_neighbours_with_second_type
                + number_of_src_neighbours_with_second_type
                    * number_of_dst_neighbours_with_first_type
                - typed_four_cycle_count
        }
    }

    #[inline(always)]
    /// Returns the number of typed 4-star orbit associated to the provided edge.
    ///
    /// # References
    /// The formula reported in this code is taken from the "Heterogeneous Graphlets" paper
    /// and specifically from the equation 23.
    ///
    fn get_typed_four_star_orbit_count(
        &self,
        typed_tailed_triangle_tail_edge_count: usize,
        first_node_type: Self::NodeLabel,
        second_node_type: Self::NodeLabel,
        number_of_src_neighbours_with_first_type: usize,
        number_of_dst_neighbours_with_first_type: usize,
        number_of_src_neighbours_with_second_type: usize,
        number_of_dst_neighbours_with_second_type: usize,
    ) -> usize {
        if first_node_type == second_node_type {
            debug_assert!(
                binomial_two(number_of_src_neighbours_with_first_type)
                + binomial_two(number_of_dst_neighbours_with_first_type) >= typed_tailed_triangle_tail_edge_count,
                concat!(
                    "The number of typed tailed triangle tail edges is greater than the number of possible edges. ",
                    "Specifically, the number of typed tailed triangle tail edges is {} while the number of possible edges is {}. ",
                    "The number of edges reported has been computed as the sum of the binomial coefficients of the number of neighbours with the first type, ",
                    "which is {} and {}."
                ),
                typed_tailed_triangle_tail_edge_count,
                binomial_two(number_of_src_neighbours_with_first_type) + binomial_two(number_of_dst_neighbours_with_first_type),
                number_of_src_neighbours_with_first_type, number_of_dst_neighbours_with_first_type
            );

            binomial_two(number_of_src_neighbours_with_first_type)
                + binomial_two(number_of_dst_neighbours_with_first_type)
                - typed_tailed_triangle_tail_edge_count
        } else {
            debug_assert!(
                number_of_src_neighbours_with_first_type * number_of_src_neighbours_with_second_type
                + number_of_dst_neighbours_with_first_type * number_of_dst_neighbours_with_second_type >= typed_tailed_triangle_tail_edge_count,
                concat!(
                    "The number of typed tailed triangle tail edges is greater than the number of possible edges. ",
                    "Specifically, the number of typed tailed triangle tail edges is {} while the number of possible edges is {}. ",
                    "The number of edges reported has been computed as the sum of the products of the number of neighbours with the first type, ",
                    "which is {} and {}, and the number of neighbours with the second type, which is {} and {}."
                ),
                typed_tailed_triangle_tail_edge_count,
                number_of_src_neighbours_with_first_type * number_of_src_neighbours_with_second_type
                    + number_of_dst_neighbours_with_first_type * number_of_dst_neighbours_with_second_type,
                number_of_src_neighbours_with_first_type, number_of_src_neighbours_with_second_type,
                number_of_dst_neighbours_with_first_type, number_of_dst_neighbours_with_second_type
            );

            number_of_src_neighbours_with_first_type * number_of_src_neighbours_with_second_type
                + number_of_dst_neighbours_with_second_type
                    * number_of_dst_neighbours_with_first_type
                - typed_tailed_triangle_tail_edge_count
        }
    }

    #[inline(always)]
    /// Returns the number of typed tailed triangle tri-edge orbit associated to the provided edge.
    ///
    /// # References
    /// The formula reported in this code is taken from the "Heterogeneous Graphlets" paper
    /// and specifically from the equation 26.
    ///
    fn get_typed_tailed_triangle_tri_edge_orbit_count(
        &self,
        typed_chordal_cycle_edge_count: usize,
        first_node_type: Self::NodeLabel,
        second_node_type: Self::NodeLabel,
        number_of_triangle_forming_neighbours_with_first_type: usize,
        number_of_triangle_forming_neighbours_with_second_type: usize,
        number_of_src_neighbours_with_first_type: usize,
        number_of_dst_neighbours_with_first_type: usize,
        number_of_src_neighbours_with_second_type: usize,
        number_of_dst_neighbours_with_second_type: usize,
    ) -> usize {
        if first_node_type == second_node_type {
            number_of_triangle_forming_neighbours_with_first_type
                * (number_of_src_neighbours_with_first_type
                    + number_of_dst_neighbours_with_first_type)
                - typed_chordal_cycle_edge_count
        } else {
            number_of_triangle_forming_neighbours_with_first_type
                * (number_of_src_neighbours_with_second_type
                    + number_of_dst_neighbours_with_second_type)
                + number_of_triangle_forming_neighbours_with_second_type
                    * (number_of_src_neighbours_with_first_type
                        + number_of_dst_neighbours_with_first_type)
                - typed_chordal_cycle_edge_count
        }
    }

    #[inline(always)]
    /// Returns the number of typed chordal-cycle center orbit associated to the provided edge.
    ///
    /// # References
    /// The formula reported in this code is taken from the "Heterogeneous Graphlets" paper
    /// and specifically from the equation 30.
    ///
    fn get_typed_chordal_cycle_center_orbit_count(
        &self,
        typed_chordal_cycle_center_count: usize,
        first_node_type: Self::NodeLabel,
        second_node_type: Self::NodeLabel,
        number_of_triangle_forming_neighbours_with_first_type: usize,
        number_of_triangle_forming_neighbours_with_second_type: usize,
    ) -> usize {
        if first_node_type == second_node_type {
            binomial_two(number_of_triangle_forming_neighbours_with_first_type)
                - typed_chordal_cycle_center_count
        } else {
            number_of_triangle_forming_neighbours_with_first_type
                * number_of_triangle_forming_neighbours_with_second_type
                - typed_chordal_cycle_center_count
        }
    }
}

impl<G> Orbit for G where G: TypedGraph {}
