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
        // The real node labels span `0..number_of_elements`, and 3-node graphlets store
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

/// Canonicalises a positional edge-coloured graphlet descriptor under the four
/// ways of assigning the abstract positions to the actual nodes: the focal
/// endpoints `(i, j)` are interchangeable (the focal edge is undirected) and the
/// two non-focal nodes `(x3, x4)` are interchangeable. The descriptor is the four
/// node labels in positions `(i, j, x3, x4)` and the six edge colours in slots
/// `g0=(i,j)`, `g1=(i,x3)`, `g2=(i,x4)`, `g3=(j,x3)`, `g4=(j,x4)`, `g5=(x3,x4)`.
/// Absent nodes and edges carry the largest digit (the sentinel), so the
/// lexicographically minimal descriptor keeps real nodes and present edges in the
/// earliest positions. Both the fast path and the differential oracle key through
/// this same function, so isomorphic typed graphlets collapse to one key. The two
/// generators (swap focal endpoints, swap non-focal nodes) commute and are
/// involutions, so these four assignments are the whole group.
#[must_use]
pub(crate) fn canonical_descriptor<NodeLabel: Copy + Ord, EdgeLabel: Copy + Ord>(
    nodes: [NodeLabel; 4],
    edges: [EdgeLabel; 6],
) -> ([NodeLabel; 4], [EdgeLabel; 6]) {
    let [c0, c1, c2, c3] =
        [(false, false), (false, true), (true, false), (true, true)].map(|(swap_ij, swap_xy)| {
            let mut n = nodes;
            let mut g = edges;
            if swap_ij {
                n.swap(0, 1);
                g.swap(1, 3); // (i,x3) <-> (j,x3)
                g.swap(2, 4); // (i,x4) <-> (j,x4)
            }
            if swap_xy {
                n.swap(2, 3);
                g.swap(1, 2); // (i,x3) <-> (i,x4)
                g.swap(3, 4); // (j,x3) <-> (j,x4)
            }
            (n, g)
        });
    // The lexicographically minimal of the four position assignments (Ord on the
    // tuple), chained pairwise to avoid an `Option` from an iterator `min`.
    c0.min(c1).min(c2).min(c3)
}

/// Encodes a canonical edge-coloured graphlet descriptor into a perfect-hash key.
/// The node base is `number_of_node_labels + 1` and the edge base is
/// `number_of_edge_labels + 1` (each reserves a sentinel digit). The kind and the
/// four node digits use the node base (the same sub-layout as the node-only hash),
/// and the six edge digits use the edge base. The node-only key is recovered by
/// dividing the result by the edge base to the sixth power.
#[must_use]
pub(crate) fn encode_edge_typed<Graphlet, NodeLabel, EdgeLabel>(
    graphlet_kind: Graphlet,
    nodes: [NodeLabel; 4],
    edges: [EdgeLabel; 6],
    node_base: Graphlet,
    edge_base: Graphlet,
) -> Graphlet
where
    Graphlet: Copy + One + 'static + Mul<Output = Graphlet> + Add<Output = Graphlet>,
    NodeLabel: AsPrimitive<Graphlet>,
    EdgeLabel: AsPrimitive<Graphlet>,
{
    let node_key = graphlet_kind * pow(node_base, 4)
        + nodes[0].as_() * pow(node_base, 3)
        + nodes[1].as_() * pow(node_base, 2)
        + nodes[2].as_() * node_base
        + nodes[3].as_();
    let edge_key = edges[0].as_() * pow(edge_base, 5)
        + edges[1].as_() * pow(edge_base, 4)
        + edges[2].as_() * pow(edge_base, 3)
        + edges[3].as_() * pow(edge_base, 2)
        + edges[4].as_() * edge_base
        + edges[5].as_();
    node_key * pow(edge_base, 6) + edge_key
}

/// Decodes a key produced by [`encode_edge_typed`] back into its kind, four node
/// digits and six edge digits, all as `u128`. The inverse of [`encode_edge_typed`].
///
/// The whole-graph deduplication path (and the differential oracle) decode keys to
/// recover and recanonicalise the descriptor.
#[must_use]
pub(crate) fn decode_edge_typed<Graphlet: AsPrimitive<u128> + 'static>(
    encoded: Graphlet,
    node_base: Graphlet,
    edge_base: Graphlet,
) -> (u128, [u128; 4], [u128; 6]) {
    let b = node_base.as_();
    let e = edge_base.as_();
    let key = encoded.as_();
    let mut edge_key = key % e.pow(6);
    let mut edges = [0u128; 6];
    for slot in (0..6).rev() {
        edges[slot] = edge_key % e;
        edge_key /= e;
    }
    let mut node_key = key / e.pow(6);
    let mut nodes = [0u128; 4];
    for slot in (0..4).rev() {
        nodes[slot] = node_key % b;
        node_key /= b;
    }
    (node_key, nodes, edges)
}

