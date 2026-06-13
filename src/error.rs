//! Error types for the crate.

/// Errors that can occur when working with heterogeneous graphlets.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GraphletError {
    /// A numeric index does not correspond to any graphlet type.
    ///
    /// Returned by the `TryFrom` conversions for the graphlet type enums when
    /// the provided value is out of range.
    #[error("invalid graphlet type {value} (must be < {max})")]
    InvalidGraphletType {
        /// The out-of-range value that was provided.
        value: u8,
        /// The number of valid graphlet types (the exclusive upper bound).
        max: u8,
    },
}
