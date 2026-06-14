//! The sets of graphlet kinds (orbits) that can be counted.

use crate::error::GraphletError;

/// The twelve edge orbits of the 4-node graphlets, distinguishing the position
/// of the edge within each graphlet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtendedGraphletType {
    /// The 4-clique orbit.
    FourClique,
    /// The center-edge orbit of the chordal cycle (diamond).
    ChordalCycleCenter,
    /// The outer-edge orbit of the chordal cycle (diamond).
    ChordalCycleEdge,
    /// The triangle-edge orbit of the tailed triangle.
    TailedTriEdge,
    /// The center-edge orbit of the tailed triangle.
    TailedTriCenter,
    /// The tail-edge orbit of the tailed triangle.
    TailedTriTail,
    /// The 4-cycle orbit.
    FourCycle,
    /// The 4-star (claw) orbit.
    FourStar,
    /// The center-edge orbit of the 4-path.
    FourPathCenter,
    /// The end-edge orbit of the 4-path.
    FourPathEdge,
    /// The triangle orbit.
    Triangle,
    /// The triad (3-path) orbit.
    Triad,
}

/// The eight graphlet kinds, without distinguishing edge orbits within a kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ReducedGraphletType {
    /// The 4-clique.
    FourClique,
    /// The chordal cycle (diamond).
    ChordalCycle,
    /// The tailed triangle.
    TailedTri,
    /// The 4-cycle.
    FourCycle,
    /// The 4-star (claw).
    FourStar,
    /// The 4-path.
    FourPath,
    /// The triangle.
    Triangle,
    /// The triad (3-path).
    Triad,
}

/// A set of graphlet kinds whose cardinality is known at the type level.
///
/// # Examples
/// ```
/// use heterogeneous_graphlets::prelude::*;
///
/// assert_eq!(
///     <ExtendedGraphletType as GraphletSet<u8>>::get_number_of_graphlets(),
///     12,
/// );
/// assert_eq!(
///     <ReducedGraphletType as GraphletSet<u8>>::get_number_of_graphlets(),
///     8,
/// );
/// ```
pub trait GraphletSet<C> {
    /// Returns the number of graphlets of the current type.
    fn get_number_of_graphlets() -> C;
}

/// Generates, for each listed integer width, the [`GraphletSet`] count impl and
/// the `From` conversions that widen to and narrow from the canonical `u8`
/// representation. The `u8` impls, which hold the actual match arms, are written
/// by hand below.
macro_rules! impl_graphlet_widths {
    ($enum:ty, $count:literal, $($int:ty),+ $(,)?) => {
        $(
            impl GraphletSet<$int> for $enum {
                fn get_number_of_graphlets() -> $int {
                    $count
                }
            }

            impl From<$int> for $enum {
                fn from(value: $int) -> Self {
                    Self::from(value as u8)
                }
            }

            impl From<$enum> for $int {
                fn from(value: $enum) -> Self {
                    u8::from(value) as $int
                }
            }
        )+
    };
}

impl_graphlet_widths!(ExtendedGraphletType, 12, u16, u32, u64, u128, usize);
impl_graphlet_widths!(ReducedGraphletType, 8, u16, u32, u64, u128, usize);

impl GraphletSet<u8> for ExtendedGraphletType {
    fn get_number_of_graphlets() -> u8 {
        12
    }
}

impl GraphletSet<u8> for ReducedGraphletType {
    fn get_number_of_graphlets() -> u8 {
        8
    }
}

impl ExtendedGraphletType {
    /// Name of the four-clique orbit.
    pub const FOUR_CLIQUE: &str = "FourClique";
    /// Name of the chordal-cycle centre-edge orbit.
    pub const CHORDAL_CYCLE_CENTER: &str = "ChordalCycleCenter";
    /// Name of the chordal-cycle outer-edge orbit.
    pub const CHORDAL_CYCLE_EDGE: &str = "ChordalCycleEdge";
    /// Name of the tailed-triangle triangle-edge orbit.
    pub const TAILED_TRI_EDGE: &str = "TailedTriEdge";
    /// Name of the tailed-triangle centre-edge orbit.
    pub const TAILED_TRI_CENTER: &str = "TailedTriCenter";
    /// Name of the tailed-triangle tail-edge orbit.
    pub const TAILED_TRI_TAIL: &str = "TailedTriTail";
    /// Name of the four-cycle orbit.
    pub const FOUR_CYCLE: &str = "FourCycle";
    /// Name of the four-star orbit.
    pub const FOUR_STAR: &str = "FourStar";
    /// Name of the four-path centre-edge orbit.
    pub const FOUR_PATH_CENTER: &str = "FourPathCenter";
    /// Name of the four-path end-edge orbit.
    pub const FOUR_PATH_EDGE: &str = "FourPathEdge";
    /// Name of the triangle orbit.
    pub const TRIANGLE: &str = "Triangle";
    /// Name of the triad (3-path) orbit.
    pub const TRIAD: &str = "Triad";

