use crate::utils::{integer_power, NumericalConstants};
use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, Rem},
};

/// A trait for quadruple perfect hash functions.
pub trait PerfectHash<
    T: Mul<T, Output = T> + Add<T, Output = T> + PartialEq + Eq + Copy + NumericalConstants + Debug,
>: Sized
{
    const NUMBER_OF_GRAPHLETS: T = T::TWELVE;

    /// Returns the hash value associated to the provided quadruple and graphlet.
    ///
    /// # Arguments
    /// * `graphlet` - The graphlet type to encode with the quadruple itself.
    /// * `number_of_elements` - The number of elements in the graphlet.
    ///
    fn encode(&self, graphlet: T, number_of_elements: T) -> T;

    /// Returns the graphlet type and the quadruple associated to the provided hash value.
    ///
    /// # Arguments
    /// * `encoded` - The hash value whose quadruple should be computed.
    /// * `number_of_elements` - The number of elements in the graphlet.
    ///
    fn decode(encoded: T, number_of_elements: T) -> (T, Self);

    /// Returns the name of the graphlet type associated to the provided hash value.
    ///
    /// # Arguments
    /// * `encoded` - The hash value whose graphlet type should be computed.
    /// * `number_of_elements` - The number of elements in the graphlet.
    fn get_graphlet_type(encoded: T, number_of_elements: T) -> Result<&'static str, String> {
        let graphlet_type = Self::decode(encoded, number_of_elements).0;
        Ok(if graphlet_type == T::ONE {
            "triad (g1)"
        } else if graphlet_type == T::TWO {
            "triangle (g2)"
        } else if graphlet_type == T::THREE {
            "four-path (g3)"
        } else if graphlet_type == T::FOUR {
            "four-path center orbit (g4)"
        } else if graphlet_type == T::FIVE {
            "four-star orbit (g5)"
        } else if graphlet_type == T::SIX {
            "four-cycle (g6)"
        } else if graphlet_type == T::SEVEN {
            "tailed tri-tail orbit (g7)"
        } else if graphlet_type == T::EIGHT {
            "tailed tri-center orbit (g8)"
        } else if graphlet_type == T::NINE {
            "tailed tri-edge orbit (g9)"
        } else if graphlet_type == T::TEN {
            "chordal cycle edge orbit (g10)"
        } else if graphlet_type == T::ELEVEN {
            "chordal cycle center orbit (g11)"
        } else if graphlet_type == T::TWELVE {
            "four-clique (g12)"
        } else {
            return Err(format!(
                concat!(
                    "The provided graphlet type is not valid. ",
                    "The graphlet type should be in the range [1, {:?}]. ",
                    "You provided {:?}, as derived from hash {:?}."
                ),
                Self::NUMBER_OF_GRAPHLETS,
                graphlet_type,
                encoded
            ));
        })
    }

    /// Returns the maximal hash value that can be encoded.
    fn maximal_hash(number_of_elements: T) -> T {
        Self::NUMBER_OF_GRAPHLETS * integer_power::<4, T>(number_of_elements)
            + integer_power::<3, T>(number_of_elements)
            + integer_power::<2, T>(number_of_elements)
            + number_of_elements
    }
}

impl<
        T: Mul<T, Output = T>
            + Rem<T, Output = T>
            + Div<T, Output = T>
            + Add<T, Output = T>
            + PartialEq
            + Eq
            + NumericalConstants
            + Debug
            + Copy,
    > PerfectHash<T> for (T, T, T, T)
{
    #[inline(always)]
    fn encode(&self, graphlet: T, number_of_elements: T) -> T {
        graphlet * integer_power::<4, T>(number_of_elements)
            + self.0 * integer_power::<3, T>(number_of_elements)
            + self.1 * integer_power::<2, T>(number_of_elements)
            + self.2 * number_of_elements
            + self.3
    }

    #[inline(always)]
    fn decode(encoded: T, number_of_elements: T) -> (T, Self) {
        let graphlet = encoded / integer_power::<4, T>(number_of_elements);
        let encoded = encoded % integer_power::<4, T>(number_of_elements);
        let first = encoded / integer_power::<3, T>(number_of_elements);
        let encoded = encoded % integer_power::<3, T>(number_of_elements);
        let second = encoded / integer_power::<2, T>(number_of_elements);
        let encoded = encoded % integer_power::<2, T>(number_of_elements);
        let third = encoded / number_of_elements;
        let fourth = encoded % number_of_elements;
        (graphlet, (first, second, third, fourth))
    }
}
