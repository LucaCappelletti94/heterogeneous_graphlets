use crate::numbers::Two;
use core::ops::{Add, Div, Mul, Sub};
use num_traits::{One, Zero};

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
        C::zero()
    } else {
        x * (x - C::one()) / C::TWO
    }
}

#[inline(always)]
/// Returns the number of 4-paths orbit associated to the provided edge.
///
/// # Arguments
/// * `four_cycle_count` - The number of 4-cycles associated to the currently considered edge.
/// * `number_of_src_neighbours` - The number of neighbours of the first node of the currently considered edge with the same type of the first node.
/// * `number_of_dst_neighbours` - The number of neighbours of the second node of the currently considered edge with the same type of the first node.
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
/// * `number_of_src_neighbours_with_row_label` - The number of neighbours of the first node of the currently considered edge with the same type of the first node.
/// * `number_of_dst_neighbours_with_row_label` - The number of neighbours of the second node of the currently considered edge with the same type of the first node.
/// * `number_of_src_neighbours_with_column_label` - The number of neighbours of the first node of the currently considered edge with the same type of the second node.
/// * `number_of_dst_neighbours_with_column_label` - The number of neighbours of the second node of the currently considered edge with the same type of the second node.
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
/// * `number_of_src_neighbours` - The number of neighbours of the first node of the currently considered edge with the same type of the first node.
/// * `number_of_dst_neighbours` - The number of neighbours of the second node of the currently considered edge with the same type of the first node.
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
/// * `number_of_src_neighbours_with_row_label` - The number of neighbours of the first node of the currently considered edge with the same type of the first node.
/// * `number_of_dst_neighbours_with_row_label` - The number of neighbours of the second node of the currently considered edge with the same type of the first node.
/// * `number_of_src_neighbours_with_column_label` - The number of neighbours of the first node of the currently considered edge with the same type of the second node.
/// * `number_of_dst_neighbours_with_column_label` - The number of neighbours of the second node of the currently considered edge with the same type of the second node.
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
/// * `number_of_triangles`: the number of neighbours with the first type that form a triangle with the current edge.
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
    number_of_triangles: C,
    number_of_src_neighbours: C,
    number_of_dst_neighbours: C,
) -> C {
    number_of_triangles * (number_of_src_neighbours + number_of_dst_neighbours)
        - typed_chordal_cycle_edge_count
}

#[inline(always)]
/// Returns the number of typed tailed triangle tri-edge orbit associated to the provided edge.
///
/// # Arguments
/// * `typed_chordal_cycle_edge_count`: the number of typed chordal cycle edges for the current edge and the provided node types.
/// * `number_of_triangles_with_row_label`: the number of neighbours with the first type that form a triangle with the current edge.
/// * `number_of_triangles_with_column_label`: the number of neighbours with the second type that form a triangle with the current edge.
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
    number_of_triangles_with_row_label: C,
    number_of_triangles_with_column_label: C,
    number_of_src_neighbours_with_row_label: C,
    number_of_dst_neighbours_with_row_label: C,
    number_of_src_neighbours_with_column_label: C,
    number_of_dst_neighbours_with_column_label: C,
) -> C {
    number_of_triangles_with_row_label
        * (number_of_src_neighbours_with_column_label + number_of_dst_neighbours_with_column_label)
        + number_of_triangles_with_column_label
            * (number_of_src_neighbours_with_row_label + number_of_dst_neighbours_with_row_label)
        - typed_chordal_cycle_edge_count
}

#[inline(always)]
/// Returns the number of typed chordal-cycle center orbit associated to the provided edge.
///
/// # Arguments
/// * `typed_chordal_cycle_center_count`: the number of typed chordal cycle center for the current edge and the provided node types.
/// * `number_of_triangles_with_row_label`: the number of neighbours with the first type that form a triangle with the current edge.
/// * `number_of_triangles_with_column_label`: the number of neighbours with the second type that form a triangle with the current edge.
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
    number_of_triangles: C,
) -> C {
    binomial_two(number_of_triangles) - number_of_four_cliques_count
}

