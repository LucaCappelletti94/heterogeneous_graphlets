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
