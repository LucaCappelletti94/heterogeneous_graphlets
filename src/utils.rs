use std::ops::Mul;

#[inline(always)]
/// Returns the binomial of the provided number of base two.
///
/// # Arguments
/// * `x` - The number whose binomial with two should be computed.
pub(crate) fn binomial_two(x: usize) -> usize {
    x * x.saturating_sub(1) / 2
}

#[inline(always)]
/// Returns the exponentiation of the provided number with the const exponent.
pub(crate) fn integer_power<const EXPONENT: usize, T: Mul<T, Output = T> + Copy>(x: T) -> T {
    let mut result = x;
    for _ in 1..EXPONENT {
        result = result * x;
    }
    result
}

pub trait NumericalConstants {
    const TWELVE: Self;
    const ELEVEN: Self;
    const TEN: Self;
    const NINE: Self;
    const EIGHT: Self;
    const SEVEN: Self;
    const SIX: Self;
    const FIVE: Self;
    const FOUR: Self;
    const THREE: Self;
    const TWO: Self;
    const ONE: Self;
}

impl NumericalConstants for usize {
    const TWELVE: Self = 12;
    const ELEVEN: Self = 11;
    const TEN: Self = 10;
    const NINE: Self = 9;
    const EIGHT: Self = 8;
    const SEVEN: Self = 7;
    const SIX: Self = 6;
    const FIVE: Self = 5;
    const FOUR: Self = 4;
    const THREE: Self = 3;
    const TWO: Self = 2;
    const ONE: Self = 1;
}
