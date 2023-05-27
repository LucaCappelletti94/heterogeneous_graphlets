use crate::numbers::*;
use std::ops::{Add, Div, Mul, Sub};

#[inline(always)]
/// Returns the binomial of the provided number of base two.
///
/// # Arguments
/// * `x` - The number whose binomial with two should be computed.
fn binomial_two<
    C: Zero + One + Two + Ord + Mul<C, Output = C> + Sub<C, Output = C> + Div<C, Output = C> + Copy,
>(
    x: C,
) -> C {
    if x < C::TWO {
        C::ZERO
    } else {
        x * (x - C::ONE) / C::TWO
    }
}

#[inline(always)]
/// Returns the number of 4-paths orbit associated to the provided edge.
///
/// # Arguments
/// * `four_cycle_count` - The number of 4-cycles associated to the currently considered edge.
/// * `number_of_src_neighbours` - The number of neighbours of the first node
/// of the currently considered edge with the same type of the first node.
/// * `number_of_dst_neighbours` - The number of neighbours of the second node
/// of the currently considered edge with the same type of the first node.
///
/// # References
/// The formula reported in this code is taken from the "Heterogeneous Graphlets" paper
/// and specifically from the equation 19, homogeneous case.
///
pub(crate) fn get_homogeneously_typed_four_path_orbit_count<
    C: Mul<C, Output = C> + Add<C, Output = C> + Sub<C, Output = C>,
>(
    typed_four_cycle_count: C,
    number_of_src_neighbours: C,
    number_of_dst_neighbours: C,
) -> C {
    number_of_src_neighbours * number_of_dst_neighbours - typed_four_cycle_count
}

#[inline(always)]
/// Returns the number of 4-paths orbit associated to the provided edge.
///
/// # Arguments
/// * `four_cycle_count` - The number of 4-cycles associated to the currently considered edge.
/// * `number_of_src_neighbours_with_row_label` - The number of neighbours of the first node
/// of the currently considered edge with the same type of the first node.
/// * `number_of_dst_neighbours_with_row_label` - The number of neighbours of the second node
/// of the currently considered edge with the same type of the first node.
/// * `number_of_src_neighbours_with_column_label` - The number of neighbours of the first node
/// of the currently considered edge with the same type of the second node.
/// * `number_of_dst_neighbours_with_column_label` - The number of neighbours of the second node
/// of the currently considered edge with the same type of the second node.
///
/// # References
/// The formula reported in this code is taken from the "Heterogeneous Graphlets" paper
/// and specifically from the equation 19.
///
pub(crate) fn get_heterogeneously_typed_four_path_orbit_count<
    C: Mul<C, Output = C> + Add<C, Output = C> + Sub<C, Output = C>,
>(
    typed_four_cycle_count: C,
    number_of_src_neighbours_with_row_label: C,
    number_of_dst_neighbours_with_row_label: C,
    number_of_src_neighbours_with_column_label: C,
    number_of_dst_neighbours_with_column_label: C,
) -> C {
    number_of_src_neighbours_with_row_label * number_of_dst_neighbours_with_column_label
        + number_of_src_neighbours_with_column_label * number_of_dst_neighbours_with_row_label
        - typed_four_cycle_count
}

#[inline(always)]
/// Returns the number of typed 4-star orbit associated to the provided edge.
///
/// # Arguments
/// * `typed_tailed_triangle_tail_edge_count` - The number of typed tailed triangle tail edges associated to the currently considered edge.
/// * `number_of_src_neighbours` - The number of neighbours of the first node
/// of the currently considered edge with the same type of the first node.
/// * `number_of_dst_neighbours` - The number of neighbours of the second node
/// of the currently considered edge with the same type of the first node.
///
/// # References
/// The formula reported in this code is taken from the "Heterogeneous Graphlets" paper
/// and specifically from the equation 23.
///
pub(crate) fn get_homogeneously_typed_four_star_orbit_count<
    C: Mul<C, Output = C>
        + Add<C, Output = C>
        + Sub<C, Output = C>
        + Div<C, Output = C>
        + Ord
        + Zero
        + One
        + Two
        + Copy,