    /// The orbit's name, which is the key it is reported under by
    /// [`GraphLetCounter::to_graphlet_names`](crate::graphlet_counter::GraphLetCounter::to_graphlet_names).
    /// Use the associated name constants (for example [`ExtendedGraphletType::TRIANGLE`])
    /// to look up an orbit rather than spelling the string literal.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::FourClique => Self::FOUR_CLIQUE,
            Self::ChordalCycleCenter => Self::CHORDAL_CYCLE_CENTER,
            Self::ChordalCycleEdge => Self::CHORDAL_CYCLE_EDGE,
            Self::TailedTriEdge => Self::TAILED_TRI_EDGE,
            Self::TailedTriCenter => Self::TAILED_TRI_CENTER,
            Self::TailedTriTail => Self::TAILED_TRI_TAIL,
            Self::FourCycle => Self::FOUR_CYCLE,
            Self::FourStar => Self::FOUR_STAR,
            Self::FourPathCenter => Self::FOUR_PATH_CENTER,
            Self::FourPathEdge => Self::FOUR_PATH_EDGE,
            Self::Triangle => Self::TRIANGLE,
            Self::Triad => Self::TRIAD,
        }
    }
}

impl ReducedGraphletType {
    /// Name of the four-clique orbit.
    pub const FOUR_CLIQUE: &str = "FourClique";
    /// Name of the chordal-cycle (diamond) orbit.
    pub const CHORDAL_CYCLE: &str = "ChordalCycle";
    /// Name of the tailed-triangle orbit.
    pub const TAILED_TRI: &str = "TailedTri";
    /// Name of the four-cycle orbit.
    pub const FOUR_CYCLE: &str = "FourCycle";
    /// Name of the four-star orbit.
    pub const FOUR_STAR: &str = "FourStar";
    /// Name of the four-path orbit.
    pub const FOUR_PATH: &str = "FourPath";
    /// Name of the triangle orbit.
    pub const TRIANGLE: &str = "Triangle";
    /// Name of the triad (3-path) orbit.
    pub const TRIAD: &str = "Triad";

    /// The kind's name, which is the key it is reported under by
    /// [`GraphLetCounter::to_graphlet_names`](crate::graphlet_counter::GraphLetCounter::to_graphlet_names).
    /// Use the associated name constants (for example [`ReducedGraphletType::TRIANGLE`])
    /// to look up a kind rather than spelling the string literal.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::FourClique => Self::FOUR_CLIQUE,
            Self::ChordalCycle => Self::CHORDAL_CYCLE,
            Self::TailedTri => Self::TAILED_TRI,
            Self::FourCycle => Self::FOUR_CYCLE,
            Self::FourStar => Self::FOUR_STAR,
            Self::FourPath => Self::FOUR_PATH,
            Self::Triangle => Self::TRIANGLE,
            Self::Triad => Self::TRIAD,
        }
    }

    /// The number of edges of this graphlet, `E_g`.
    ///
    /// When per-edge graphlet counts are summed over every edge of a graph, each
    /// occurrence is tallied once per edge it contains, so the whole-graph total
    /// for a kind is exactly `E_g` times the number of distinct occurrences. This
    /// is the divisor that recovers exact occurrence counts when deduplicating a
    /// whole-graph signature.
    #[must_use]
    // Each arm states one graphlet's edge count, an independent structural fact;
    // distinct kinds that happen to share a count stay on their own arm for clarity.
    #[allow(clippy::match_same_arms)]
    pub const fn number_of_edges(self) -> usize {
        match self {
            Self::FourClique => 6,
            Self::ChordalCycle => 5,
            Self::TailedTri => 4,
            Self::FourCycle => 4,
            Self::FourStar => 3,
            Self::FourPath => 3,
            Self::Triangle => 3,
            Self::Triad => 2,
        }
    }
}

