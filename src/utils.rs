/// Returns the binomial of the provided number of base two.
/// 
/// # Arguments
/// * `x` - The number whose binomial with two should be computed.
pub(crate) fn binomial_two(x: usize) -> usize {
    x * (x - 1) / 2
}
