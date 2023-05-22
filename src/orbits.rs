use crate::prelude::*;

pub trait Orbit: TypedGraph {
    #[inline(always)]
    /// Returns the number of 4-paths orbit associated to the provided edge.
    ///
    /// # References
    /// The formula reported in this code is taken from the "Heterogeneous Graphlets" paper
    /// and specifically from the equation 19.
    fn get_typed_4_path_orbit_count(
        &self,
        typed_four_cycle_count: usize,
        src_node_type: Self::NodeLabel,
        dst_node_type: Self::NodeLabel,
        number_of_src_neighbours_with_src_type: usize,
        number_of_dst_neighbours_with_src_type: usize,
        number_of_src_neighbours_with_dst_type: usize,
        number_of_dst_neighbours_with_dst_type: usize,
    ) -> usize {
        if src_node_type == dst_node_type {
            number_of_src_neighbours_with_src_type * number_of_dst_neighbours_with_src_type
                - typed_four_cycle_count
        } else {
            number_of_src_neighbours_with_src_type * number_of_dst_neighbours_with_dst_type
                + number_of_src_neighbours_with_dst_type * number_of_dst_neighbours_with_src_type
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
    fn get_typed_4_star_orbit_count(
        &self,
        typed_tailed_triangle_tail_edge_count: usize,
        src_node_type: Self::NodeLabel,
        dst_node_type: Self::NodeLabel,
        number_of_src_neighbours_with_src_type: usize,
        number_of_dst_neighbours_with_src_type: usize,
        number_of_src_neighbours_with_dst_type: usize,
        number_of_dst_neighbours_with_dst_type: usize,
    ) -> usize {
        if src_node_type == dst_node_type {
            binomial_two(number_of_src_neighbours_with_src_type)
                + binomial_two(number_of_dst_neighbours_with_src_type)
                - typed_tailed_triangle_tail_edge_count
        } else {
            number_of_src_neighbours_with_src_type * number_of_src_neighbours_with_dst_type
                + number_of_dst_neighbours_with_dst_type * number_of_dst_neighbours_with_src_type
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
        src_node_type: Self::NodeLabel,
        dst_node_type: Self::NodeLabel,
        number_of_triangle_forming_neighbours_with_src_type: usize,
        number_of_triangle_forming_neighbours_with_dst_type: usize,
        number_of_src_neighbours_with_src_type: usize,
        number_of_dst_neighbours_with_src_type: usize,
        number_of_src_neighbours_with_dst_type: usize,
        number_of_dst_neighbours_with_dst_type: usize,
    ) -> usize {
        if src_node_type == dst_node_type {
            number_of_triangle_forming_neighbours_with_src_type
                * (number_of_src_neighbours_with_src_type + number_of_dst_neighbours_with_src_type)
                - typed_chordal_cycle_edge_count
        } else {
            number_of_triangle_forming_neighbours_with_src_type
                * (number_of_src_neighbours_with_dst_type + number_of_dst_neighbours_with_dst_type)
                + number_of_triangle_forming_neighbours_with_dst_type
                    * (number_of_src_neighbours_with_src_type
                        + number_of_dst_neighbours_with_src_type)
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
        typed_chordal_cycle_edge_count: usize,
        src_node_type: Self::NodeLabel,
        dst_node_type: Self::NodeLabel,
        number_of_triangle_forming_neighbours_with_src_type: usize,
        number_of_triangle_forming_neighbours_with_dst_type: usize,
    ) -> usize {
        if src_node_type == dst_node_type {
            binomial_two(number_of_triangle_forming_neighbours_with_src_type)
                - typed_chordal_cycle_edge_count
        } else {
            number_of_triangle_forming_neighbours_with_src_type
                * number_of_triangle_forming_neighbours_with_dst_type
                - typed_chordal_cycle_edge_count
        }
    }
}