impl From<ExtendedGraphletType> for ReducedGraphletType {
    /// Collapses an edge orbit to its graphlet kind, forgetting which edge of the
    /// graphlet the orbit distinguished (for example both chordal-cycle orbits
    /// become [`ReducedGraphletType::ChordalCycle`]).
    fn from(value: ExtendedGraphletType) -> Self {
        match value {
            ExtendedGraphletType::FourClique => Self::FourClique,
            ExtendedGraphletType::ChordalCycleCenter | ExtendedGraphletType::ChordalCycleEdge => {
                Self::ChordalCycle
            }
            ExtendedGraphletType::TailedTriEdge
            | ExtendedGraphletType::TailedTriCenter
            | ExtendedGraphletType::TailedTriTail => Self::TailedTri,
            ExtendedGraphletType::FourCycle => Self::FourCycle,
            ExtendedGraphletType::FourStar => Self::FourStar,
            ExtendedGraphletType::FourPathCenter | ExtendedGraphletType::FourPathEdge => {
                Self::FourPath
            }
            ExtendedGraphletType::Triangle => Self::Triangle,
            ExtendedGraphletType::Triad => Self::Triad,
        }
    }
}

impl core::fmt::Display for ExtendedGraphletType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let name: &str = self.into();
        f.write_str(name)
    }
}

impl core::fmt::Display for ReducedGraphletType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let name: &str = self.into();
        f.write_str(name)
    }
}

impl From<&ExtendedGraphletType> for &str {
    fn from(value: &ExtendedGraphletType) -> Self {
        value.name()
    }
}

impl From<&ReducedGraphletType> for &str {
    fn from(value: &ReducedGraphletType) -> Self {
        value.name()
    }
}

impl From<u8> for ExtendedGraphletType {
    fn from(value: u8) -> Self {
        // Indices 0..12 are valid. This conversion is used internally on the
        // trusted perfect-hash decode path, where the value is always in range.
        // `try_from_index` is the checked conversion for untrusted input. We fall
        // back to a defined variant for out-of-range values rather than panicking,
        // so the crate stays free of panicking macros.
        Self::VARIANTS
            .get(value as usize)
            .copied()
            .unwrap_or(Self::Triad)
    }
}

impl From<u8> for ReducedGraphletType {
    fn from(value: u8) -> Self {
        // Indices 0..8 are valid. This conversion is used internally on the
        // trusted perfect-hash decode path, where the value is always in range.
        // `try_from_index` is the checked conversion for untrusted input. We fall
        // back to a defined variant for out-of-range values rather than panicking,
        // so the crate stays free of panicking macros.
        Self::VARIANTS
            .get(value as usize)
            .copied()
            .unwrap_or(Self::Triad)
    }
}

impl ExtendedGraphletType {
    /// All twelve variants in numeric-index order, so `VARIANTS[i]` is the
    /// variant whose `u8` index is `i`.
    const VARIANTS: [Self; 12] = [
        Self::Triad,
        Self::Triangle,
        Self::FourPathEdge,
        Self::FourPathCenter,
        Self::FourStar,
        Self::FourCycle,
        Self::TailedTriTail,
        Self::TailedTriCenter,
        Self::TailedTriEdge,
        Self::ChordalCycleEdge,
        Self::ChordalCycleCenter,
        Self::FourClique,
    ];

    /// Builds an [`ExtendedGraphletType`] from its numeric index, validating the range.
    ///
    /// Unlike the infallible `From<u8>` impl (used internally on the trusted
    /// perfect-hash decode path), this returns a [`GraphletError`] for an
    /// out-of-range value, so it is the conversion to use for untrusted input.
    ///
    /// # Errors
    /// Returns [`GraphletError::InvalidGraphletType`] if `value` is not a valid
    /// graphlet index (i.e. `value >= 12`).
    ///
    /// # Examples
    /// ```
    /// use heterogeneous_graphlets::prelude::*;
    ///
    /// assert_eq!(
    ///     ExtendedGraphletType::try_from_index(0),
    ///     Ok(ExtendedGraphletType::Triad),
    /// );
    /// assert_eq!(
    ///     ExtendedGraphletType::try_from_index(11),
    ///     Ok(ExtendedGraphletType::FourClique),
    /// );
    /// assert_eq!(
    ///     ExtendedGraphletType::try_from_index(12),
    ///     Err(GraphletError::InvalidGraphletType { value: 12, max: 12 }),
    /// );
    /// ```
    pub fn try_from_index(value: u8) -> Result<Self, GraphletError> {
        let max = <Self as GraphletSet<u8>>::get_number_of_graphlets();
        if value < max {
            Ok(Self::from(value))
        } else {
            Err(GraphletError::InvalidGraphletType { value, max })
        }
    }
}

