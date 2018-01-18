//! The starting properties of the game world.

use std::{cmp, f32};
use failure::Error;
use fnv::FnvHashMap;
use rand::{SeedableRng, StdRng};
use rand::distributions::IndependentSample;
use rand::distributions::range::Range;

use constants::*;
use error::GameError;
use location::*;
use unit::*;
use world::*;

mod mapparser;

enum Symmetry {
    Rotational,
    Horizontal,
    Vertical
}

/// The map defining the starting state for an entire game.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameMap {
    /// Seed for random number generation.
    pub seed: u16,
    /// Earth map.
    pub earth_map: PlanetMap,
    /// Mars map.
    pub mars_map: PlanetMap,
    /// The asteroid strike pattern on Mars.
    pub asteroids: AsteroidPattern,
    /// The orbit pattern that determines a rocket's flight duration.
    pub orbit: OrbitPattern,
}

impl GameMap {
    /// Validate the game map.
    ///
    /// * InvalidMapObject - the game map is invalid.
    pub fn validate(&self) {
        if self.earth_map.validate() && self.mars_map.validate() &&
            self.asteroids.validate() && self.orbit.validate() {
            println!("Map is valid!");
        }
    }

    pub fn map_template(seed: u16, width: usize, height: usize) -> GameMap {
        let rng_seed: &[_] = &[seed as usize];
        let mut rng: StdRng = SeedableRng::from_seed(rng_seed);
        let seed_rng = Range::new(0, u16::max_value());

        let mut earth = PlanetMap::empty_map(Planet::Earth, width, height);
        let mars = PlanetMap::empty_map(Planet::Mars, width, height);

        let asteroids = AsteroidPattern::random(seed_rng.ind_sample(&mut rng), &mars);
        let orbit = OrbitPattern::random(seed_rng.ind_sample(&mut rng));

        earth.initial_units.push(Unit::new(
            1, Team::Red, UnitType::Worker, 0,
            Location::OnMap(MapLocation::new(Planet::Earth, 1, 1))
        ).unwrap());
        earth.initial_units.push(Unit::new(
            2, Team::Blue, UnitType::Worker, 0,
            Location::OnMap(MapLocation::new(Planet::Earth, width as i32 - 1, width as i32 - 1))
        ).unwrap());

        GameMap {
            seed: seed,
            earth_map: earth,
            mars_map: mars,
            asteroids: asteroids,
            orbit: orbit
        }
    }

    pub fn test_map() -> GameMap {
        let seed = 1;
        let mars_map = PlanetMap::test_map(Planet::Mars);
        GameMap {
            seed: seed,
            earth_map: PlanetMap::test_map(Planet::Earth),
            mars_map: mars_map.clone(),
            asteroids: AsteroidPattern::random(seed, &mars_map),
            orbit: OrbitPattern::new(50, 100, 100),
        }
    }

    pub fn parse_text_map(map: &str) -> Result<GameMap, Error> {
        self::mapparser::parse_text_map(map)
    }
}

/// The map for one of the planets in the Battlecode world. This information
/// defines the terrain, dimensions, and initial units of the planet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanetMap {
    /// The planet of the map.
    pub planet: Planet,

    /// The height of this map, in squares. Must be in the range
    /// [[`MAP_HEIGHT_MIN`](../constants/constant.MAP_HEIGHT_MIN.html),
    /// [`MAP_HEIGHT_MAX`](../constants/constant.MAP_HEIGHT_MAX.html)],
    /// inclusive.
    pub height: usize,

    /// The height of this map, in squares. Must be in the range
    /// [[`MAP_WIDTH_MIN`](../constants/constant.MAP_WIDTH_MIN.html),
    /// [`MAP_WIDTH_MAX`](../constants/constant.MAP_WIDTH_MAX.html)],
    /// inclusive.
    pub width: usize,

    /// The initial units on the map. Each team starts with 1 to 3 Workers
    /// on Earth.
    pub initial_units: Vec<Unit>,

    /// Whether the specified square contains passable terrain. Is only
    /// false when the square contains impassable terrain (distinct from
    /// containing a building, for instance).
    ///
    /// Stored as a two-dimensional array, where the first index 
    /// represents a square's y-coordinate, and the second index its 
    /// x-coordinate.
    ///
    /// Earth is always symmetric by either a rotation or a reflection.
    pub is_passable_terrain: Vec<Vec<bool>>,

    /// The amount of Karbonite deposited on the specified square.
    ///
    /// Stored as a two-dimensional array, where the first index 
    /// represents a square's y-coordinate, and the second index its 
    /// x-coordinate.
    ///
    /// Earth is always symmetric by either a rotation or a reflection.
    pub initial_karbonite: Vec<Vec<u32>>,
}

