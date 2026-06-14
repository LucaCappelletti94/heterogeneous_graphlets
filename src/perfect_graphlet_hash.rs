//! Perfect hashing of a typed graphlet (a graphlet kind plus its four node
//! labels) into a single integer, and the inverse decode.
//!
//! # Examples
//! ```
//! use heterogeneous_graphlets::perfect_graphlet_hash::PerfectGraphletHash;
//! use heterogeneous_graphlets::prelude::ExtendedGraphletType;
//!
//! // Encode a 4-clique whose four nodes carry the labels (0, 1, 2, 3), for a
//! // graph with 4 node labels, into a `u32` key, then recover its kind.
//! let key: u32 = (0u8, 1, 2, 3)
//!     .encode_with_graphlet::<ExtendedGraphletType>(ExtendedGraphletType::FourClique, 4);
//! let kind =
//!     <(u8, u8, u8, u8)>::decode_graphlet_kind::<ExtendedGraphletType>(key, 4);
//! assert_eq!(kind, ExtendedGraphletType::FourClique);
//! ```

use crate::graphlet_set::GraphletSet;
use core::{
    fmt::Debug,
    ops::{Add, Div, Mul, Rem},
};
use num_traits::{pow, AsPrimitive, One};

/// A trait for quadruple perfect hash functions.
pub trait PerfectGraphletHash<
    Graphlet: Debug + Copy + 'static + Mul<Output = Graphlet> + Add<Output = Graphlet>,
    Element: Mul<Element, Output = Element>
        + Add<Element, Output = Element>
        + AsPrimitive<Graphlet>
        + PartialEq
        + Eq
        + Copy
        + Debug
        + Ord,
>: Sized
{
    /// Returns the hash value associated to self and graphlet.
    ///
    /// # Arguments
    /// * `graphlet` - The graphlet type to encode.
    /// * `number_of_elements` - The number of elements in the graphlet.
    ///
    fn encode_with_graphlet<GraphletKind: GraphletSet<Graphlet> + From<Graphlet>>(
        &self,
        graphlet_kind: GraphletKind,
        number_of_elements: Element,
    ) -> Graphlet
    where
        Graphlet: From<GraphletKind>;

    /// Returns the graphlet type associated to the provided hash value.
    ///
    /// # Arguments
    /// * `encoded` - The hash value whose quadruple should be computed.
    /// * `number_of_elements` - The number of elements in the graphlet.
    fn decode_graphlet_kind<GraphletKind: GraphletSet<Graphlet> + From<Graphlet>>(
        encoded: Graphlet,
        number_of_elements: Element,
    ) -> GraphletKind;
}

