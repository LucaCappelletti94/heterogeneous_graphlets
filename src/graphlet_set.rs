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

impl GraphletSet<u16> for ExtendedGraphletType {
    fn get_number_of_graphlets() -> u16 {
        12
    }
}

impl GraphletSet<u16> for ReducedGraphletType {
    fn get_number_of_graphlets() -> u16 {
        8
    }
}

impl GraphletSet<u32> for ExtendedGraphletType {
    fn get_number_of_graphlets() -> u32 {
        12
    }
}

impl GraphletSet<u32> for ReducedGraphletType {
    fn get_number_of_graphlets() -> u32 {
        8
    }
}

impl GraphletSet<u64> for ExtendedGraphletType {
    fn get_number_of_graphlets() -> u64 {
        12
    }
}

impl GraphletSet<u64> for ReducedGraphletType {
    fn get_number_of_graphlets() -> u64 {
        8
    }
}

impl GraphletSet<usize> for ExtendedGraphletType {
    fn get_number_of_graphlets() -> usize {
        12
    }
}

impl GraphletSet<usize> for ReducedGraphletType {
    fn get_number_of_graphlets() -> usize {
        8
    }
}

impl ToString for ExtendedGraphletType {
    fn to_string(&self) -> String {
        let name: &str = self.into();
        name.to_string()
    }
}

impl ToString for ReducedGraphletType {
    fn to_string(&self) -> String {
        let name: &str = self.into();
        name.to_string()
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
            _ => panic!(
                "Invalid graphlet type: {} (should be between 0 and 11)",
                value
            ),
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
            _ => panic!(
                "Invalid graphlet type: {} (should be between 0 and 7)",
                value
            ),
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

impl From<u16> for ExtendedGraphletType {
    fn from(value: u16) -> Self {
        ExtendedGraphletType::from(value as u8)
    }
}

impl From<ReducedGraphletType> for u16 {
    fn from(value: ReducedGraphletType) -> Self {
        u8::from(value) as u16
    }
}

impl From<u16> for ReducedGraphletType {
    fn from(value: u16) -> Self {
        ReducedGraphletType::from(value as u8)
    }
}

impl From<ExtendedGraphletType> for u16 {
    fn from(value: ExtendedGraphletType) -> Self {
        u8::from(value) as u16
    }
}

impl From<u32> for ReducedGraphletType {
    fn from(value: u32) -> Self {
        ReducedGraphletType::from(value as u8)
    }
}

impl From<u32> for ExtendedGraphletType {
    fn from(value: u32) -> Self {
        ExtendedGraphletType::from(value as u8)
    }
}

impl From<ReducedGraphletType> for u32 {
    fn from(value: ReducedGraphletType) -> Self {
        u8::from(value) as u32
    }
}

impl From<ExtendedGraphletType> for u32 {
    fn from(value: ExtendedGraphletType) -> Self {
        u8::from(value) as u32
    }
}

impl From<u64> for ReducedGraphletType {
    fn from(value: u64) -> Self {
        ReducedGraphletType::from(value as u8)
    }
}

impl From<u64> for ExtendedGraphletType {
    fn from(value: u64) -> Self {
        ExtendedGraphletType::from(value as u8)
    }
}

impl From<ReducedGraphletType> for u64 {
    fn from(value: ReducedGraphletType) -> Self {
        u8::from(value) as u64
    }
}

impl From<ExtendedGraphletType> for u64 {
    fn from(value: ExtendedGraphletType) -> Self {
        u8::from(value) as u64
    }
}

impl From<u128> for ReducedGraphletType {
    fn from(value: u128) -> Self {
        ReducedGraphletType::from(value as u8)
    }
}

impl From<u128> for ExtendedGraphletType {
    fn from(value: u128) -> Self {
        ExtendedGraphletType::from(value as u8)
    }
}

impl From<ReducedGraphletType> for u128 {
    fn from(value: ReducedGraphletType) -> Self {
        u8::from(value) as u128
    }
}

impl From<ExtendedGraphletType> for u128 {
    fn from(value: ExtendedGraphletType) -> Self {
        u8::from(value) as u128
    }
}

impl From<usize> for ReducedGraphletType {
    fn from(value: usize) -> Self {
        ReducedGraphletType::from(value as u8)
    }
}

impl From<usize> for ExtendedGraphletType {
    fn from(value: usize) -> Self {
        ExtendedGraphletType::from(value as u8)
    }
}

impl From<ReducedGraphletType> for usize {
    fn from(value: ReducedGraphletType) -> Self {
        u8::from(value) as usize
    }
}

impl From<ExtendedGraphletType> for usize {
    fn from(value: ExtendedGraphletType) -> Self {
        u8::from(value) as usize
    }
}