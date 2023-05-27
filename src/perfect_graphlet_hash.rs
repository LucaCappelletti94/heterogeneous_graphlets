use crate::{graphlet_set::GraphletSet, numbers::Primitive};
use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, Rem},
};

#[inline(always)]
/// Returns the exponentiation of the provided number with the const exponent.
fn integer_power<const EXPONENT: usize, T: Mul<T, Output = T> + Copy>(x: T) -> T {
    let mut result = x;
    for _ in 1..EXPONENT {
        result = result * x;
    }
    result
}

/// A trait for quadruple perfect hash functions.
pub trait PerfectGraphletHash<
    Graphlet: Debug + Copy + Primitive<Element> + Mul<Output = Graphlet> + Add<Output = Graphlet>,
    Element: Mul<Element, Output = Element>
        + Add<Element, Output = Element>
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

    /// Returns the graphlet type and object associated to the provided hash value.
    ///
    /// # Arguments
    /// * `encoded` - The hash value whose quadruple should be computed.
    /// * `number_of_elements` - The number of elements in the graphlet.
    ///
    fn decode_with_graphlet<GraphletKind: GraphletSet<Graphlet> + From<Graphlet>>(
        encoded: Graphlet,
        number_of_elements: Element,
    ) -> (GraphletKind, Self)
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

    /// Returns the maximal hash value that can be encoded.
    ///
    /// # Arguments
    /// * `number_of_elements` - The number of elements in the graphlet.
    ///
    fn maximal_hash<GraphletKind: GraphletSet<Graphlet> + From<Graphlet>>(
        number_of_elements: Element,
    ) -> Graphlet;
}

impl<
        Graphlet: Debug
            + Copy
            + Primitive<Element>
            + Div<Output = Graphlet>
            + Rem<Output = Graphlet>
            + Mul<Output = Graphlet>
            + Add<Output = Graphlet>,
        Element: Mul<Element, Output = Element>
            + Add<Element, Output = Element>
            + Primitive<Graphlet>
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
        let number_of_elements: Graphlet = Graphlet::convert(number_of_elements);
        let first: Graphlet = Graphlet::convert(self.0);
        let second: Graphlet = Graphlet::convert(self.1);
        let third: Graphlet = Graphlet::convert(self.2);
        let fourth: Graphlet = Graphlet::convert(self.3);
        graphlet_kind * integer_power::<4, Graphlet>(number_of_elements)
            + first * integer_power::<3, Graphlet>(number_of_elements)
            + second * integer_power::<2, Graphlet>(number_of_elements)
            + third * integer_power::<1, Graphlet>(number_of_elements)
            + fourth
    }

    #[inline(always)]
    fn decode_with_graphlet<GraphletKind: GraphletSet<Graphlet> + From<Graphlet>>(
        encoded: Graphlet,
        number_of_elements: Element,
    ) -> (GraphletKind, Self)
    where
        Graphlet: From<GraphletKind>,
    {
        let number_of_elements: Graphlet = Graphlet::convert(number_of_elements);
        let graphlet_kind: Graphlet = encoded / integer_power::<4, Graphlet>(number_of_elements);
        let encoded: Graphlet = encoded % integer_power::<4, Graphlet>(number_of_elements);
        let first: Graphlet = encoded / integer_power::<3, Graphlet>(number_of_elements);
        let encoded: Graphlet = encoded % integer_power::<3, Graphlet>(number_of_elements);
        let second: Graphlet = encoded / integer_power::<2, Graphlet>(number_of_elements);
        let encoded: Graphlet = encoded % integer_power::<2, Graphlet>(number_of_elements);
        let third: Graphlet = encoded / integer_power::<1, Graphlet>(number_of_elements);
        let encoded: Graphlet = encoded % integer_power::<1, Graphlet>(number_of_elements);
        let fourth: Graphlet = encoded;
        (
            graphlet_kind.into(),
            (
                Element::convert(first),
                Element::convert(second),
                Element::convert(third),
                Element::convert(fourth),
            ),
        )
    }

    #[inline(always)]
    fn decode_graphlet_kind<GraphletKind: GraphletSet<Graphlet> + From<Graphlet>>(
        encoded: Graphlet,
        number_of_elements: Element,
    ) -> GraphletKind {
        let number_of_elements: Graphlet = Graphlet::convert(number_of_elements);
        let graphlet_kind: Graphlet = encoded / integer_power::<4, Graphlet>(number_of_elements);
        graphlet_kind.into()
    }

    #[inline(always)]
    fn maximal_hash<GraphletKind: GraphletSet<Graphlet> + From<Graphlet>>(
        number_of_elements: Element,
    ) -> Graphlet {
        let number_of_graphlets: Graphlet = GraphletKind::get_number_of_graphlets().into();
        let number_of_elements: Graphlet = Graphlet::convert(number_of_elements);

        integer_power::<4, Graphlet>(number_of_elements) * number_of_graphlets
            + integer_power::<4, Graphlet>(number_of_elements)
            + integer_power::<3, Graphlet>(number_of_elements)
            + integer_power::<2, Graphlet>(number_of_elements)
            + number_of_elements
    }
}
