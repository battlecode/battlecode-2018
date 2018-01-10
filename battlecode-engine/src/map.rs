//! The starting properties of the game world.

use std::f32;
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
    pub fn new(seed: u16, earth_map: PlanetMap, mars_map: PlanetMap,
               asteroids: AsteroidPattern, orbit: OrbitPattern) -> GameMap {
        GameMap {
            seed: seed,
            earth_map: earth_map,
            mars_map: mars_map,
            asteroids: asteroids,
            orbit: orbit,
        }
    }

    /// Validate the game map.
    ///
    /// * InvalidMapObject - the game map is invalid.
    pub fn validate(&self) -> Result<(), Error> {
        self.earth_map.validate()?;
        self.mars_map.validate()?;
        self.asteroids.validate()?;
        self.orbit.validate()?;
        Ok(())
    }

    pub fn test_map() -> GameMap {
        let seed = 1;
        let mars_map = PlanetMap::test_map(Planet::Mars);
        GameMap {
            seed: seed,
            earth_map: PlanetMap::test_map(Planet::Earth),
            mars_map: mars_map.clone(),
            asteroids: AsteroidPattern::random(seed, &mars_map),
            orbit: OrbitPattern::new(100, 100, 300),
        }
    }
}

/// The type of symmetry for a given planet map.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SymmetryType {
    /// The map is symmetric by a 180 degree rotation.
    Rotational = 0,

    /// The map is horizontally symmetric.
    Horizontal = 1,

    /// The map is vertically symmetric.
    Vertical = 2,

    /// The map is Mars, and has no symmetry.
    Mars = 3,
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

    /// The symmetry type of the map. Should always be Mars if this is
    /// Mars's PlanetMap.
    symmetry: SymmetryType,
}

