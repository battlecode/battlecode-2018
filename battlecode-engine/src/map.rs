//! The starting properties of the game world.

use std::f32;
use failure::Error;
use fnv::FnvHashMap;

use constants::*;
use location::*;
use unit::Unit;

/// The map for one of the planets in the Battlecode world. This information
/// defines the terrain, dimensions, and initial units of the planet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Map {
    /// The planet of the map.
    pub planet: Planet,

    /// The height of this map, in squares. Must be in the range
    /// [constants::MAP_HEIGHT_MIN, constants::MAP_HEIGHT_MAX], inclusive.
    pub height: usize,

    /// The width of this map, in squares. Must be in the range
    /// [constants::MAP_WIDTH_MIN, constants::MAP_WIDTH_MAX], inclusive.
    pub width: usize,

    /// The coordinates of the bottom-left corner. Essentially, the
    /// minimum x and y coordinates for this map. Each lies within
    /// [constants::MAP_COORDINATE_MIN, constants::MAP_COORDINATE_MAX],
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
    pub initial_karbonite: Vec<Vec<u32>>,

    /// The initial units on the map. Each team starts with 1 to 3 Workers
    /// on Earth. The coordinates of the units are absolute (NOT relative to
    /// the origin).
    pub initial_units: Vec<Unit>,
}

impl Map {
    /// Validates the map and checks some invariants are followed.
    pub fn validate(&self) -> Result<(), Error> {
        // The width and height are of valid dimensions.
        assert!(self.height >= MAP_HEIGHT_MIN);
        assert!(self.height <= MAP_HEIGHT_MAX);
        assert!(self.width >= MAP_WIDTH_MIN);
        assert!(self.width <= MAP_WIDTH_MAX);

        // The origin is valid.
        assert!(self.origin.x >= MAP_COORDINATE_MIN);
        assert!(self.origin.x <= MAP_COORDINATE_MAX);
        assert!(self.origin.y >= MAP_COORDINATE_MIN);
        assert!(self.origin.y <= MAP_COORDINATE_MAX);
        assert_eq!(self.origin.planet, self.planet);

        // The terrain definition is valid.
        assert_eq!(self.is_passable_terrain.len(), self.height);
        assert_eq!(self.is_passable_terrain[0].len(), self.width);

        // The initial karbonite deposits are valid.
        assert_eq!(self.initial_karbonite.len(), self.height);
        assert_eq!(self.initial_karbonite[0].len(), self.width);
        for y in 0..self.height {
            for x in 0..self.width {
                match self.planet {
                    Planet::Mars => { assert_eq!(self.initial_karbonite[y][x], 0); }
                    Planet::Earth => {
                        assert!(self.initial_karbonite[y][x] >= MAP_KARBONITE_MIN);
                        assert!(self.initial_karbonite[y][x] <= MAP_KARBONITE_MAX);
                    }
                }
            }
        }

        // The initial units are valid.
        let num_units = self.initial_units.len();
        match self.planet {
            Planet::Mars => { assert_eq!(num_units, 0); }
            Planet::Earth => { assert!(num_units > 0 &&
                                       num_units % 2 == 0 &&
                                       num_units <= 6); }
        }
        for ref unit in &self.initial_units {
            let x = (unit.location.x - self.origin.x) as usize;
            let y = (unit.location.y - self.origin.y) as usize;
            assert_eq!(unit.location.planet, self.planet);
            assert!(self.is_passable_terrain[y][x]);
        }

        // The map is symmetric on Earth.
        if self.planet == Planet::Earth {
            // TODO
        }
        Ok(())
    }

    pub fn test_map() -> Map {
        Map {
            planet: Planet::Earth,
            height: MAP_HEIGHT_MIN,
            width: MAP_WIDTH_MIN,
            origin: MapLocation::new(Planet::Earth, 0, 0),
            is_passable_terrain: vec![vec![true; MAP_WIDTH_MIN]; MAP_HEIGHT_MIN],
            initial_karbonite: vec![vec![0; MAP_WIDTH_MIN]; MAP_HEIGHT_MIN],
            initial_units: vec![],
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
        // [ASTEROID_KARB_MIN, ASTEROID_KARB_MAX], inclusive.
        for (round, asteroid) in self.pattern.clone() {
            assert!(round >= 1);
            assert!(round <= ROUND_LIMIT);
            assert!(asteroid.karbonite >= ASTEROID_KARB_MIN);
            assert!(asteroid.karbonite <= ASTEROID_KARB_MAX);
            assert_eq!(asteroid.location.planet, Planet::Mars);
        }

        // An asteroid strikes every [ASTEROID_ROUND_MIN,
        // ASTEROID_ROUND_MAX] rounds, inclusive.
        let mut rounds: Vec<&u32> = self.pattern.keys().collect();
        if !rounds.contains(&&1) {
            rounds.push(&1);
        }
        if !rounds.contains(&&ROUND_LIMIT) {
            rounds.push(&ROUND_LIMIT);
        }
        rounds.sort();
        for i in 0..rounds.len() - 1 {
            let diff = rounds[i + 1] - rounds[i];
            assert!(diff >= ASTEROID_ROUND_MIN);
            assert!(diff <= ASTEROID_ROUND_MAX);
        }
        Ok(())
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
        // The flight times are within [ORIBIT_FLIGHT_MIN, ORBIT_FLIGHT_MAX].
        assert!(self.c - self.a >= ORBIT_FLIGHT_MIN);
        assert!(self.c + self.a <= ORBIT_FLIGHT_MAX);
        Ok(())
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

    /// Validate the weather pattern.
    pub fn validate(&self) -> Result<(), Error> {
        self.asteroids.validate()?;
        self.orbit.validate()?;
        Ok(())
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