impl ReducedGraphletType {
    /// All eight variants in numeric-index order, so `VARIANTS[i]` is the variant
    /// whose `u8` index is `i`.
    const VARIANTS: [Self; 8] = [
        Self::Triad,
        Self::Triangle,
        Self::FourPath,
        Self::FourStar,
        Self::FourCycle,
        Self::TailedTri,
        Self::ChordalCycle,
        Self::FourClique,
    ];

    /// Builds a [`ReducedGraphletType`] from its numeric index, validating the range.
    ///
    /// Unlike the infallible `From<u8>` impl (used internally on the trusted
    /// perfect-hash decode path), this returns a [`GraphletError`] for an
    /// out-of-range value, so it is the conversion to use for untrusted input.
    ///
    /// # Errors
    /// Returns [`GraphletError::InvalidGraphletType`] if `value` is not a valid
    /// graphlet index (i.e. `value >= 8`).
    ///
    /// # Examples
    /// ```
    /// use heterogeneous_graphlets::prelude::*;
    ///
    /// assert_eq!(
    ///     ReducedGraphletType::try_from_index(0),
    ///     Ok(ReducedGraphletType::Triad),
    /// );
    /// assert_eq!(
    ///     ReducedGraphletType::try_from_index(7),
    ///     Ok(ReducedGraphletType::FourClique),
    /// );
    /// assert!(ReducedGraphletType::try_from_index(8).is_err());
    /// ```
    pub fn try_from_index(value: u8) -> Result<Self, GraphletError> {
        let max = <Self as GraphletSet<u8>>::get_number_of_graphlets();
        if value < max {
            Ok(Self::from(value))
        } else {
            Err(GraphletError::InvalidGraphletType { value, max })
        }
    }
}

impl From<ExtendedGraphletType> for u8 {
    fn from(value: ExtendedGraphletType) -> Self {
        match value {
            ExtendedGraphletType::FourClique => 11,
            ExtendedGraphletType::ChordalCycleCenter => 10,
            ExtendedGraphletType::ChordalCycleEdge => 9,
            ExtendedGraphletType::TailedTriEdge => 8,
            ExtendedGraphletType::TailedTriCenter => 7,
            ExtendedGraphletType::TailedTriTail => 6,
            ExtendedGraphletType::FourCycle => 5,
            ExtendedGraphletType::FourStar => 4,
            ExtendedGraphletType::FourPathCenter => 3,
            ExtendedGraphletType::FourPathEdge => 2,
            ExtendedGraphletType::Triangle => 1,
            ExtendedGraphletType::Triad => 0,
        }
    }
}

