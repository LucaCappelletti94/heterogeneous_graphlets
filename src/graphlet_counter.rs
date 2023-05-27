use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Add, AddAssign, Mul},
};

use crate::{
    graphlet_set::GraphletSet,
    numbers::{One, Primitive, Zero},
    perfect_graphlet_hash::*,
};

/// Trait defining characteristics of a set of graphlets.
///
/// Many implementations are possible for this trait depending
/// on the expected graph topologies.
pub trait GraphLetCounter<Graphlet, Count>
where
    Count: Debug + One,
    Graphlet: Debug + Copy + Mul<Output = Graphlet> + Add<Output = Graphlet>,
{
    type Iter<'a>: Iterator<Item = (Graphlet, Count)> + 'a
    where
        Self: 'a,
        Count: 'a;

    /// Inserts the provided graphlet into the graphlet set.
    ///
    /// # Arguments
    /// * `graphlet` - The graphlet to insert into the graphlet set.
    fn insert(&mut self, graphlet: Graphlet) {
        self.insert_count(graphlet, Count::ONE);
    }

    /// Inserts the provided graphlet into the graphlet set.
    ///
    /// # Arguments
    /// * `graphlet` - The graphlet to insert into the graphlet set.
    /// * `count` - The number of times the graphlet should be inserted.
    fn insert_count(&mut self, graphlet: Graphlet, count: Count);

    /// Returns the number of graphlets of the provided type.
    ///
    /// # Arguments
    /// * `graphlet` - The graphlet whose number of occurrences should be returned.
    fn get_number_of_graphlets(&self, graphlet: Graphlet) -> Count;

    /// Iterate over the graphlets and their counts.
    fn iter_graphlets_and_counts<'a>(&'a self) -> Self::Iter<'a>
    where
        Self: 'a,
        Count: 'a;

    /// Create new counter object with given number of elements.
    ///
    /// # Arguments
    /// * `number_of_elements` - The number of elements, i.e. the node labels, in the graph.
    ///
    fn with_number_of_elements<Element>(number_of_elements: Element) -> Self;

    /// Returns extensive report describing the graphlet set.
    fn get_report<GraphletKind: GraphletSet<Graphlet> + ToString, Element>(
        &self,
        number_of_elements: Element,
    ) -> Result<String, String>
    where
        Element: Add<Element, Output = Element>
            + Mul<Output = Element>
            + Debug
            + Copy
            + One
            + Zero
            + Ord,

        Graphlet: From<GraphletKind> + Primitive<Element>,
        (Element, Element, Element, Element): PerfectGraphletHash<Graphlet, GraphletKind, Element>,
    {
        let mut report = String::new();
        for (graphlet, count) in self.iter_graphlets_and_counts() {
            let graphlet_kind: GraphletKind =
                <(Element, Element, Element, Element)>::decode_graphlet_kind(
                    graphlet,
                    number_of_elements,
                );
            let graphlet_name = graphlet_kind.to_string();
            report.push_str(&format!("{}: {:?}\n", graphlet_name, count));
        }
        Ok(report)
    }
}

impl<Graphlet, Count> GraphLetCounter<Graphlet, Count>
    for HashMap<Graphlet, Count>
where
    Count: Debug + Zero + One + Ord + AddAssign + Copy,
    Graphlet: Debug
        + Copy
        + Eq
        + std::hash::Hash
        + Mul<Output = Graphlet>
        + Add<Output = Graphlet>,
{
    type Iter<'a> = std::iter::Map<std::collections::hash_map::Iter<'a, Graphlet, Count>, fn((&Graphlet, &Count)) -> (Graphlet, Count)> where Self: 'a;

    fn with_number_of_elements<Element>(_number_of_elements: Element) -> Self {
        HashMap::new()
    }

    fn insert_count(&mut self, graphlet: Graphlet, count: Count) {
        if count > Count::ZERO {
            *self.entry(graphlet).or_insert(Count::ZERO) += count;
        }
    }

    fn get_number_of_graphlets(&self, graphlet: Graphlet) -> Count {
        *self.get(&graphlet).unwrap_or(&Count::ZERO)
    }

    fn iter_graphlets_and_counts<'a>(&'a self) -> Self::Iter<'a>
    where
        Self: 'a,
        Count: 'a,
    {
        self.iter().map(|(graphlet, count)| (*graphlet, *count))
    }
}
