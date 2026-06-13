use alloc::string::{String, ToString};
use core::{
    fmt::Debug,
    ops::{Add, AddAssign, Mul},
};
use hashbrown::HashMap;

use crate::{graphlet_set::GraphletSet, perfect_graphlet_hash::PerfectGraphletHash};
use num_traits::{AsPrimitive, One, Zero};

/// Trait defining characteristics of a set of graphlets.
///
/// Many implementations are possible for this trait depending
/// on the expected graph topologies.
pub trait GraphLetCounter<Graphlet, Count>
where
    Count: Debug + One,
    Graphlet: Debug + Copy + Mul<Output = Graphlet> + Add<Output = Graphlet>,
{
    /// Iterator over the stored graphlets and their counts.
    type Iter<'a>: Iterator<Item = (Graphlet, Count)> + 'a
    where
        Self: 'a,
        Count: 'a;

    /// Inserts the provided graphlet into the graphlet set.
    ///
    /// # Arguments
    /// * `graphlet` - The graphlet to insert into the graphlet set.
    fn insert(&mut self, graphlet: Graphlet) {
        self.insert_count(graphlet, Count::one());
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
    fn get_report<GraphletKind: GraphletSet<Graphlet> + ToString + From<Graphlet>, Element>(
        &self,
        number_of_elements: Element,
    ) -> String
    where
        Element: Add<Element, Output = Element>
            + Mul<Output = Element>
            + AsPrimitive<Graphlet>
            + Debug
            + Copy
            + Ord,
        Graphlet: From<GraphletKind>
            + Debug
            + Copy
            + 'static
            + Mul<Output = Graphlet>
            + Add<Output = Graphlet>,
        (Element, Element, Element, Element): PerfectGraphletHash<Graphlet, Element>,
    {
        use core::fmt::Write as _;
        let mut report = String::new();
        for (graphlet, count) in self.iter_graphlets_and_counts() {
            let graphlet_kind: GraphletKind =
                <(Element, Element, Element, Element)>::decode_graphlet_kind::<GraphletKind>(
                    graphlet,
                    number_of_elements,
                );
            let graphlet_name = graphlet_kind.to_string();
            // Writing to a String is infallible.
            let _ = writeln!(report, "{graphlet_name}: {count:?}");
        }
        report
    }

    /// Returns a map from graphlet names to their counts.
    fn to_graphlet_names<GraphletKind: GraphletSet<Graphlet> + ToString + From<Graphlet>, Element>(
        &self,
        number_of_elements: Element,
    ) -> HashMap<String, Count>
    where
        Element: Add<Element, Output = Element>
            + Mul<Output = Element>
            + AsPrimitive<Graphlet>
            + Debug
            + Copy
            + Ord,
        Graphlet: From<GraphletKind>
            + Debug
            + Copy
            + 'static
            + Mul<Output = Graphlet>
            + Add<Output = Graphlet>,
        (Element, Element, Element, Element): PerfectGraphletHash<Graphlet, Element>,
    {
        self.iter_graphlets_and_counts()
            .map(|(graphlet, count)| {
                (
                    <(Element, Element, Element, Element)>::decode_graphlet_kind::<GraphletKind>(
                        graphlet,
                        number_of_elements,
                    )
                    .to_string(),
                    count,
                )
            })
            .collect()
    }
}

impl<Graphlet, Count> GraphLetCounter<Graphlet, Count> for HashMap<Graphlet, Count>
where
    Count: Debug + Zero + One + Ord + AddAssign + Copy,
    Graphlet:
        Debug + Copy + Eq + core::hash::Hash + Mul<Output = Graphlet> + Add<Output = Graphlet>,
{
    type Iter<'a>
        = core::iter::Map<
        hashbrown::hash_map::Iter<'a, Graphlet, Count>,
        fn((&Graphlet, &Count)) -> (Graphlet, Count),
    >
    where
        Self: 'a;

    fn with_number_of_elements<Element>(_number_of_elements: Element) -> Self {
        Self::new()
    }

    fn insert_count(&mut self, graphlet: Graphlet, count: Count) {
        if count > Count::zero() {
            *self.entry(graphlet).or_insert_with(Count::zero) += count;
        }
    }

    fn get_number_of_graphlets(&self, graphlet: Graphlet) -> Count {
        self.get(&graphlet).copied().unwrap_or_else(Count::zero)
    }

    fn iter_graphlets_and_counts<'a>(&'a self) -> Self::Iter<'a>
    where
        Self: 'a,
        Count: 'a,
    {
        self.iter().map(|(graphlet, count)| (*graphlet, *count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphlet_set::ExtendedGraphletType;

    /// Encodes a graphlet of the given kind with all-zero labels for `n` labels.
    fn encode(kind: ExtendedGraphletType, n: u8) -> u32 {
        (0u8, 0, 0, 0).encode_with_graphlet::<ExtendedGraphletType>(kind, n)
    }

    #[test]
    fn insert_count_accumulates() {
        let mut counter: HashMap<u32, u32> = HashMap::new();
        counter.insert_count(10, 3);
        counter.insert_count(10, 2);
        assert_eq!(counter.get_number_of_graphlets(10), 5);
    }

    #[test]
    fn insert_count_skips_zero() {
        let mut counter: HashMap<u32, u32> = HashMap::new();
        counter.insert_count(10, 4);
        counter.insert_count(20, 0);
        assert_eq!(counter.get_number_of_graphlets(20), 0);
        // A zero count must not create an entry: only key 10 is stored.
        assert_eq!(counter.iter_graphlets_and_counts().count(), 1);
    }

    #[test]
    fn insert_adds_one() {
        let mut counter: HashMap<u32, u32> = HashMap::new();
        // Fully qualified to call the trait method, not HashMap::insert.
        GraphLetCounter::insert(&mut counter, 10);
        GraphLetCounter::insert(&mut counter, 10);
        assert_eq!(counter.get_number_of_graphlets(10), 2);
    }

    #[test]
    fn report_and_names_render_counts() {
        let n = 4u8;
        let mut counter: HashMap<u32, u32> = HashMap::new();
        counter.insert_count(encode(ExtendedGraphletType::Triangle, n), 7);

        let report = counter.get_report::<ExtendedGraphletType, u8>(n);
        assert!(report.contains("Triangle: 7"), "report was: {report:?}");

        let names = counter.to_graphlet_names::<ExtendedGraphletType, u8>(n);
        assert_eq!(names.get("Triangle"), Some(&7));
    }
}
