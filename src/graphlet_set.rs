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
        match value {
            11 => Self::FourClique,
            10 => Self::ChordalCycleCenter,
            9 => Self::ChordalCycleEdge,
            8 => Self::TailedTriEdge,
            7 => Self::TailedTriCenter,
            6 => Self::TailedTriTail,
            5 => Self::FourCycle,
            4 => Self::FourStar,
            3 => Self::FourPathCenter,
            2 => Self::FourPathEdge,
            1 => Self::Triangle,
            0 => Self::Triad,
            // This conversion is used internally on the trusted perfect-hash
            // decode path, where the value is always in range. Use the fallible
            // `TryFrom<u8>` impl for untrusted input.
            _ => unreachable!("invalid extended graphlet type index: {}", value),
        }
    }
}

impl From<u8> for ReducedGraphletType {
    fn from(value: u8) -> Self {
        match value {
            7 => Self::FourClique,
            6 => Self::ChordalCycle,
            5 => Self::TailedTri,
            4 => Self::FourCycle,
            3 => Self::FourStar,
            2 => Self::FourPath,
            1 => Self::Triangle,
            0 => Self::Triad,
            // This conversion is used internally on the trusted perfect-hash
            // decode path, where the value is always in range. Use the fallible
            // `TryFrom<u8>` impl for untrusted input.
            _ => unreachable!("invalid reduced graphlet type index: {}", value),
        }
    }
}

impl ExtendedGraphletType {
    /// Builds an [`ExtendedGraphletType`] from its numeric index, validating the range.
    ///
    /// Unlike the infallible `From<u8>` impl (used internally on the trusted
    /// perfect-hash decode path), this returns a [`GraphletError`] for an
    /// out-of-range value, so it is the conversion to use for untrusted input.
    ///
    /// # Errors
    /// Returns [`GraphletError::InvalidGraphletType`] if `value` is not a valid
    /// graphlet index (i.e. `value >= 12`).
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
    /// Builds a [`ReducedGraphletType`] from its numeric index, validating the range.
    ///
    /// Unlike the infallible `From<u8>` impl (used internally on the trusted
    /// perfect-hash decode path), this returns a [`GraphletError`] for an
    /// out-of-range value, so it is the conversion to use for untrusted input.
    ///
    /// # Errors
    /// Returns [`GraphletError::InvalidGraphletType`] if `value` is not a valid
    /// graphlet index (i.e. `value >= 8`).
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
}