impl From<ReducedGraphletType> for u8 {
    fn from(value: ReducedGraphletType) -> Self {
        match value {
            ReducedGraphletType::FourClique => 7,
            ReducedGraphletType::ChordalCycle => 6,
            ReducedGraphletType::TailedTri => 5,
            ReducedGraphletType::FourCycle => 4,
            ReducedGraphletType::FourStar => 3,
            ReducedGraphletType::FourPath => 2,
            ReducedGraphletType::Triangle => 1,
            ReducedGraphletType::Triad => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_accepts_valid_extended_indices() {
        for value in 0..12u8 {
            assert!(ExtendedGraphletType::try_from_index(value).is_ok());
        }
    }

    #[test]
    fn try_from_rejects_out_of_range_extended_index() {
        assert_eq!(
            ExtendedGraphletType::try_from_index(12),
            Err(GraphletError::InvalidGraphletType { value: 12, max: 12 })
        );
    }

    #[test]
    fn try_from_accepts_valid_reduced_indices() {
        for value in 0..8u8 {
            assert!(ReducedGraphletType::try_from_index(value).is_ok());
        }
    }

    #[test]
    fn try_from_rejects_out_of_range_reduced_index() {
        assert_eq!(
            ReducedGraphletType::try_from_index(8),
            Err(GraphletError::InvalidGraphletType { value: 8, max: 8 })
        );
    }

    #[test]
    fn extended_index_name_roundtrip() {
        let cases = [
            (ExtendedGraphletType::FourClique, 11u8, "FourClique"),
            (
                ExtendedGraphletType::ChordalCycleCenter,
                10,
                "ChordalCycleCenter",
            ),
            (
                ExtendedGraphletType::ChordalCycleEdge,
                9,
                "ChordalCycleEdge",
            ),
            (ExtendedGraphletType::TailedTriEdge, 8, "TailedTriEdge"),
            (ExtendedGraphletType::TailedTriCenter, 7, "TailedTriCenter"),
            (ExtendedGraphletType::TailedTriTail, 6, "TailedTriTail"),
            (ExtendedGraphletType::FourCycle, 5, "FourCycle"),
            (ExtendedGraphletType::FourStar, 4, "FourStar"),
            (ExtendedGraphletType::FourPathCenter, 3, "FourPathCenter"),
            (ExtendedGraphletType::FourPathEdge, 2, "FourPathEdge"),
            (ExtendedGraphletType::Triangle, 1, "Triangle"),
            (ExtendedGraphletType::Triad, 0, "Triad"),
        ];
        for (variant, index, name) in cases {
            assert_eq!(u8::from(variant), index);
            assert_eq!(ExtendedGraphletType::from(index), variant);
            assert_eq!(<&str>::from(&variant), name);
            assert_eq!(alloc::format!("{variant}"), name);
        }
    }

    #[test]
    fn reduced_index_name_roundtrip() {
        let cases = [
            (ReducedGraphletType::FourClique, 7u8, "FourClique"),
            (ReducedGraphletType::ChordalCycle, 6, "ChordalCycle"),
            (ReducedGraphletType::TailedTri, 5, "TailedTri"),
            (ReducedGraphletType::FourCycle, 4, "FourCycle"),
            (ReducedGraphletType::FourStar, 3, "FourStar"),
            (ReducedGraphletType::FourPath, 2, "FourPath"),
            (ReducedGraphletType::Triangle, 1, "Triangle"),
            (ReducedGraphletType::Triad, 0, "Triad"),
        ];
        for (variant, index, name) in cases {
            assert_eq!(u8::from(variant), index);
            assert_eq!(ReducedGraphletType::from(index), variant);
            assert_eq!(<&str>::from(&variant), name);
            assert_eq!(alloc::format!("{variant}"), name);
        }
    }

    #[test]
    fn orbit_reduces_to_expected_kind() {
        let cases = [
            (
                ExtendedGraphletType::FourClique,
                ReducedGraphletType::FourClique,
            ),
            (
                ExtendedGraphletType::ChordalCycleCenter,
                ReducedGraphletType::ChordalCycle,
            ),
            (
                ExtendedGraphletType::ChordalCycleEdge,
                ReducedGraphletType::ChordalCycle,
            ),
            (
                ExtendedGraphletType::TailedTriEdge,
                ReducedGraphletType::TailedTri,
            ),
            (
                ExtendedGraphletType::TailedTriCenter,
                ReducedGraphletType::TailedTri,
            ),
            (
                ExtendedGraphletType::TailedTriTail,
                ReducedGraphletType::TailedTri,
            ),
            (
                ExtendedGraphletType::FourCycle,
                ReducedGraphletType::FourCycle,
            ),
            (
                ExtendedGraphletType::FourStar,
                ReducedGraphletType::FourStar,
            ),
            (
                ExtendedGraphletType::FourPathCenter,
                ReducedGraphletType::FourPath,
            ),
            (
                ExtendedGraphletType::FourPathEdge,
                ReducedGraphletType::FourPath,
            ),
            (
                ExtendedGraphletType::Triangle,
                ReducedGraphletType::Triangle,
            ),
            (ExtendedGraphletType::Triad, ReducedGraphletType::Triad),
        ];
        for (orbit, kind) in cases {
            assert_eq!(ReducedGraphletType::from(orbit), kind);
            // The orbit name always begins with the reduced kind name, so the two
            // mappings agree (e.g. "ChordalCycleEdge" starts with "ChordalCycle").
            assert!(
                orbit.name().starts_with(kind.name()),
                "{} does not start with {}",
                orbit.name(),
                kind.name(),
            );
        }
    }

    #[test]
    fn number_of_edges_matches_graphlet_structure() {
        let cases = [
            (ReducedGraphletType::FourClique, 6),
            (ReducedGraphletType::ChordalCycle, 5),
            (ReducedGraphletType::TailedTri, 4),
            (ReducedGraphletType::FourCycle, 4),
            (ReducedGraphletType::FourStar, 3),
            (ReducedGraphletType::FourPath, 3),
            (ReducedGraphletType::Triangle, 3),
            (ReducedGraphletType::Triad, 2),
        ];
        for (kind, edges) in cases {
            assert_eq!(kind.number_of_edges(), edges, "{}", kind.name());
        }
    }

    #[test]
    fn reduced_graphlet_type_is_ord() {
        // Needed so it can key a BTreeMap of deduplicated graphlet counts.
        let mut sorted = ReducedGraphletType::VARIANTS;
        sorted.sort_unstable();
        assert_eq!(sorted.len(), 8);
        for pair in sorted.windows(2) {
            assert!(pair[0] <= pair[1]);
        }
    }
}
