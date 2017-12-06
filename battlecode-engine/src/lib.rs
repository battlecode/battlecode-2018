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
extern crate failure;

// Provides FnvHashMap and FnvHashSet, which are like std::HashMap and std::HashSet, but
// significantly faster for integer keys.
extern crate fnv;

// see schema.rs
pub mod schema;

// see engine.rs
pub mod engine;

// export everything from the engine to everyone who wants to use it
pub use engine::*;

// simple modules available to all competitors
pub mod common;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
