//! The Battlecode Engine.
//! Provides an embeddable library with no dependencies (besides libc), which
//! holds battlecode state, and can query and apply changes to that state.
//! It can also 

// Enable the clippy linter when we build with the feature "clippy".
// Otherwise, this does nothing.
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

// Serialization.
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

// Error handling.
#[macro_use] extern crate failure;

// Random number generation.
extern crate rand;

// Provides FnvHashMap and FnvHashSet, which are like std::HashMap and std::HashSet, but
// significantly faster for integer keys.
extern crate fnv;

// see error.rs
#[macro_use]
pub mod error;

// see controller.rs
pub mod controller;

// see config.rs
pub mod config;

// see schema.rs
pub mod schema;

// see world.rs
pub mod world;

// see location.rs
pub mod location;

// see map.rs
pub mod map;

// see unit.rs
pub mod unit;

// see research.rs
pub mod research;

// see rockets.rs
pub mod rockets;

// see constants.rs
pub mod constants;

// see id_generator.rs
mod id_generator;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
