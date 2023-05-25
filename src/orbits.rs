use crate::prelude::*;
use crate::utils::binomial_two;

pub trait Orbit: TypedGraph {
    #[inline(always)]
    /// Returns the number of 4-paths orbit associated to the provided edge.
    ///
    /// # Arguments
    /// * `four_cycle_count` - The number of 4-cycles associated to the currently considered edge.
    /// * `first_node_type` - The type of the first node of the currently considered edge.
    /// * `second_node_type` - The type of the second node of the currently considered edge.
    /// * `number_of_src_neighbours_with_first_type` - The number of neighbours of the first node
    /// of the currently considered edge with the same type of the first node.
    /// * `number_of_dst_neighbours_with_first_type` - The number of neighbours of the second node
    /// of the currently considered edge with the same type of the first node.
    /// * `number_of_src_neighbours_with_second_type` - The number of neighbours of the first node
    /// of the currently considered edge with the same type of the second node.
    /// * `number_of_dst_neighbours_with_second_type` - The number of neighbours of the second node
    /// of the currently considered edge with the same type of the second node.
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
    /// # Arguments
    /// * `typed_tailed_triangle_tail_edge_count` - The number of typed tailed triangle tail edges associated to the currently considered edge.
    /// * `first_node_type` - The type of the first node of the currently considered edge.
    /// * `second_node_type` - The type of the second node of the currently considered edge.
    /// * `number_of_src_neighbours_with_first_type` - The number of neighbours of the first node
    /// of the currently considered edge with the same type of the first node.
    /// * `number_of_dst_neighbours_with_first_type` - The number of neighbours of the second node
    /// of the currently considered edge with the same type of the first node.
    /// * `number_of_src_neighbours_with_second_type` - The number of neighbours of the first node
    /// of the currently considered edge with the same type of the second node.
    /// * `number_of_dst_neighbours_with_second_type` - The number of neighbours of the second node
    /// of the currently considered edge with the same type of the second node.
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
            binomial_two(number_of_src_neighbours_with_first_type)
                + binomial_two(number_of_dst_neighbours_with_first_type)
                - typed_tailed_triangle_tail_edge_count
        } else {
            number_of_src_neighbours_with_first_type * number_of_src_neighbours_with_second_type
                + number_of_dst_neighbours_with_second_type
                    * number_of_dst_neighbours_with_first_type
                - typed_tailed_triangle_tail_edge_count
        }
    }

    #[inline(always)]
    /// Returns the number of typed tailed triangle tri-edge orbit associated to the provided edge.
    ///
    /// # Arguments
    /// * `typed_chordal_cycle_edge_count`: the number of typed chordal cycle edges for the current edge and the provided node types.
    /// * `first_node_type`: the type of the first node, this is usually the row matrix node type.
    /// * `second_node_type`: the type of the second node, this is usually the column matrix node type.
    /// * `number_of_triangle_forming_neighbours_with_first_type`: the number of neighbours with the first type that form a triangle with the current edge.
    /// * `number_of_triangle_forming_neighbours_with_second_type`: the number of neighbours with the second type that form a triangle with the current edge.
    /// * `number_of_src_neighbours_with_first_type`: the number of neighbours with the first type that are connected to the source node.
    /// * `number_of_dst_neighbours_with_first_type`: the number of neighbours with the first type that are connected to the destination node.
    /// * `number_of_src_neighbours_with_second_type`: the number of neighbours with the second type that are connected to the source node.
    /// * `number_of_dst_neighbours_with_second_type`: the number of neighbours with the second type that are connected to the destination node.
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
    /// # Arguments
    /// * `typed_chordal_cycle_center_count`: the number of typed chordal cycle center for the current edge and the provided node types.
    /// * `first_node_type`: the type of the first node, this is usually the row matrix node type.
    /// * `second_node_type`: the type of the second node, this is usually the column matrix node type.
    /// * `number_of_triangle_forming_neighbours_with_first_type`: the number of neighbours with the first type that form a triangle with the current edge.
    /// * `number_of_triangle_forming_neighbours_with_second_type`: the number of neighbours with the second type that form a triangle with the current edge.
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
