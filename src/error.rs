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
    /// The chosen `Graphlet` key integer type is too small to hold every
    /// graphlet hash for the graph's node-label count, so counting in it would
    /// overflow. Choose a wider `Graphlet` type (see the crate documentation for
    /// the per-type capacities).
    #[error(
        "the maximal graphlet hash ({maximal_hash}) for {number_of_node_labels} node labels \
         exceeds the maximum value ({maximal_graphlet}) of the chosen graphlet key type"
    )]
    GraphletKeyTooSmall {
        /// The number of node labels reported by the graph.
        number_of_node_labels: u128,
        /// The largest hash any graphlet could encode to for that label count.
        maximal_hash: u128,
        /// The maximum value representable by the chosen graphlet key type.
        maximal_graphlet: u128,
    },
}