/// Edge slot (`0..6`) of the edge between node positions `a` and `b` (`a != b`), in
/// the fixed layout `g0=(0,1), g1=(0,2), g2=(0,3), g3=(1,2), g4=(1,3), g5=(2,3)`.
/// Symmetric in `a` and `b`; the diagonal is unused (`usize::MAX`).
const EDGE_SLOT: [[usize; 4]; 4] = [
    [usize::MAX, 0, 1, 2],
    [0, usize::MAX, 3, 4],
    [1, 3, usize::MAX, 5],
    [2, 4, 5, usize::MAX],
];

/// The 24 permutations of the four node positions, in lexicographic order. A test
/// pins that this is exactly the symmetric group on `{0,1,2,3}`.
const PERMS_S4: [[usize; 4]; 24] = [
    [0, 1, 2, 3],
    [0, 1, 3, 2],
    [0, 2, 1, 3],
    [0, 2, 3, 1],
    [0, 3, 1, 2],
    [0, 3, 2, 1],
    [1, 0, 2, 3],
    [1, 0, 3, 2],
    [1, 2, 0, 3],
    [1, 2, 3, 0],
    [1, 3, 0, 2],
    [1, 3, 2, 0],
    [2, 0, 1, 3],
    [2, 0, 3, 1],
    [2, 1, 0, 3],
    [2, 1, 3, 0],
    [2, 3, 0, 1],
    [2, 3, 1, 0],
    [3, 0, 1, 2],
    [3, 0, 2, 1],
    [3, 1, 0, 2],
    [3, 1, 2, 0],
    [3, 2, 0, 1],
    [3, 2, 1, 0],
];