impl PlanetMap {
    /// Validates the map and checks some invariants are followed.
    ///
    /// * InvalidMapObject - the planet map is invalid.
    pub fn validate(&self) -> bool {
        let mut valid = true;

        // The width and height are of valid dimensions.
        if !(self.height >= MAP_HEIGHT_MIN && self.height <= MAP_HEIGHT_MAX &&
             self.width >= MAP_WIDTH_MIN && self.width <= MAP_WIDTH_MAX) {
            println!("Map dimensions invalid: {} x {}", self.width, self.height);
            valid = false;
        }

        // The passable terrain defition has the same dimensions as the map.
        if self.is_passable_terrain.len() != self.height ||
           self.is_passable_terrain[0].len() != self.width {
            println!("Is passable terrain dimensions invalid: {} x {}",
                self.is_passable_terrain.len(), self.is_passable_terrain[0].len());
            valid = false;
        }

        // The initial karbonite deposits have the same dimensions as the map.
        if self.initial_karbonite.len() != self.height ||
           self.initial_karbonite[0].len() != self.width {
            println!("Initial karbonite dimensions invalid: {} x {}",
                self.is_passable_terrain.len(), self.is_passable_terrain[0].len());
            valid = false;
        }
        for y in 0..self.height {
            for x in 0..self.width {
                match self.planet {
                    Planet::Mars => {
                        // Mars cannot have any initial karbonite.
                        if self.initial_karbonite[y][x] != 0 {
                            println!("Mars has initial karbonite {} at ({}, {})",
                                self.initial_karbonite[y][x], x, y);
                            valid = false;
                        }
                    }
                    Planet::Earth => {
                        // Earth's initial karbonite has limited values.
                        if self.initial_karbonite[y][x] < MAP_KARBONITE_MIN ||
                           self.initial_karbonite[y][x] > MAP_KARBONITE_MAX {
                            println!("Earth has initial karbonite {} at ({}, {})",
                                self.initial_karbonite[y][x], x, y);
                            valid = false;
                        }
                    }
                }
            }
        }

        // The number of initial units is valid.
        let num_units = self.initial_units.len();
        match self.planet {
            Planet::Mars => {
                // There are no units on Mars.
                if num_units != 0 {
                    println!("Mars should not have initial units.");
                    valid = false;
                }
            }
            Planet::Earth => {
                // There are 1 to 3 workers per team on Earth.
                if !(num_units > 0 && num_units % 2 == 0 && num_units <= 6) {
                    println!("Earth should not have {} initial units.", num_units);
                    valid = false;
                }
            }
        }
        for ref unit in &self.initial_units {
            let location = match unit.location().map_location() {
                Ok(location) => location,
                _ => {
                    println!("Unit {} should be on the map", unit.id());
                    valid = false;
                    continue;
                }
            };
            let x = location.x as usize;
            let y = location.y as usize;

            // Unit must be on this planet
            if location.planet != self.planet {
                println!("Unit {} should not be on this planet: {:?}", unit.id(), location.planet);
                valid = false;
            }
            // Unit must be on passable terrain
            if !self.is_passable_terrain[y][x] {
                println!("Unit {} on ({}, {}) should be on passable terrain",
                    unit.id(), x, y);
                valid = false;
            }
        }

        // The map is symmetric on Earth.
        if self.planet == Planet::Earth {
            if self.is_terrain_karbonite_symmetric(Symmetry::Rotational) {
                return valid;
            }
            if self.is_terrain_karbonite_symmetric(Symmetry::Horizontal) {
                return valid;
            }
            if self.is_terrain_karbonite_symmetric(Symmetry::Vertical) {
                return valid;
            }
            println!("Earth is not symmetric");
            valid = false;
        }
        valid
    }

