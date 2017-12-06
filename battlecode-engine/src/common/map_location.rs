//! Represents two-dimensional coordinates in the Battlecode world.

/// The planets in the Battlecode world.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Planet {
    Earth,
    Mars,
}

/// A two-dimensional coordinate on one of the Battlecode planets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MapLocation {
    pub x: i32,
    pub y: i32,
    pub planet: Planet,
}

impl MapLocation {
    /// Returns a new MapLocation representing the location with the given
    /// coordinates on the given planet.
    pub fn new(x: i32, y: i32, planet: Planet) -> MapLocation {
        MapLocation { x: x, y: y, planet: planet }
    }

    // TODO: more methods
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