impl<
        Graphlet: Debug
            + Copy
            + 'static
            + One
            + AsPrimitive<Element>
            + Div<Output = Graphlet>
            + Rem<Output = Graphlet>
            + Mul<Output = Graphlet>
            + Add<Output = Graphlet>,
        Element: Mul<Element, Output = Element>
            + Add<Element, Output = Element>
            + AsPrimitive<Graphlet>
            + PartialEq
            + Eq
            + Copy
            + Debug
            + Ord,
    > PerfectGraphletHash<Graphlet, Element> for (Element, Element, Element, Element)
{
    #[inline(always)]
    fn encode_with_graphlet<GraphletKind: GraphletSet<Graphlet> + From<Graphlet>>(
        &self,
        graphlet_kind: GraphletKind,
        number_of_elements: Element,
    ) -> Graphlet
    where
        Graphlet: From<GraphletKind>,
    {
        let graphlet_kind: Graphlet = graphlet_kind.into();
        // The positional base is `number_of_elements + 1`, not `number_of_elements`.
        // The real node labels span `0..number_of_elements`; 3-node graphlets store
        // a sentinel 4th digit equal to `number_of_elements` to mark the absent
        // node. Using the label count itself as the base would make that sentinel
        // equal to the base, so it carries into higher positions and an
        // all-maximum-label 3-node graphlet aliases another kind (for example a
        // maximum-label triangle collides with an all-zero four-path-edge).
        // Reserving one extra digit value keeps the hash injective.
        let base: Graphlet = number_of_elements.as_() + Graphlet::one();
        let first: Graphlet = self.0.as_();
        let second: Graphlet = self.1.as_();
        let third: Graphlet = self.2.as_();
        let fourth: Graphlet = self.3.as_();
        graphlet_kind * pow(base, 4)
            + first * pow(base, 3)
            + second * pow(base, 2)
            + third * base
            + fourth
    }

    #[inline(always)]
    fn decode_graphlet_kind<GraphletKind: GraphletSet<Graphlet> + From<Graphlet>>(
        encoded: Graphlet,
        number_of_elements: Element,
    ) -> GraphletKind {
        // Mirror of `encode_with_graphlet`: the positional base is
        // `number_of_elements + 1` (one digit value is reserved as the 3-node
        // sentinel), so the kind occupies the `base^4` place.
        let base: Graphlet = number_of_elements.as_() + Graphlet::one();
        let graphlet_kind: Graphlet = encoded / pow(base, 4);
        graphlet_kind.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphlet_set::ExtendedGraphletType;
    use proptest::prelude::*;

    #[test]
    fn encode_is_positional_base_n_plus_one() {
        // The base is `number_of_elements + 1`. With 10 labels the base is 11, so
        // kind*11^4 + a*11^3 + b*11^2 + c*11 + d, with kind index 1 (Triangle):
        // 1*14641 + 2*1331 + 3*121 + 4*11 + 5 = 17715.
        let encoded: u32 = (2u8, 3, 4, 5)
            .encode_with_graphlet::<ExtendedGraphletType>(ExtendedGraphletType::Triangle, 10u8);
        assert_eq!(encoded, 17715);
    }

    #[test]
    fn decode_kind_recovers_the_top_digit() {
        // FourClique has index 11; with 10 labels the base is 11, so the encoding
        // is 11*11^4 + 2*11^3 + 3*11^2 + 4*11 + 5 = 164125, and decoding the top
        // base-11 digit must round-trip to the kind.
        let encoded: u32 = (2u8, 3, 4, 5)
            .encode_with_graphlet::<ExtendedGraphletType>(ExtendedGraphletType::FourClique, 10u8);
        assert_eq!(encoded, 164_125);
        let kind = <(u8, u8, u8, u8)>::decode_graphlet_kind::<ExtendedGraphletType>(encoded, 10u8);
        assert_eq!(kind, ExtendedGraphletType::FourClique);
    }

    #[test]
    fn perfect_hash_is_injective_over_all_emitted_graphlets() {
        use alloc::vec::Vec;
        use hashbrown::HashSet;
        // For every label count `n`, every typed graphlet the crate can emit must
        // hash to a distinct value and decode back to its kind. The two 3-node
        // kinds (Triad, Triangle) carry three real labels plus the sentinel
        // (`= n`) in the 4th slot; the ten 4-node kinds carry four real labels in
        // `0..n`. This exhaustively verifies the injectivity the base-(n+1)
        // encoding guarantees, a regression guard for the sentinel-carry collision.
        for n in 1u8..=8 {
            let mut seen = HashSet::new();
            for kind_index in 0u8..12 {
                let kind = ExtendedGraphletType::from(kind_index);
                let fourth: Vec<u8> = if kind_index <= 1 {
                    alloc::vec![n]
                } else {
                    (0..n).collect()
                };
                for a in 0..n {
                    for b in 0..n {
                        for c in 0..n {
                            for &d in &fourth {
                                let hash: u32 = (a, b, c, d)
                                    .encode_with_graphlet::<ExtendedGraphletType>(kind, n);
                                assert!(
                                    seen.insert(hash),
                                    "collision at n={n} kind={kind:?} ({a},{b},{c},{d}) -> {hash}"
                                );
                                assert_eq!(
                                    <(u8, u8, u8, u8)>::decode_graphlet_kind::<ExtendedGraphletType>(
                                        hash, n
                                    ),
                                    kind
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    proptest! {
        /// Encoding any typed graphlet then decoding recovers its kind, across
        /// label counts and labels well beyond the exhaustive range above.
        #[test]
        fn decode_kind_roundtrips(
            (n, kind_index, labels) in (1u8..=24u8).prop_flat_map(|n| {
                (Just(n), 0u8..12u8, proptest::array::uniform4(0u8..=n))
            })
        ) {
            let kind = ExtendedGraphletType::from(kind_index);
            let [a, b, c, d] = labels;
            let hash: u32 = (a, b, c, d).encode_with_graphlet::<ExtendedGraphletType>(kind, n);
            prop_assert_eq!(
                <(u8, u8, u8, u8)>::decode_graphlet_kind::<ExtendedGraphletType>(hash, n),
                kind
            );
        }
    }
}