    fn is_terrain_karbonite_symmetric(&self, symmetry: Symmetry) -> bool {
        fn flip(n: usize, max_n: usize) -> usize {
            let mid_n = max_n / 2;
            let new_n = -(n as i32 - mid_n as i32) + mid_n as i32;
            (new_n - (1 - max_n as i32 % 2)) as usize
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let (new_x, new_y) = match symmetry {
                    Symmetry::Rotational => (flip(x, self.width), y),
                    Symmetry::Horizontal => (x, flip(y, self.height)),
                    Symmetry::Vertical => (flip(x, self.width), flip(y, self.height)),
                };
                if self.is_passable_terrain[y][x] != self.is_passable_terrain[new_y][new_x] {
                    return false;
                }
                if self.is_passable_terrain[y][x] != self.is_passable_terrain[new_y][new_x] {
                    return false;
                }
            }
        }
        true
    }

    /// Whether a location is on the map.
    pub fn on_map(&self, location: MapLocation) -> bool {
        self.planet == location.planet
            && location.x >= 0
            && location.y >= 0
            && location.x < self.width as i32
            && location.y < self.height as i32
    }

    /// Whether the location on the map contains passable terrain. Is only
    /// false when the square contains impassable terrain (distinct from
    /// containing a building, for instance).
    ///
    /// * LocationOffMap - the location is off the map.
    pub fn is_passable_terrain_at(&self, location: MapLocation) -> Result<bool, GameError> {
        if self.on_map(location) {
            Ok(self.is_passable_terrain[location.y as usize][location.x as usize])
        } else {
            Err(GameError::LocationOffMap)?
        }
    }

    /// The amount of Karbonite initially deposited at the given location.
    ///
    /// * LocationOffMap - the location is off the map.
    pub fn initial_karbonite_at(&self, location: MapLocation) -> Result<u32, GameError> {
        if self.on_map(location) {
            Ok(self.initial_karbonite[location.y as usize][location.x as usize])
        } else {
            Err(GameError::LocationOffMap)?
        }
    }

    fn empty_map(planet: Planet, width: usize, height: usize) -> PlanetMap {
        PlanetMap {
            planet: planet,
            height: height,
            width: width,
            is_passable_terrain: vec![vec![true; width]; height],
            initial_karbonite: vec![vec![0; width]; height],
            initial_units: vec![],
        }
    }

    fn test_map(planet: Planet) -> PlanetMap {
        let karbonite = match planet {
            Planet::Earth => 10,
            Planet::Mars => 0,
        };

        let mut map = PlanetMap {
            planet: planet,
            height: MAP_HEIGHT_MIN,
            width: MAP_WIDTH_MIN,
            is_passable_terrain: vec![vec![true; MAP_WIDTH_MIN]; MAP_HEIGHT_MIN],
            initial_karbonite: vec![vec![karbonite; MAP_WIDTH_MIN]; MAP_HEIGHT_MIN],
            initial_units: vec![],
        };

        if planet == Planet::Earth {
            map.initial_units.push(Unit::new(
                1, Team::Red, UnitType::Worker, 0,
                Location::OnMap(MapLocation::new(planet, 1, 1))
            ).expect("invalid test unit"));
            map.initial_units.push(Unit::new(
                2, Team::Blue, UnitType::Worker, 0,
                Location::OnMap(MapLocation::new(planet, MAP_WIDTH_MIN as i32 - 2, MAP_HEIGHT_MIN as i32 - 2))
            ).expect("invalid test unit"));
        };

        map
    }
}

/// A single asteroid strike on Mars.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct AsteroidStrike {
    /// The karbonite on the asteroid.
    pub karbonite: u32,
    /// The location of the strike.
    pub location: MapLocation,
}

/// The asteroid pattern, defined by the timing and contents of each asteroid
/// strike.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AsteroidPattern {
    pub(crate) pattern: FnvHashMap<Rounds, AsteroidStrike>,
}

/// The orbit pattern that determines a rocket's flight duration. This pattern
/// is a sinusoidal function y=a*sin(bx)+c.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrbitPattern {
    /// Amplitude of the orbit.
    pub amplitude: Rounds,
    /// The period of the orbit.
    pub period: Rounds,
    /// The center of the orbit.
    pub center: Rounds,

    amplitude_s: i32,
    period_s: i32,
    center_s: i32,
}

impl AsteroidStrike {
    /// Constructs a new asteroid strike.
    pub fn new(karbonite: u32, location: MapLocation) -> AsteroidStrike {
        AsteroidStrike {
            karbonite: karbonite,
            location: location,
        }
    }
}

impl AsteroidPattern {
    /// Constructs a new asteroid pattern from a map of round number to strike.
    pub fn new(pattern: &FnvHashMap<Rounds, AsteroidStrike>) -> AsteroidPattern {
        AsteroidPattern {
            pattern: pattern.clone(),
        }
    }

