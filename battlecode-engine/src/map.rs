//! The starting properties of the game world.

use std::f32;
use failure::Error;
use fnv::FnvHashMap;

use constants::*;
use location::*;
use unit::Unit;

/// The map for one of the planets in the Battlecode world. This information
/// defines the terrain, dimensions, and starting units of the planet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Map {
    /// The height of this map, in squares. Must be in the range
    /// [constants::MAP_MIN_HEIGHT, constants::MAP_MAX_HEIGHT], inclusive.
    pub height: usize,

    /// The width of this map, in squares. Must be in the range
    /// [constants::MAP_MIN_WIDTH, constants::MAP_MAX_WIDTH], inclusive.
    pub width: usize,

    /// The coordinates of the bottom-left corner. Essentially, the
    /// minimum x and y coordinates for this map. Each lies within
    /// [constants::MAP_MIN_COORDINATE, constants::MAP_MAX_COORDINATE],
    /// inclusive.
    pub origin: MapLocation,

    /// Whether the specified square contains passable terrain. Is only
    /// false when the square contains impassable terrain (distinct from
    /// containing a building, for instance).
    ///
    /// Stored as a two-dimensional array, where the first index 
    /// represents a square's y-coordinate, and the second index its 
    /// x-coordinate. These coordinates are *relative to the origin*.
    ///
    /// Earth is always symmetric by either a rotation or a reflection.
    pub is_passable_terrain: Vec<Vec<bool>>,

    /// The amount of Karbonite deposited on the specified square.
    ///
    /// Stored as a two-dimensional array, where the first index 
    /// represents a square's y-coordinate, and the second index its 
    /// x-coordinate. These coordinates are *relative to the origin*.
    pub starting_karbonite: Vec<Vec<u32>>,

    /// The starting units on the map. Each team starts with 1 to 3 Workers
    /// on Earth. The coordinates of the units are absolute (NOT relative to
    /// the origin).
    pub starting_units: Vec<Unit>,
}

impl Map {
    /// Validates the map and checks some invariants are followed.
    pub fn validate(map: Map) -> Result<(), Error> {
        unimplemented!();
    }

    pub fn test_map() -> Map {
        Map {
            height: MAP_MIN_HEIGHT,
            width: MAP_MIN_WIDTH,
            origin: MapLocation::new(Planet::Earth, 0, 0),
            is_passable_terrain: vec![vec![true; MAP_MIN_WIDTH]; MAP_MIN_HEIGHT],
            starting_karbonite: vec![vec![0; MAP_MIN_WIDTH]; MAP_MIN_HEIGHT],
            starting_units: vec![],
        }
    }
}

/// A single asteroid strike on Mars.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AsteroidStrike {
    /// The karbonite on the asteroid.
    karbonite: u32,
    /// The location of the strike.
    location: MapLocation,
}

/// The round number to an asteroid strike.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AsteroidPattern {
    pattern: FnvHashMap<u32, AsteroidStrike>,
}

/// The orbit pattern that determines a rocket's flight duration. This pattern
/// is a sinusoidal function y=a*sin(bx)+c.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrbitPattern {
    /// Amplitude of the orbit.
    a: i32,
    /// 2*pi / the period of the orbit.
    b: i32,
    /// The average of the orbit.
    c: i32,
}

impl AsteroidPattern {
    /// Constructs a new asteroid pattern from a map of round number to strike.
    pub fn new(pattern: FnvHashMap<u32, AsteroidStrike>) -> AsteroidPattern {
        AsteroidPattern {
            pattern: pattern,
        }
    }

    /// Validates the asteroid pattern.
    pub fn validate(&self) -> Result<(), Error> {
        // The Karbonite on each asteroid is in the range
        // [MAP_ASTEROID_KARB_MIN, MAP_ASTEROID_KARB_MAX], inclusive.

        // An asteroid strikes every [MAP_ASTEROID_ROUND_MIN,
        // MAP_ASTEROID_ROUND_MAX] rounds, inclusive.
        unimplemented!();
    }

    /// Get the asteroid strike at the given round.
    pub fn get_asteroid(&self, round: u32) -> Option<&AsteroidStrike> {
        self.pattern.get(&round)
    }
}

impl OrbitPattern {
    /// Construct a new orbit pattern. This pattern is a sinusoidal function
    /// y=a*sin(bx)+c, where the x-axis is the round number of takeoff and the
    /// the y-axis is the flight of duration to the nearest integer.
    ///
    /// The sine function has an amplitude of a, a period of 2*pi/b, and an
    /// average of c.
    pub fn new(a: i32, b: i32, c: i32) -> OrbitPattern {
        OrbitPattern {
            a: a,
            b: b,
            c: c,
        }
    }

    /// Validates the orbit pattern.
    pub fn validate(&self) -> Result<(), Error> {
        // The flight duration is always non-negative.

        // a, b, and c are within the spec's constraints.
        unimplemented!();
    }

    /// Get the duration of flight if the rocket were to take off from either
    /// planet on the given round.
    pub fn get_duration(&self, round: i32) -> i32 {
        ((self.a as f32) * f32::sin((self.b * round) as f32)) as i32 + self.c
    }
}

/// The weather patterns defined in the game world.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WeatherPattern {
    /// The asteroid strike pattern on Mars.
    pub asteroids: AsteroidPattern,
    /// The orbit pattern that determines a rocket's flight duration.
    pub orbit: OrbitPattern,
}

impl WeatherPattern {
    /// Construct a new weather pattern.
    pub fn new(asteroids: AsteroidPattern, orbit: OrbitPattern) -> WeatherPattern {
        WeatherPattern {
            asteroids: asteroids,
            orbit: orbit,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn validate_asteroid() {
        unimplemented!();
    }

    #[test]
    fn validate_orbit() {
        unimplemented!();
    }

    #[test]
    fn validate_map() {
        unimplemented!();
    }

    #[test]
    fn get_asteroid() {
        unimplemented!();
    }

    #[test]
    fn get_duration() {
        unimplemented!();
    }

    #[test]
    fn construct_weather() {
        unimplemented!();
    }
}
