#![feature(return_position_impl_trait_in_trait)]

pub mod graph;
pub mod orbits;
pub mod utils;
mod quadruple_perfect_hash;

pub mod prelude {
    pub use crate::utils::*;
    pub use crate::graph::*;
    pub use crate::orbits::*;
}