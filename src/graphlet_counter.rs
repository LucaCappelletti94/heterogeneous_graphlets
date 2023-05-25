use std::{
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    ops::{Add, Div, Mul, Rem},
};

use crate::{perfect_hash::PerfectHash, utils::NumericalConstants};

/// Trait defining characteristics of a set of graphlets.
///
/// Many implementations are possible for this trait depending
/// on the expected graph topologies.
pub trait GraphLetCounter<T>: Default
where
    T: Mul<T, Output = T>
        + Add<T, Output = T>
        + Div<T, Output = T>
        + PartialEq
        + Eq
        + Copy
        + NumericalConstants
        + Debug
        + Rem<T, Output = T>,
{
    type Iter<'a>: Iterator<Item = (T, usize)> + 'a
    where
        Self: 'a;

    /// Inserts the provided graphlet into the graphlet set.
    ///
    /// # Arguments
    /// * `graphlet` - The graphlet to insert into the graphlet set.
    fn insert(&mut self, graphlet: T);

    /// Inserts the provided graphlet into the graphlet set.
    ///
    /// # Arguments
    /// * `graphlet` - The graphlet to insert into the graphlet set.
    /// * `count` - The number of times the graphlet should be inserted.
    fn insert_count(&mut self, graphlet: T, count: usize);

    /// Returns the number of graphlets of the provided type.
    ///
    /// # Arguments
    /// * `graphlet` - The graphlet whose number of occurrences should be returned.
    fn get_number_of_graphlets(&self, graphlet: T) -> usize;

    /// Iterate over the graphlets and their counts.
    fn iter_graphlets_and_counts<'a>(&'a self) -> Self::Iter<'a>;

    /// Returns extensive report describing the graphlet set.
    fn get_report(&self, number_of_elements: T) -> Result<String, String> {
        let mut report = String::new();
        for (graphlet, count) in self.iter_graphlets_and_counts() {
            let graphlet_name =
                <(T, T, T, T) as PerfectHash<T>>::get_graphlet_type(graphlet, number_of_elements)?;
            report.push_str(&format!("{}: {}\n", graphlet_name, count));
        }
        Ok(report)
    }
}

impl<T> GraphLetCounter<T> for HashMap<T, usize>
where
    T: Mul<T, Output = T>
        + Add<T, Output = T>
        + Div<T, Output = T>
        + PartialEq
        + Eq
        + Hash
        + Copy
        + NumericalConstants
        + Debug
        + Rem<T, Output = T>,
{
    type Iter<'a> = std::iter::Map<std::collections::hash_map::Iter<'a, T, usize>, fn((&T, &usize)) -> (T, usize)> where Self: 'a, T: 'a;

    fn insert(&mut self, graphlet: T) {
        self.insert_count(graphlet, 1);
    }

    fn insert_count(&mut self, graphlet: T, count: usize) {
        if count > 0 {
            *self.entry(graphlet).or_insert(0) += count;
        }
    }

    fn get_number_of_graphlets(&self, graphlet: T) -> usize {
        *self.get(&graphlet).unwrap_or(&0)
    }

    fn iter_graphlets_and_counts<'a>(&'a self) -> Self::Iter<'a> {
        self.iter().map(|(graphlet, count)| (*graphlet, *count))
    }
}
