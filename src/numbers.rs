pub trait One {
    const ONE: Self;
}

pub trait Two {
    const TWO: Self;
}

pub trait Zero {
    const ZERO: Self;
}

impl One for u8 {
    const ONE: Self = 1;
}

impl One for u16 {
    const ONE: Self = 1;
}

impl One for u32 {
    const ONE: Self = 1;
}

impl One for u64 {
    const ONE: Self = 1;
}

impl One for usize {
    const ONE: Self = 1;
}

impl One for u128 {
    const ONE: Self = 1;
}

impl Two for u8 {
    const TWO: Self = 2;
}

impl Two for u16 {
    const TWO: Self = 2;
}

impl Two for u32 {
    const TWO: Self = 2;
}

impl Two for u64 {
    const TWO: Self = 2;
}

impl Two for usize {
    const TWO: Self = 2;
}

impl Two for u128 {
    const TWO: Self = 2;
}

impl Zero for u8 {
    const ZERO: Self = 0;
}

impl Zero for u16 {
    const ZERO: Self = 0;
}

impl Zero for u32 {
    const ZERO: Self = 0;
}

impl Zero for u64 {
    const ZERO: Self = 0;
}

impl Zero for usize {
    const ZERO: Self = 0;
}

impl Zero for u128 {
    const ZERO: Self = 0;
}

pub trait Primitive<Other> {
    fn convert(other: Other) -> Self;
}

impl Primitive<u8> for u8 {
    fn convert(other: u8) -> Self {
        other
    }
}

impl Primitive<u8> for u16 {
    fn convert(other: u8) -> Self {
        other as Self
    }
}

impl Primitive<u16> for u8 {
    fn convert(other: u16) -> Self {
        other as Self
    }
}

impl Primitive<u32> for u8 {
    fn convert(other: u32) -> Self {
        other as Self
    }
}

impl Primitive<u64> for u8 {
    fn convert(other: u64) -> Self {
        other as Self
    }
}

impl Primitive<usize> for u8 {
    fn convert(other: usize) -> Self {
        other as Self
    }
}

impl Primitive<u16> for u32 {
    fn convert(other: u16) -> Self {
        other as Self
    }
}

impl Primitive<u16> for u16 {
    fn convert(other: u16) -> Self {
        other as Self
    }
}

impl Primitive<u32> for u16 {
    fn convert(other: u32) -> Self {
        other as Self
    }
}

impl Primitive<u64> for u16 {
    fn convert(other: u64) -> Self {
        other as Self
    }
}

impl Primitive<usize> for u16 {
    fn convert(other: usize) -> Self {
        other as Self
    }
}

impl Primitive<u32> for u64 {
    fn convert(other: u32) -> Self {
        other as Self
    }
}

impl Primitive<u32> for u32 {
    fn convert(other: u32) -> Self {
        other as Self
    }
}

impl Primitive<u64> for u64 {
    fn convert(other: u64) -> Self {
        other as Self
    }
}

impl Primitive<u64> for u32 {
    fn convert(other: u64) -> Self {
        other as Self
    }
}

impl Primitive<usize> for u32 {
    fn convert(other: usize) -> Self {
        other as Self
    }
}

impl Primitive<u64> for usize {
    fn convert(other: u64) -> Self {
        other as Self
    }
}

impl Primitive<usize> for u64 {
    fn convert(other: usize) -> Self {
        other as Self
    }
}

impl Primitive<usize> for usize {
    fn convert(other: usize) -> Self {
        other as Self
    }
}

impl Primitive<u128> for u8 {
    fn convert(other: u128) -> Self {
        other as Self
    }
}

impl Primitive<u128> for u16 {
    fn convert(other: u128) -> Self {
        other as Self
    }
}

impl Primitive<u128> for u32 {
    fn convert(other: u128) -> Self {
        other as Self
    }
}

impl Primitive<u128> for u64 {
    fn convert(other: u128) -> Self {
        other as Self
    }
}

impl Primitive<u128> for usize {
    fn convert(other: u128) -> Self {
        other as Self
    }
}

impl Primitive<u128> for u128 {
    fn convert(other: u128) -> Self {
        other as Self
    }
}

impl Primitive<u16> for u128 {
    fn convert(other: u16) -> Self {
        other as Self
    }
}

impl Primitive<u32> for u128 {
    fn convert(other: u32) -> Self {
        other as Self
    }
}

impl Primitive<u64> for u128 {
    fn convert(other: u64) -> Self {
        other as Self
    }
}

impl Primitive<usize> for u128 {
    fn convert(other: usize) -> Self {
        other as Self
    }
}

impl Primitive<u8> for usize {
    fn convert(other: u8) -> Self {
        other as Self
    }
}

pub trait Maximal {
    const MAXIMAL: Self;
}

impl Maximal for u8 {
    const MAXIMAL: Self = u8::MAX;
}

impl Maximal for u16 {
    const MAXIMAL: Self = u16::MAX;
}

impl Maximal for u32 {
    const MAXIMAL: Self = u32::MAX;
}

impl Maximal for u64 {
    const MAXIMAL: Self = u64::MAX;
}

impl Maximal for usize {
    const MAXIMAL: Self = usize::MAX;
}

impl Maximal for u128 {
    const MAXIMAL: Self = u128::MAX;
}