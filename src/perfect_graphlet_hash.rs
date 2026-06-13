//! Perfect hashing of a typed graphlet (a graphlet kind plus its four node
//! labels) into a single integer, and the inverse decode.

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
        let number_of_elements: Graphlet = number_of_elements.as_();
        let first: Graphlet = self.0.as_();
        let second: Graphlet = self.1.as_();
        let third: Graphlet = self.2.as_();
        let fourth: Graphlet = self.3.as_();
        graphlet_kind * pow(number_of_elements, 4)
            + first * pow(number_of_elements, 3)
            + second * pow(number_of_elements, 2)
            + third * number_of_elements
            + fourth
    }

    #[inline(always)]
    fn decode_graphlet_kind<GraphletKind: GraphletSet<Graphlet> + From<Graphlet>>(
        encoded: Graphlet,
        number_of_elements: Element,
    ) -> GraphletKind {
        let number_of_elements: Graphlet = number_of_elements.as_();
        let graphlet_kind: Graphlet = encoded / pow(number_of_elements, 4);
        graphlet_kind.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphlet_set::ExtendedGraphletType;

    #[test]
    fn encode_is_positional_base_n() {
        // kind*n^4 + a*n^3 + b*n^2 + c*n + d, with n = 10 and kind index 1:
        // 1*10000 + 2*1000 + 3*100 + 4*10 + 5 = 12345
        let encoded: u32 = (2u8, 3, 4, 5)
            .encode_with_graphlet::<ExtendedGraphletType>(ExtendedGraphletType::Triangle, 10u8);
        assert_eq!(encoded, 12345);
    }

    #[test]
    fn decode_kind_recovers_the_top_digit() {
        // FourClique has index 11, so encoding then decoding must round-trip.
        let encoded: u32 = (2u8, 3, 4, 5)
            .encode_with_graphlet::<ExtendedGraphletType>(ExtendedGraphletType::FourClique, 10u8);
        assert_eq!(encoded, 112_345);
        let kind =
            <(u8, u8, u8, u8)>::decode_graphlet_kind::<ExtendedGraphletType>(encoded, 10u8);
        assert_eq!(kind, ExtendedGraphletType::FourClique);
    }
}
