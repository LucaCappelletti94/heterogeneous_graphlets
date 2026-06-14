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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
        match value {
            ExtendedGraphletType::FourClique => "FourClique",
            ExtendedGraphletType::ChordalCycleCenter => "ChordalCycleCenter",
            ExtendedGraphletType::ChordalCycleEdge => "ChordalCycleEdge",
            ExtendedGraphletType::TailedTriEdge => "TailedTriEdge",
            ExtendedGraphletType::TailedTriCenter => "TailedTriCenter",
            ExtendedGraphletType::TailedTriTail => "TailedTriTail",
            ExtendedGraphletType::FourCycle => "FourCycle",
            ExtendedGraphletType::FourStar => "FourStar",
            ExtendedGraphletType::FourPathCenter => "FourPathCenter",
            ExtendedGraphletType::FourPathEdge => "FourPathEdge",
            ExtendedGraphletType::Triangle => "Triangle",
            ExtendedGraphletType::Triad => "Triad",
        }
    }
}

impl From<&ReducedGraphletType> for &str {
    fn from(value: &ReducedGraphletType) -> Self {
        match value {
            ReducedGraphletType::FourClique => "FourClique",
            ReducedGraphletType::ChordalCycle => "ChordalCycle",
            ReducedGraphletType::TailedTri => "TailedTri",
            ReducedGraphletType::FourCycle => "FourCycle",
            ReducedGraphletType::FourStar => "FourStar",
            ReducedGraphletType::FourPath => "FourPath",
            ReducedGraphletType::Triangle => "Triangle",
            ReducedGraphletType::Triad => "Triad",
        }
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
}