/// Canonicalises a positional graphlet descriptor under the FULL automorphism
/// group of the 4-node graphlet, by minimising `(nodes, edges)` over all 24
/// relabellings of the four node positions (each node permutation inducing the
/// matching permutation of the six edge slots). Where [`canonical_descriptor`]
/// quotients only the focal-edge-fixing subgroup, so that per-edge counting still
/// tallies each occurrence once per edge, this widens to the whole of `S4`: any
/// relabelling of a graphlet is an isomorphic graph, so the lexicographic minimum
/// is the canonical representative of the isomorphism class and the stabiliser of
/// that minimum is the automorphism group (never enumerated explicitly). Two
/// per-edge keys produced from different focal-edge rootings of the same coloured
/// occurrence are two labellings of one abstract graph and collapse to the same
/// result; two genuinely different coloured patterns never collide. This is the
/// fold that turns an over-counted whole-graph signature into exact occurrence
/// counts (after dividing by the graphlet's edge count).
#[must_use]
#[allow(clippy::needless_range_loop)]
pub(crate) fn full_canonical_descriptor<NodeLabel: Copy + Ord, EdgeLabel: Copy + Ord>(
    nodes: [NodeLabel; 4],
    edges: [EdgeLabel; 6],
) -> ([NodeLabel; 4], [EdgeLabel; 6]) {
    // Start from the identity relabelling (the first entry of PERMS_S4) and fold
    // the minimum over all 24, so no `Option`/`unwrap` is needed.
    let mut best = (nodes, edges);
    for p in PERMS_S4 {
        let permuted_nodes = [nodes[p[0]], nodes[p[1]], nodes[p[2]], nodes[p[3]]];
        let mut permuted_edges = edges;
        for a in 0..4 {
            for b in (a + 1)..4 {
                permuted_edges[EDGE_SLOT[a][b]] = edges[EDGE_SLOT[p[a]][p[b]]];
            }
        }
        best = core::cmp::min(best, (permuted_nodes, permuted_edges));
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphlet_set::ExtendedGraphletType;
    use proptest::prelude::*;

    proptest! {
        /// Property 10 (key well-formedness): the edge-coloured hash round-trips
        /// (decode inverts encode) and collapses correctly (dividing by
        /// `edge_base^6` recovers the node-only key). Node digits range over
        /// `0..=c` and edge digits over `0..=d`, so both sentinel values are
        /// exercised.
        #[test]
        fn edge_typed_hash_roundtrips_and_collapses(
            c in 1u64..=4,
            d in 1u64..=4,
            kind in 0u64..12,
            raw_nodes in proptest::array::uniform4(0u64..=4),
            raw_edges in proptest::array::uniform6(0u64..=4),
        ) {
            let node_base = c + 1;
            let edge_base = d + 1;
            // Reduce into the valid digit ranges (real values plus the sentinel).
            let nodes = raw_nodes.map(|x| x % node_base);
            let edges = raw_edges.map(|x| x % edge_base);

            let key = encode_edge_typed::<u64, u64, u64>(kind, nodes, edges, node_base, edge_base);

            // Collapse: dropping the six edge digits recovers the node-only key.
            let node_only = kind * node_base.pow(4)
                + nodes[0] * node_base.pow(3)
                + nodes[1] * node_base.pow(2)
                + nodes[2] * node_base
                + nodes[3];
            prop_assert_eq!(key / edge_base.pow(6), node_only);

            // Round-trip: decode recovers every digit.
            let (decoded_kind, decoded_nodes, decoded_edges) =
                decode_edge_typed::<u64>(key, node_base, edge_base);
            prop_assert_eq!(decoded_kind, u128::from(kind));
            prop_assert_eq!(decoded_nodes, nodes.map(u128::from));
            prop_assert_eq!(decoded_edges, edges.map(u128::from));
        }
    }

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
        // FourClique has index 11, and with 10 labels the base is 11, so the encoding
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
        // (`= n`) in the 4th slot, while the ten 4-node kinds carry four real labels in
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

    /// Relabels a descriptor by permutation `p` (new position `k` takes the value
    /// from old position `p[k]`), the same action `full_canonical_descriptor`
    /// minimises over.
    fn relabel(nodes: [u64; 4], edges: [u64; 6], p: [usize; 4]) -> ([u64; 4], [u64; 6]) {
        let n = [nodes[p[0]], nodes[p[1]], nodes[p[2]], nodes[p[3]]];
        let mut e = edges;
        for a in 0..4 {
            for b in (a + 1)..4 {
                e[EDGE_SLOT[a][b]] = edges[EDGE_SLOT[p[a]][p[b]]];
            }
        }
        (n, e)
    }

    #[test]
    fn perms_s4_is_exactly_the_symmetric_group() {
        use hashbrown::HashSet;
        let set: HashSet<[usize; 4]> = PERMS_S4.iter().copied().collect();
        assert_eq!(set.len(), 24, "PERMS_S4 must hold 24 distinct permutations");
        for p in PERMS_S4 {
            let mut seen = [false; 4];
            for &k in &p {
                assert!(k < 4 && !seen[k], "not a permutation of 0..4: {p:?}");
                seen[k] = true;
            }
        }
    }

    #[test]
    fn full_canonical_is_relabelling_invariant_and_idempotent() {
        // An arbitrary coloured 4-node graphlet with a mix of present (0,1,2) and
        // absent (sentinel 3) bond colours.
        let nodes = [2u64, 0, 1, 0];
        let edges = [1u64, 0, 3, 2, 3, 3];
        let canon = full_canonical_descriptor(nodes, edges);
        // Idempotence: canonicalising the canonical form is a no-op.
        assert_eq!(full_canonical_descriptor(canon.0, canon.1), canon);
        // Invariance: every one of the 24 relabellings has the same canonical form.
        for p in PERMS_S4 {
            let (n, e) = relabel(nodes, edges, p);
            assert_eq!(full_canonical_descriptor(n, e), canon);
        }
    }

    #[test]
    fn full_canonical_collapses_four_path_rootings() {
        // The same coloured path with node colours 0-1-2-3, encoded as the crate
        // would when rooted on an end edge versus the centre edge. Bond colour 0 is
        // present, sentinel 9 marks an absent edge.
        let s = 9u64;
        // End rooting (FourPathEdge): nodes [0,1,2,3]; edges (0,1)=g0, (1,2)=g3, (2,3)=g5.
        let edge_rep = ([0u64, 1, 2, 3], [0, s, s, 0, s, 0]);
        // Centre rooting (FourPathCenter): nodes [1,2,0,3]; focal (1,2)=g0, (1,0)=g1, (2,3)=g4.
        let center_rep = ([1u64, 2, 0, 3], [0, 0, s, s, 0, s]);
        assert_eq!(
            full_canonical_descriptor(edge_rep.0, edge_rep.1),
            full_canonical_descriptor(center_rep.0, center_rep.1),
        );
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
