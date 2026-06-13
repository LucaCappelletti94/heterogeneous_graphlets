use crate::error::GraphletError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtendedGraphletType {
    FourClique,
    ChordalCycleCenter,
    ChordalCycleEdge,
    TailedTriEdge,
    TailedTriCenter,
    TailedTriTail,
    FourCycle,
    FourStar,
    FourPathCenter,
    FourPathEdge,
    Triangle,
    Triad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReducedGraphletType {
    FourClique,
    ChordalCycle,
    TailedTri,
    FourCycle,
    FourStar,
    FourPath,
    Triangle,
    Triad,
}

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
            11 => ExtendedGraphletType::FourClique,
            10 => ExtendedGraphletType::ChordalCycleCenter,
            9 => ExtendedGraphletType::ChordalCycleEdge,
            8 => ExtendedGraphletType::TailedTriEdge,
            7 => ExtendedGraphletType::TailedTriCenter,
            6 => ExtendedGraphletType::TailedTriTail,
            5 => ExtendedGraphletType::FourCycle,
            4 => ExtendedGraphletType::FourStar,
            3 => ExtendedGraphletType::FourPathCenter,
            2 => ExtendedGraphletType::FourPathEdge,
            1 => ExtendedGraphletType::Triangle,
            0 => ExtendedGraphletType::Triad,
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
            7 => ReducedGraphletType::FourClique,
            6 => ReducedGraphletType::ChordalCycle,
            5 => ReducedGraphletType::TailedTri,
            4 => ReducedGraphletType::FourCycle,
            3 => ReducedGraphletType::FourStar,
            2 => ReducedGraphletType::FourPath,
            1 => ReducedGraphletType::Triangle,
            0 => ReducedGraphletType::Triad,
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