#[inline(always)]
/// Returns the number of typed chordal-cycle center orbit associated to the provided edge.
///
/// # Arguments
/// * `number_of_four_cliques_count`: the number of typed chordal cycle center for the current edge and the provided node types.
/// * `number_of_triangles_with_row_label`: the number of neighbours with the first type that form a triangle with the current edge.
/// * `number_of_triangles_with_column_label`: the number of neighbours with the second type that form a triangle with the current edge.
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
    number_of_triangles_with_row_label: C,
    number_of_triangles_with_column_label: C,
) -> C {
    number_of_triangles_with_row_label * number_of_triangles_with_column_label
        - number_of_four_cliques_count
}

#[cfg(test)]
mod tests {
    use super::*;

    // Each orbit formula is checked with concrete operands chosen so that every
    // operation (+, -, *, /) produces a distinct result, pinning the exact
    // arithmetic of the paper's equations 19, 23, 26 and 30.

    #[test]
    fn homogeneous_four_path() {
        // src * dst - four_cycle = 3 * 4 - 2 = 10
        assert_eq!(get_homogeneously_typed_four_path_orbit_count(2u32, 3, 4), 10);
    }

    #[test]
    fn heterogeneous_four_path() {
        // src_row * dst_col + src_col * dst_row - four_cycle = 2*5 + 4*3 - 1 = 21
        assert_eq!(
            get_heterogeneously_typed_four_path_orbit_count(1u32, 2, 3, 4, 5),
            21
        );
    }

    #[test]
    fn homogeneous_four_star() {
        // binom2(src) + binom2(dst) - tailed = 6 + 10 - 3 = 13
        assert_eq!(get_homogeneously_typed_four_star_orbit_count(3u32, 4, 5), 13);
    }

    #[test]
    fn heterogeneous_four_star() {
        // src_row * src_col + dst_col * dst_row - tailed = 2*4 + 5*3 - 1 = 22
        assert_eq!(
            get_heterogeneously_typed_four_star_orbit_count(1u32, 2, 3, 4, 5),
            22
        );
    }

    #[test]
    fn homogeneous_tailed_triangle_tri_edge() {
        // triangles * (src + dst) - chordal_edge = 2 * (3 + 4) - 1 = 13
        assert_eq!(
            get_homogeneously_typed_tailed_triangle_tri_edge_orbit_count(1u32, 2, 3, 4),
            13
        );
    }

    #[test]
    fn heterogeneous_tailed_triangle_tri_edge() {
        // tri_row*(src_col+dst_col) + tri_col*(src_row+dst_row) - chordal_edge
        //   = 2*(6+7) + 3*(4+5) - 1 = 52
        assert_eq!(
            get_heterogeneously_typed_tailed_triangle_tri_edge_orbit_count(1u32, 2, 3, 4, 5, 6, 7),
            52
        );
    }

    #[test]
    fn homogeneous_chordal_cycle_center() {
        // binom2(triangles) - four_clique = 10 - 1 = 9
        assert_eq!(
            get_homogeneously_typed_chordal_cycle_center_orbit_count(1u32, 5),
            9
        );
    }

    #[test]
    fn heterogeneous_chordal_cycle_center() {
        // tri_row * tri_col - four_clique = 3 * 4 - 1 = 11
        assert_eq!(
            get_heterogeneously_typed_chordal_cycle_center_orbit_count(1u32, 3, 4),
            11
        );
    }

    #[test]
    fn binomial_two_below_two_is_zero() {
        // Exercises the x < 2 branch of binomial_two via the four-star formula:
        // binom2(1) + binom2(1) - 0 = 0
        assert_eq!(get_homogeneously_typed_four_star_orbit_count(0u32, 1, 1), 0);
    }
}