    /// Constructs a pseudorandom asteroid pattern given a map of Mars.
    pub fn random(seed: u16, mars_map: &PlanetMap) -> AsteroidPattern {
        let mut pattern: FnvHashMap<Rounds, AsteroidStrike> = FnvHashMap::default();

        let karbonite_gen = Range::new(ASTEROID_KARB_MIN, ASTEROID_KARB_MAX);
        let round_gen = Range::new(0, 40);
        let x_gen = Range::new(0, mars_map.width as i32);
        let y_gen = Range::new(0, mars_map.height as i32);

        let seed: &[_] = &[seed as usize];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let mut round = 0;
        loop {
            round += round_gen.ind_sample(&mut rng);
            if round >= ROUND_LIMIT {
                break;
            }
            pattern.insert(round, AsteroidStrike {
                karbonite: karbonite_gen.ind_sample(&mut rng),
                location: MapLocation {
                    planet: Planet::Mars,
                    x: x_gen.ind_sample(&mut rng),
                    y: y_gen.ind_sample(&mut rng),
                },
            });
        }

        AsteroidPattern {
            pattern: pattern,
        }
    }

    /// Validates the asteroid pattern.
    ///
    /// * InvalidMapObject - the asteroid pattern is invalid.
    pub fn validate(&self) -> bool {
        let mut valid = true;

        // The Karbonite on each asteroid is in the range
        // [ASTEROID_KARB_MIN, ASTEROID_KARB_MAX], inclusive.
        for (round, asteroid) in self.pattern.clone() {
            if round < 1 || round > ROUND_LIMIT {
                println!("Asteroid at invalid round: {}", round);
                valid = false;
            }
            if asteroid.karbonite < ASTEROID_KARB_MIN ||
               asteroid.karbonite > ASTEROID_KARB_MAX {
                println!("Asteroid with invalid karbonite: {}", asteroid.karbonite);
                valid = false;
            }
            if asteroid.location.planet != Planet::Mars {
                println!("Asteroid landing on Earth at round {}", round);
                valid = false;
            }
        }
        valid
    }

    /// Whether there is an asteroid strike at the given round.
    pub fn has_asteroid(&self, round: Rounds) -> bool {
        self.pattern.get(&round).is_some()
    }

    /// Get the asteroid strike at the given round.
    ///
    /// * NullValue - There is no asteroid strike at this round.
    pub fn asteroid(&self, round: Rounds) -> Result<&AsteroidStrike, GameError> {
        if let Some(asteroid) = self.pattern.get(&round) {
            Ok(asteroid)
        } else {
            Err(GameError::NullValue)?
        }
    }

    /// Get a map of round numbers to asteroid strikes.
    pub fn asteroid_map(&self) -> FnvHashMap<Rounds, AsteroidStrike> {
        self.pattern.clone()
    }
}

impl OrbitPattern {
    /// Construct a new orbit pattern. This pattern is a sinusoidal function
    /// y=a*sin(bx)+c, where the x-axis is the round number of takeoff and the
    /// the y-axis is the duration of flight to the nearest integer.
    ///
    /// The amplitude, period, and center are measured in rounds.
    pub fn new(amplitude: Rounds, period: Rounds, center: Rounds) -> OrbitPattern {
        OrbitPattern {
            amplitude: amplitude,
            period: period,
            center: center,
            amplitude_s: amplitude as i32,
            period_s: period as i32,
            center_s: center as i32,
        }
    }

    /// Validates the orbit pattern.
    ///
    /// * InvalidMapObject - the orbit pattern is invalid.
    pub fn validate(&self) -> bool {
        // The flight times are within [ORIBIT_FLIGHT_MIN, ORBIT_FLIGHT_MAX].
        let mut valid = true;
        if self.center - self.amplitude < ORBIT_FLIGHT_MIN {
            println!("Min orbit time is too short: {:?}", self);
            valid = false;
        }
        if self.center + self.amplitude > ORBIT_FLIGHT_MAX {
            println!("Max orbit time is too long: {:?}", self);
            valid = false;
        }
        valid
    }

    /// Get the duration of flight if the rocket were to take off from either
    /// planet on the given round.
    pub fn duration(&self, round: Rounds) -> Rounds {
        let arg = 2. * f32::consts::PI / self.period_s as f32 * round as f32;
        let sin = ((self.amplitude_s as f32) * f32::sin(arg)) as i32;
        (sin + self.center_s) as Rounds
    }

