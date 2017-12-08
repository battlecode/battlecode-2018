//! The core battlecode engine.

use super::schema::Delta;
use super::location::MapLocation;
use super::location::Planet;

/// The full world of the Battlecode game.
pub struct GameWorld {
    earth: Map,
    mars: Map,
}

/// The map for one of the planets in the Battlecode world.
#[derive(Debug, Serialize, Deserialize)]
pub struct Map {
    /// The planet of this map.
    planet: Planet,

    /// The height of this map, in squares.
    height: u32,

    /// The width of this map, in squares.
    width: u32,

    /// The coordinates of the bottom-left corner. Essentially, the
    /// minimum x and y coordinates for this map. Each lies within
    /// [-10,000, 10,000].
    origin: MapLocation,

    /// Whether the specified square contains passable terrain. Is only
    /// false when the square contains impassable terrain (distinct from
    /// containing a building, for instance).
    ///
    /// Stored as a two-dimensional array, where the first index 
    /// represents a square's y-coordinate, and the second index its 
    /// x-coordinate. These coordinates are *relative to the origin*.
    is_passable_terrain: Vec<Vec<bool>>,

    /// The amount of Karbonite deposited on the specified square.
    ///
    /// Stored as a two-dimensional array, where the first index 
    /// represents a square's y-coordinate, and the second index its 
    /// x-coordinate. These coordinates are *relative to the origin*.
    karbonite: Vec<Vec<u32>>,

    /// A list of all entities (robots + buildings) present in the map.
    entitites: Vec<()>,
}