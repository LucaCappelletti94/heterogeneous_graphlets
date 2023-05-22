/// A trait for quadruple perfect hash functions.
pub(crate) trait QuadruplePerfectHash<const N: usize> {
    /// Returns the hash value associated to the provided quadruple.
    fn encode_quadruple(&self) -> usize;

    /// Returns the quadruple associated to the provided hash value.
    /// 
    /// # Arguments
    /// * `encoded` - The hash value whose quadruple should be computed.
    fn decode_quadruple(encoded: usize) -> Self;

    /// Returns the maximal hash value that can be encoded.
    fn maximal_hash() -> usize {
        N * N * N * N + N * N * N + N * N + N
    }
}

impl<const N: usize> QuadruplePerfectHash<N> for (usize, usize, usize, usize) {
    fn encode_quadruple(&self) -> usize {
        let (a, b, c, d) = *self;
        a * N * N * N + b * N * N + c * N + d
    }

    fn decode_quadruple(encoded: usize) -> Self {
        let a = encoded / (N * N * N);
        let b = (encoded - a * N * N * N) / (N * N);
        let c = (encoded - a * N * N * N - b * N * N) / N;
        let d = encoded - a * N * N * N - b * N * N - c * N;
        (a, b, c, d)
    }
}