impl PlanetMap {
    /// Validates the map and checks some invariants are followed.
    ///
    /// * InvalidMapObject - the planet map is invalid.
    pub fn validate(&self) -> Result<(), Error> {
        // The width and height are of valid dimensions.
        if !(self.height >= MAP_HEIGHT_MIN && self.height <= MAP_HEIGHT_MAX &&
             self.width >= MAP_WIDTH_MIN && self.width <= MAP_WIDTH_MAX) {
            Err(GameError::InvalidMapObject)?
        }

        // The terrain definition is valid.
        if self.is_passable_terrain.len() != self.height ||
           self.is_passable_terrain[0].len() != self.width {
            Err(GameError::InvalidMapObject)?
        }

        // The initial karbonite deposits are valid.
        if self.initial_karbonite.len() != self.height ||
           self.initial_karbonite[0].len() != self.width {
            Err(GameError::InvalidMapObject)?
        }
        for y in 0..self.height {
            for x in 0..self.width {
                match self.planet {
                    Planet::Mars => {
                        if self.initial_karbonite[y][x] != 0 {
                            Err(GameError::InvalidMapObject)?
                        }
                    }
                    Planet::Earth => {
                        if self.initial_karbonite[y][x] < MAP_KARBONITE_MIN ||
                           self.initial_karbonite[y][x] > MAP_KARBONITE_MAX {
                            Err(GameError::InvalidMapObject)?
                        }
                    }
                }
            }
        }

        // The initial units are valid.
        let num_units = self.initial_units.len();
        match self.planet {
            Planet::Mars => {
                if num_units != 0 {
                    Err(GameError::InvalidMapObject)?
                }
            }
            Planet::Earth => {
                if !(num_units > 0 && num_units % 2 == 0 && num_units <= 6) {
                    Err(GameError::InvalidMapObject)?
                }
            }
        }
        for ref unit in &self.initial_units {
            let location = unit.location().map_location()
                               .or(Err(GameError::InvalidMapObject))?;
            let x = location.x as usize;
            let y = location.y as usize;
            if location.planet != self.planet {
                Err(GameError::InvalidMapObject)?
            }
            if !self.is_passable_terrain[y][x] {
                Err(GameError::InvalidMapObject)?
            }
        }

        // The map is symmetric on Earth.
        if self.planet == Planet::Earth {
            // TODO
        }
        Ok(())
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
    pub fn is_passable_terrain_at(&self, location: MapLocation) -> Result<bool, Error> {
        if self.on_map(location) {
            Ok(self.is_passable_terrain[location.y as usize][location.x as usize])
        } else {
            Err(GameError::LocationOffMap)?
        }
    }

    /// The amount of Karbonite initially deposited at the given location.
    ///
    /// * LocationOffMap - the location is off the map.
    pub fn initial_karbonite_at(&self, location: MapLocation) -> Result<u32, Error> {
        if self.on_map(location) {
            Ok(self.initial_karbonite[location.y as usize][location.x as usize])
        } else {
            Err(GameError::LocationOffMap)?
        }
    }

    fn test_map(planet: Planet) -> PlanetMap {
        let mut map = PlanetMap {
            planet: planet,
            height: MAP_HEIGHT_MIN,
            width: MAP_WIDTH_MIN,
            is_passable_terrain: vec![vec![true; MAP_WIDTH_MIN]; MAP_HEIGHT_MIN],
            initial_karbonite: vec![vec![10; MAP_WIDTH_MIN]; MAP_HEIGHT_MIN],
            initial_units: vec![],
            symmetry: match planet {
                Planet::Earth => SymmetryType::Rotational,
                Planet::Mars => SymmetryType::Mars,
            }
        };

        if planet == Planet::Earth {
            map.initial_units.push(Unit::new(
                1, Team::Red, UnitType::Worker, 0,
                Location::OnMap(MapLocation::new(planet, 1, 1))
            ).expect("invalid test unit"));
            map.initial_units.push(Unit::new(
                2, Team::Blue, UnitType::Worker, 0,
                Location::OnMap(MapLocation::new(planet, MAP_WIDTH_MIN as i32 - 1, MAP_HEIGHT_MIN as i32 - 1))
            ).expect("invalid test unit"));
        };

        map
    }

    /*** MAP EDITOR METHODS ***/

    /// Creates an empty Earth map.
    pub fn empty_earth_map(height: usize, width: usize, symmetry: SymmetryType) 
                           -> Result<PlanetMap, Error> {
        if height < MAP_HEIGHT_MIN || height > MAP_HEIGHT_MAX
           || width < MAP_WIDTH_MIN || width > MAP_WIDTH_MAX {
            Err(GameError::InvalidMapObject)?;
        }

        if symmetry == SymmetryType::Mars {
            Err(GameError::InvalidMapObject)?;
        }

        Ok(PlanetMap {
            planet: Planet::Earth,
            height: height,
            width: width,
            is_passable_terrain: vec![vec![true; width]; height],
            initial_karbonite: vec![vec![0; width]; height],
            initial_units: vec![],
            symmetry: symmetry,
        })
    }

    /// Creates an empty Mars map.
    pub fn empty_mars_map(height: usize, width: usize) -> Result<PlanetMap, Error> {
        if height < MAP_HEIGHT_MIN || height > MAP_HEIGHT_MAX
           || width < MAP_WIDTH_MIN || width > MAP_WIDTH_MAX {
            Err(GameError::InvalidMapObject)?;
        }

        Ok(PlanetMap {
            planet: Planet::Mars,
            height: height,
            width: width,
            is_passable_terrain: vec![vec![true; width]; height],
            initial_karbonite: vec![vec![0; width]; height],
            initial_units: vec![],
            symmetry: SymmetryType::Mars,
        })
    }

    fn ok_if_on_map(&self, x: usize, y: usize) -> Result<(), Error> {
        if x >= self.width || y >= self.height {
            Err(GameError::LocationOffMap)?;
        }
        Ok(())
    }

    fn reflect(&self, x: usize, y: usize) -> (usize, usize) {
        match self.symmetry {
            SymmetryType::Rotational => (self.width - x - 1, self.height - y - 1),
            SymmetryType::Horizontal => (self.width - x - 1, y),
            SymmetryType::Vertical => (x, self.height - y - 1),
            SymmetryType::Mars => (x, y),
        }
    }

    fn next_id(&self) -> UnitID {
        for id in 0..10 {
            let mut unique = true;
            for ref unit in self.initial_units.iter() {
                if unit.id() == id {
                    unique = false;
                    break;
                }
            }
            if unique {
                return id;
            }
        }
        panic!("Too many units on board!");
    }

    fn contains_unit(&self, x: usize, y: usize) -> bool {
        for ref unit in self.initial_units.iter() {
            if unit.location().map_location().unwrap().x as usize == x
               && unit.location().map_location().unwrap().y as usize == y {
                return true;
            }
        }
        false
    }

    pub fn add_starting_worker(&mut self, x: usize, y: usize) -> Result<(), Error> {
        self.ok_if_on_map(x, y)?;
        if self.planet == Planet::Mars {
            Err(GameError::InvalidMapObject)?;
        }
        if !self.is_passable_terrain[y][x] {
            Err(GameError::LocationNotEmpty)?;
        }
        if self.initial_units.len() > 6 {
            Err(GameError::InvalidMapObject)?;
        }
        if self.contains_unit(x, y) {
            Err(GameError::LocationNotEmpty)?;
        }

        let (o_x, o_y) = self.reflect(x, y);
        if (o_x, o_y) == (x, y) {
            Err(GameError::InvalidMapObject)?;
        }
        
        let red_id = self.next_id();
        self.initial_units.push(Unit::new(
            red_id, Team::Red, UnitType::Worker, 0,
            Location::OnMap(MapLocation::new(self.planet, x as i32, y as i32))
        ).expect("invalid unit?"));
        
        let blue_id = self.next_id();
        self.initial_units.push(Unit::new(
            blue_id, Team::Blue, UnitType::Worker, 0,
            Location::OnMap(MapLocation::new(self.planet, o_x as i32, o_y as i32))
        ).expect("invalid unit?"));

        Ok(())
    }

    pub fn set_impassable(&mut self, x: usize, y: usize) -> Result<(), Error> {
        self.ok_if_on_map(x, y)?;
        if self.contains_unit(x, y) {
            Err(GameError::LocationNotEmpty)?;
        }

        let (o_x, o_y) = self.reflect(x, y);
        self.is_passable_terrain[y][x] = false;
        self.is_passable_terrain[o_y][o_x] = false;
        Ok(())
    }

    pub fn set_karbonite(&mut self, x: usize, y: usize, amount: u32) -> Result<(), Error> {
        self.ok_if_on_map(x, y)?;
        if self.planet == Planet::Mars {
            Err(GameError::InvalidMapObject)?;
        }
        if amount < MAP_KARBONITE_MIN || amount > MAP_KARBONITE_MAX {
            Err(GameError::InvalidMapObject)?;
        }

        let (o_x, o_y) = self.reflect(x, y);
        self.initial_karbonite[y][x] = amount;
        self.initial_karbonite[o_y][o_x] = amount;
        Ok(())
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
    pattern: FnvHashMap<Rounds, AsteroidStrike>,
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
        let round_gen = Range::new(ASTEROID_ROUND_MIN, ASTEROID_ROUND_MAX);
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
    pub fn validate(&self) -> Result<(), Error> {
        // The Karbonite on each asteroid is in the range
        // [ASTEROID_KARB_MIN, ASTEROID_KARB_MAX], inclusive.
        for (round, asteroid) in self.pattern.clone() {
            if round < 1 || round > ROUND_LIMIT {
                Err(GameError::InvalidMapObject)?
            }
            if asteroid.karbonite < ASTEROID_KARB_MIN ||
               asteroid.karbonite > ASTEROID_KARB_MAX {
                Err(GameError::InvalidMapObject)?
            }
            if asteroid.location.planet != Planet::Mars {
                Err(GameError::InvalidMapObject)?
            }
        }

        // An asteroid strikes every [ASTEROID_ROUND_MIN,
        // ASTEROID_ROUND_MAX] rounds, inclusive.
        let mut rounds: Vec<&Rounds> = self.pattern.keys().collect();
        rounds.sort();
        if *rounds[0] > ASTEROID_ROUND_MAX {
            Err(GameError::InvalidMapObject)?
        }
        if ROUND_LIMIT - rounds[rounds.len() - 1] > ASTEROID_ROUND_MAX {
            Err(GameError::InvalidMapObject)?
        }
        for i in 0..rounds.len() - 1 {
            let diff = rounds[i + 1] - rounds[i];
            if diff < ASTEROID_ROUND_MIN || diff > ASTEROID_ROUND_MAX {
                Err(GameError::InvalidMapObject)?
            }
        }
        Ok(())
    }

    /// Whether there is an asteroid strike at the given round.
    pub fn has_asteroid(&self, round: Rounds) -> bool {
        self.pattern.get(&round).is_some()
    }

    /// Get the asteroid strike at the given round.
    ///
    /// * NullValue - There is no asteroid strike at this round.
    pub fn asteroid(&self, round: Rounds) -> Result<&AsteroidStrike, Error> {
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
    pub fn validate(&self) -> Result<(), Error> {
        // The flight times are within [ORIBIT_FLIGHT_MIN, ORBIT_FLIGHT_MAX].
        if self.center - self.amplitude < ORBIT_FLIGHT_MIN {
            Err(GameError::InvalidMapObject)?
        }
        if self.center + self.amplitude > ORBIT_FLIGHT_MAX {
            Err(GameError::InvalidMapObject)?
        }
        Ok(())
    }

    /// Get the duration of flight if the rocket were to take off from either
    /// planet on the given round.
    pub fn duration(&self, round: Rounds) -> Rounds {
        let arg = 2. * f32::consts::PI / self.period_s as f32 * round as f32;
        let sin = ((self.amplitude_s as f32) * f32::sin(arg)) as i32;
        (sin + self.center_s) as Rounds
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn insert_and_err(pattern: &FnvHashMap<Rounds, AsteroidStrike>,
                      round: Rounds, karbonite: u32, location: MapLocation) {
        let mut invalid = pattern.clone();
        invalid.insert(round, AsteroidStrike::new(karbonite, location));
        assert_err!(AsteroidPattern::new(&invalid).validate(), GameError::InvalidMapObject);
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
            AsteroidPattern::random(seed, mars_map).validate().is_ok();
        }

        // Generate an asteroid pattern from a map.
        let asteroid_map = AsteroidPattern::random(0, mars_map).asteroid_map();
        let asteroids = AsteroidPattern::new(&asteroid_map);
        assert!(asteroids.validate().is_ok());

        let mut asteroid_map = gen_asteroid_map(1, ASTEROID_ROUND_MAX);
        assert!(AsteroidPattern::new(&asteroid_map).validate().is_ok());

        // Invalid asteroid strikes.
        let loc = MapLocation::new(Planet::Mars, 0, 0);
        insert_and_err(&asteroid_map, 0, ASTEROID_KARB_MIN, loc);
        insert_and_err(&asteroid_map, ROUND_LIMIT + 1, ASTEROID_KARB_MIN, loc);
        insert_and_err(&asteroid_map, 1, ASTEROID_KARB_MIN - 1, loc);
        insert_and_err(&asteroid_map, 1, ASTEROID_KARB_MAX + 1, loc);
        insert_and_err(&asteroid_map, 1, ASTEROID_KARB_MAX, MapLocation::new(Planet::Earth, 0, 0));

        // Invalid strike pattern.
        insert_and_err(&asteroid_map, 2, ASTEROID_KARB_MIN, loc);
        asteroid_map.remove(&1);
        assert_err!(AsteroidPattern::new(&asteroid_map).validate(), GameError::InvalidMapObject);
    }

    #[test]
    fn validate_orbit() {
        assert_err!(OrbitPattern::new(150, 200, 200).validate(), GameError::InvalidMapObject);
        assert_err!(OrbitPattern::new(150, 200, 300).validate(), GameError::InvalidMapObject);
        assert!(OrbitPattern::new(75, 200, 125).validate().is_ok());
    }

    #[test]
    fn test_asteroid() {
        let asteroid_map = gen_asteroid_map(ASTEROID_ROUND_MAX, ASTEROID_ROUND_MAX);
        let asteroids = AsteroidPattern::new(&asteroid_map);
        for round in 1..ROUND_LIMIT {
            if round % ASTEROID_ROUND_MAX == 0 {
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