    /// Generates a random orbit pattern.
    pub fn random(seed: u16) -> OrbitPattern {
        let seed: &[_] = &[seed as usize];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let center = Range::new(ORBIT_FLIGHT_MIN, ORBIT_FLIGHT_MAX).ind_sample(&mut rng);
        let max_amplitude = cmp::min(center - ORBIT_FLIGHT_MIN, ORBIT_FLIGHT_MAX - center);
        let amplitude = Range::new(0, max_amplitude).ind_sample(&mut rng);
        let period = Range::new(ORBIT_PERIOD_MIN, ORBIT_PERIOD_MAX).ind_sample(&mut rng);
        OrbitPattern::new(amplitude, period, center)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn insert_and_err(pattern: &FnvHashMap<Rounds, AsteroidStrike>,
                      round: Rounds, karbonite: u32, location: MapLocation) {
        let mut invalid = pattern.clone();
        invalid.insert(round, AsteroidStrike::new(karbonite, location));
        assert!(!AsteroidPattern::new(&invalid).validate());
    }

    fn gen_asteroid_map(start_round: Rounds, skip_round: Rounds)
                        -> FnvHashMap<Rounds, AsteroidStrike> {
        let mut asteroid_map: FnvHashMap<Rounds, AsteroidStrike> = FnvHashMap::default();
        let mut round = start_round;
        let default_loc = MapLocation::new(Planet::Mars, 0, 0);
        let default_strike = AsteroidStrike::new(ASTEROID_KARB_MIN, default_loc);
        while round <= ROUND_LIMIT {
            asteroid_map.insert(round, default_strike.clone());
            round += skip_round;
        }
        asteroid_map
    }

    #[test]
    fn validate_asteroid() {
        // Valid randomly-generated asteroid patterns.
        let ref mars_map = super::PlanetMap::test_map(Planet::Mars);
        for seed in 0..5 {
            assert!(AsteroidPattern::random(seed, mars_map).validate());
        }

        // Generate an asteroid pattern from a map.
        let asteroid_map = AsteroidPattern::random(0, mars_map).asteroid_map();
        let asteroids = AsteroidPattern::new(&asteroid_map);
        assert!(asteroids.validate());

        let asteroid_map = gen_asteroid_map(1, 50);
        assert!(AsteroidPattern::new(&asteroid_map).validate());

        // Invalid asteroid strikes.
        let loc = MapLocation::new(Planet::Mars, 0, 0);
        insert_and_err(&asteroid_map, 0, ASTEROID_KARB_MIN, loc);
        insert_and_err(&asteroid_map, ROUND_LIMIT + 1, ASTEROID_KARB_MIN, loc);
        insert_and_err(&asteroid_map, 1, ASTEROID_KARB_MIN - 1, loc);
        insert_and_err(&asteroid_map, 1, ASTEROID_KARB_MAX + 1, loc);
        insert_and_err(&asteroid_map, 1, ASTEROID_KARB_MAX, MapLocation::new(Planet::Earth, 0, 0));
    }

    #[test]
    fn validate_orbit() {
        assert!(!OrbitPattern::new(150, 200, 200).validate());
        assert!(!OrbitPattern::new(150, 200, 300).validate());
        assert!(OrbitPattern::new(75, 200, 125).validate());
    }

    #[test]
    fn test_asteroid() {
        let asteroid_map = gen_asteroid_map(50, 50);
        let asteroids = AsteroidPattern::new(&asteroid_map);
        for round in 1..ROUND_LIMIT {
            if round % 50 == 0 {
                assert!(asteroids.asteroid(round).is_ok());
            } else {
                assert_err!(asteroids.asteroid(round), GameError::NullValue);
            }
        }
    }

    #[test]
    fn test_duration() {
        let period = 200;
        let orbit = OrbitPattern::new(150, period, 250);
        for i in 0..5 {
            let base = period * i;
            assert_eq!(250, orbit.duration(base));
            assert_eq!(400, orbit.duration(base + period / 4));
            assert_eq!(250, orbit.duration(base + period / 2));
            assert_eq!(100, orbit.duration(base + period * 3 / 4));
            assert_eq!(250, orbit.duration(base + period));

            let duration = orbit.duration(base + period / 8);
            assert!(duration > 250 && duration < 400);
        }
    }
}
