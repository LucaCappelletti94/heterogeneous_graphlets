//! Numeric helper traits not provided by `num-traits`.
//!
//! `num-traits` supplies `Zero`, `One`, `Bounded` and `AsPrimitive`, which
//! cover the rest of the crate's numeric needs. It has no equivalent for the
//! constant two, so we keep this tiny trait to avoid threading an extra `Add`
//! bound through every signature that only needs to divide by two.

/// A type that has a constant value of two.
pub trait Two {
    const TWO: Self;
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