>(
    typed_tailed_triangle_tail_edge_count: C,
    number_of_src_neighbours: C,
    number_of_dst_neighbours: C,
) -> C {
    binomial_two(number_of_src_neighbours) + binomial_two(number_of_dst_neighbours)
        - typed_tailed_triangle_tail_edge_count
}

#[inline(always)]
/// Returns the number of typed 4-star orbit associated to the provided edge.
///
/// # Arguments
/// * `typed_tailed_triangle_tail_edge_count` - The number of typed tailed triangle tail edges associated to the currently considered edge.
/// * `number_of_src_neighbours_with_row_label` - The number of neighbours of the first node
/// of the currently considered edge with the same type of the first node.
/// * `number_of_dst_neighbours_with_row_label` - The number of neighbours of the second node
/// of the currently considered edge with the same type of the first node.
/// * `number_of_src_neighbours_with_column_label` - The number of neighbours of the first node
/// of the currently considered edge with the same type of the second node.
/// * `number_of_dst_neighbours_with_column_label` - The number of neighbours of the second node
/// of the currently considered edge with the same type of the second node.
///
/// # References
/// The formula reported in this code is taken from the "Heterogeneous Graphlets" paper
/// and specifically from the equation 23.
///
pub(crate) fn get_heterogeneously_typed_four_star_orbit_count<
    C: Mul<C, Output = C>
        + Add<C, Output = C>
        + Sub<C, Output = C>
        + Div<C, Output = C>
        + Ord
        + Zero
        + One
        + Two
        + Copy,
>(
    typed_tailed_triangle_tail_edge_count: C,
    number_of_src_neighbours_with_row_label: C,
    number_of_dst_neighbours_with_row_label: C,
    number_of_src_neighbours_with_column_label: C,
    number_of_dst_neighbours_with_column_label: C,
) -> C {
    number_of_src_neighbours_with_row_label * number_of_src_neighbours_with_column_label
        + number_of_dst_neighbours_with_column_label * number_of_dst_neighbours_with_row_label
        - typed_tailed_triangle_tail_edge_count
}

#[inline(always)]
/// Returns the number of typed tailed triangle tri-edge orbit associated to the provided edge.
///
/// # Arguments
/// * `typed_chordal_cycle_edge_count`: the number of typed chordal cycle edges for the current edge and the provided node types.
/// * `number_of_triangle_forming_neighbours`: the number of neighbours with the first type that form a triangle with the current edge.
/// * `number_of_src_neighbours`: the number of neighbours with the first type that are connected to the source node.
/// * `number_of_dst_neighbours`: the number of neighbours with the first type that are connected to the destination node.
///
/// # References
/// The formula reported in this code is taken from the "Heterogeneous Graphlets" paper
/// and specifically from the equation 26.
///
pub(crate) fn get_homogeneously_typed_tailed_triangle_tri_edge_orbit_count<
    C: Mul<C, Output = C> + Add<C, Output = C> + Sub<C, Output = C>,
>(
    typed_chordal_cycle_edge_count: C,
    number_of_triangle_forming_neighbours: C,
    number_of_src_neighbours: C,
    number_of_dst_neighbours: C,
) -> C {
    number_of_triangle_forming_neighbours * (number_of_src_neighbours + number_of_dst_neighbours)
        - typed_chordal_cycle_edge_count
}

#[inline(always)]
/// Returns the number of typed tailed triangle tri-edge orbit associated to the provided edge.
///
/// # Arguments
/// * `typed_chordal_cycle_edge_count`: the number of typed chordal cycle edges for the current edge and the provided node types.
/// * `number_of_triangle_forming_neighbours_with_row_label`: the number of neighbours with the first type that form a triangle with the current edge.
/// * `number_of_triangle_forming_neighbours_with_column_label`: the number of neighbours with the second type that form a triangle with the current edge.
/// * `number_of_src_neighbours_with_row_label`: the number of neighbours with the first type that are connected to the source node.
/// * `number_of_dst_neighbours_with_row_label`: the number of neighbours with the first type that are connected to the destination node.
/// * `number_of_src_neighbours_with_column_label`: the number of neighbours with the second type that are connected to the source node.
/// * `number_of_dst_neighbours_with_column_label`: the number of neighbours with the second type that are connected to the destination node.
///
/// # References
/// The formula reported in this code is taken from the "Heterogeneous Graphlets" paper
/// and specifically from the equation 26.
///
pub(crate) fn get_heterogeneously_typed_tailed_triangle_tri_edge_orbit_count<
    C: Mul<C, Output = C> + Add<C, Output = C> + Sub<C, Output = C>,
>(
    typed_chordal_cycle_edge_count: C,
    number_of_triangle_forming_neighbours_with_row_label: C,
    number_of_triangle_forming_neighbours_with_column_label: C,
    number_of_src_neighbours_with_row_label: C,
    number_of_dst_neighbours_with_row_label: C,
    number_of_src_neighbours_with_column_label: C,
    number_of_dst_neighbours_with_column_label: C,
) -> C {
    number_of_triangle_forming_neighbours_with_row_label
        * (number_of_src_neighbours_with_column_label + number_of_dst_neighbours_with_column_label)
        + number_of_triangle_forming_neighbours_with_column_label
            * (number_of_src_neighbours_with_row_label + number_of_dst_neighbours_with_row_label)
        - typed_chordal_cycle_edge_count
}

#[inline(always)]
/// Returns the number of typed chordal-cycle center orbit associated to the provided edge.
///
/// # Arguments
/// * `typed_chordal_cycle_center_count`: the number of typed chordal cycle center for the current edge and the provided node types.
/// * `number_of_triangle_forming_neighbours_with_row_label`: the number of neighbours with the first type that form a triangle with the current edge.
/// * `number_of_triangle_forming_neighbours_with_column_label`: the number of neighbours with the second type that form a triangle with the current edge.
///
/// # References
/// The formula reported in this code is taken from the "Heterogeneous Graphlets" paper
/// and specifically from the equation 30.
///
pub(crate) fn get_homogeneously_typed_chordal_cycle_center_orbit_count<
    C: Mul<C, Output = C>
        + Add<C, Output = C>
        + Sub<C, Output = C>
        + Div<C, Output = C>
        + Ord
        + Zero
        + One
        + Two
        + Copy,
>(
    number_of_four_cliques_count: C,
    number_of_triangle_forming_neighbours: C,
) -> C {
    binomial_two(number_of_triangle_forming_neighbours) - number_of_four_cliques_count
}

#[inline(always)]
/// Returns the number of typed chordal-cycle center orbit associated to the provided edge.
///
/// # Arguments
/// * `number_of_four_cliques_count`: the number of typed chordal cycle center for the current edge and the provided node types.
/// * `number_of_triangle_forming_neighbours_with_row_label`: the number of neighbours with the first type that form a triangle with the current edge.
/// * `number_of_triangle_forming_neighbours_with_column_label`: the number of neighbours with the second type that form a triangle with the current edge.
///
/// # References
/// The formula reported in this code is taken from the "Heterogeneous Graphlets" paper
/// and specifically from the equation 30.
///
pub(crate) fn get_heterogeneously_typed_chordal_cycle_center_orbit_count<
    C: Mul<C, Output = C>
        + Add<C, Output = C>
        + Sub<C, Output = C>
        + Div<C, Output = C>
        + Ord
        + Zero
        + One
        + Two
        + Copy,
>(
    number_of_four_cliques_count: C,
    number_of_triangle_forming_neighbours_with_row_label: C,
    number_of_triangle_forming_neighbours_with_column_label: C,
) -> C {
    number_of_triangle_forming_neighbours_with_row_label
        * number_of_triangle_forming_neighbours_with_column_label
        - number_of_four_cliques_count
}
