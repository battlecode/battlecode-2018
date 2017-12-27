//! The core battlecode engine.

use fnv::FnvHashMap;
use std::cmp;

use super::constants::*;
use super::schema::Delta;
use super::id_generator::IDGenerator;
use super::location::*;
use super::map::*;
use super::unit::*;
use super::research::*;
use super::error::GameError;
use failure::Error;

// A round consists of a turn from each player.
pub type Rounds = u32;

/// There are two teams in Battlecode: Red and Blue.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Team {
    Red,
    Blue,
}

/// The state for one of the planets in a game. Stores neutral info like the
/// planet's original map and the current karbonite deposits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanetInfo {
    /// The map of the planet.
    map: PlanetMap,

    /// The amount of Karbonite deposited on the specified square.
    ///
    /// Stored as a two-dimensional array, where the first index 
    /// represents a square's y-coordinate, and the second index its 
    /// x-coordinate. These coordinates are *relative to the origin*.
    karbonite: Vec<Vec<u32>>,
}

impl PlanetInfo {
    /// Construct a planet with the given map, where the current karbonite
    /// deposits are initialized with the map's initial deposits.
    pub fn new(map: PlanetMap) -> PlanetInfo {
        let karbonite = map.initial_karbonite.clone();
        PlanetInfo {
            map: map,
            karbonite: karbonite,
        }
    }

    pub fn test_planet_info(planet: Planet) -> PlanetInfo {
        PlanetInfo {
            map: PlanetMap::test_map(planet),
            karbonite: vec![vec![0; MAP_WIDTH_MAX]; MAP_HEIGHT_MAX],
        }
    }
}

/// A team-shared communication array.
pub type TeamArray = Vec<u8>;

/// A history of communication arrays. Read from the back of the queue on the
/// current planet, and the front of the queue on the other planet.
pub type TeamArrayHistory = Vec<TeamArray>;

/// Persistent info specific to a single team. Teams are only able to access
/// the team info of their own team.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamInfo {
    /// Team identification.
    pub team: Team,

    /// Unit ID generator.
    id_generator: IDGenerator,

    /// Communication array histories for each planet.
    team_arrays: FnvHashMap<Planet, TeamArrayHistory>,

    /// Unit info of a given type at initialization.
    unit_infos: FnvHashMap<UnitType, UnitInfo>,

    /// The current state of research.
    research: ResearchInfo,
}

impl TeamInfo {
    /// Construct a team with the default properties.
    pub fn new(team: Team, seed: u32) -> TeamInfo {
        // Initialize default unit infos
        let mut unit_infos = FnvHashMap::default();
        for unit_type in UnitType::all() {
            let unit_info = unit_type.default();
            unit_infos.insert(unit_type, unit_info);
        }

        TeamInfo {
            team: team,
            id_generator: IDGenerator::new(team, seed),
            team_arrays: FnvHashMap::default(),
            unit_infos: unit_infos,
            research: ResearchInfo::new(),
        }
    }

    pub fn get_unit_info(&self, unit_type: UnitType) -> Result<&UnitInfo, Error> {
        if let Some(unit_info) = self.unit_infos.get(&unit_type) {
            Ok(unit_info)
        } else {
            Err(GameError::NoSuchUnitType)?
        }
    }

    pub fn get_unit_info_mut(&mut self, unit_type: UnitType)
                             -> Result<&mut UnitInfo, Error> {
        if let Some(unit_info) = self.unit_infos.get_mut(&unit_type) {
            Ok(unit_info)
        } else {
            Err(GameError::NoSuchUnitType)?
        }
    }
}

/// A player represents a program controlling some group of units.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Player {
    /// The team of this player.
    pub team: Team,

    /// The planet for this player. Each team disjointly controls the robots on each planet.
    pub planet: Planet,
}

/// The full world of the Battlecode game.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameWorld {
    /// The current round, starting at 1.
    pub round: Rounds,

    /// The player whose turn it is.
    pub player_to_move: Player,

    /// All the units on the map.
    units: FnvHashMap<UnitID, Unit>,

    /// All the units on the map, by location.
    units_by_loc: FnvHashMap<MapLocation, UnitID>,

    /// Rocket landings. Maps round numbers to a list of rockets
    /// landing on the given round.
    rocket_landings: FnvHashMap<u32, Vec<(UnitID, MapLocation)>>,

    /// The weather patterns.
    pub weather: WeatherPattern,

    /// The state of each planet.
    pub planet_states: FnvHashMap<Planet, PlanetInfo>,

    /// The state of each team.
    pub team_states: FnvHashMap<Team, TeamInfo>,
}

impl GameWorld {
    /// Initialize a new game world with maps from both planets.
    pub fn new(map: GameMap) -> Result<GameWorld, Error> {
        map.validate()?;

        let mut planet_states = FnvHashMap::default();
        planet_states.insert(Planet::Earth, PlanetInfo::new(map.earth_map));
        planet_states.insert(Planet::Mars, PlanetInfo::new(map.mars_map));

        let mut team_states = FnvHashMap::default();
        team_states.insert(Team::Red, TeamInfo::new(Team::Red, map.seed));
        team_states.insert(Team::Blue, TeamInfo::new(Team::Blue, map.seed));

        Ok(GameWorld {
            round: 1,
            player_to_move: Player { team: Team::Red, planet: Planet::Earth },
            units: FnvHashMap::default(),
            units_by_loc: FnvHashMap::default(),
            rocket_landings: FnvHashMap::default(),
            weather: map.weather,
            planet_states: planet_states,
            team_states: team_states,
        })
    }

    /// Creates a GameWorld for testing purposes.
    pub fn test_world() -> GameWorld {
        let mut planet_states = FnvHashMap::default();
        planet_states.insert(Planet::Earth, PlanetInfo::test_planet_info(Planet::Earth));
        planet_states.insert(Planet::Mars, PlanetInfo::test_planet_info(Planet::Mars));

        let mut team_states = FnvHashMap::default();
        team_states.insert(Team::Red, TeamInfo::new(Team::Red, 6147));
        team_states.insert(Team::Blue, TeamInfo::new(Team::Blue, 6147));

        let weather = WeatherPattern::new(AsteroidPattern::new(&FnvHashMap::default()),
                                          OrbitPattern::new(100, 100, 400));

        GameWorld {
            round: 1,
            player_to_move: Player { team: Team::Red, planet: Planet::Earth },
            units: FnvHashMap::default(),
            units_by_loc: FnvHashMap::default(),
            rocket_landings: FnvHashMap::default(),
            weather: weather,
            planet_states: planet_states,
            team_states: team_states,
        }
    }

    // ************************************************************************
    // ****************************** ACCESSORS *******************************
    // ************************************************************************

    fn get_planet_info(&self, planet: Planet) -> Result<&PlanetInfo, Error> {
        if let Some(planet_info) = self.planet_states.get(&planet) {
            Ok(planet_info)
        } else {
            Err(GameError::NoSuchPlanet)?
        }
    }
    
    pub fn get_planet_info_mut(&mut self, planet: Planet) -> Result<&mut PlanetInfo, Error> {
        if let Some(planet_info) = self.planet_states.get_mut(&planet) {
            Ok(planet_info)
        } else {
            Err(GameError::NoSuchPlanet)?
        }
    }

    fn get_team_info(&self, team: Team) -> Result<&TeamInfo, Error> {
        if let Some(team_info) = self.team_states.get(&team) {
            Ok(team_info)
        } else {
            Err(GameError::NoSuchTeam)?
        }
    }

    fn get_team_info_mut(&mut self, team: Team) -> Result<&mut TeamInfo, Error> {
        if let Some(team_info) = self.team_states.get_mut(&team) {
            Ok(team_info)
        } else {
            Err(GameError::NoSuchTeam)?
        }
    }

    pub fn get_unit(&self, id: UnitID) -> Result<&Unit, Error> {
        if let Some(unit) = self.units.get(&id) {
            Ok(unit)
        } else {
            Err(GameError::NoSuchUnit)?
        }
    }

    fn get_unit_mut(&mut self, id: UnitID) -> Result<&mut Unit, Error> {
        if let Some(unit) = self.units.get_mut(&id) {
            Ok(unit)
        } else {
            Err(GameError::NoSuchUnit)?
        }
    }

    // ************************************************************************
    // **************** UNIT CREATION / DESTRUCTION METHODS *******************
    // ************************************************************************

    /// Places a unit onto the map at the given location. Assumes the given square is occupiable.
    pub fn place_unit(&mut self, id: UnitID, location: MapLocation) -> Result<(), Error> {
        if self.is_occupiable(location)? {
            {
                let unit = self.get_unit_mut(id)?;
                if unit.location.is_some() {
                    // The unit has already been placed.
                    Err(GameError::InternalEngineError)?;
                }
                unit.location = Some(location);
            }
            self.units_by_loc.insert(location, id);
            Ok(())
        } else {
            Err(GameError::InternalEngineError)?
        }
    }

    /// Creates and inserts a new unit into the game world, so that it can be
    /// referenced by ID.
    pub fn create_unit(&mut self, team: Team, location: MapLocation,
                       unit_type: UnitType) -> Result<UnitID, Error> {
        let id = self.get_team_info_mut(team)?.id_generator.next_id();
        let unit_info = self.get_team_info(team)?.get_unit_info(unit_type)?.clone();
        let unit = Unit::new(id, team, unit_info);

        self.units.insert(unit.id, unit);
        self.place_unit(id, location)?;
        Ok(id)
    }

    /// Removes a unit from the map. Assumes the unit is on the map.
    pub fn remove_unit(&mut self, id: UnitID) -> Result<(), Error> {
        let location = {
            let unit = self.get_unit_mut(id)?;
            let location = unit.location;
            unit.location = None;
            // If location is None, then the unit was already removed.
            location.ok_or(GameError::InternalEngineError)?
        };
        self.units_by_loc.remove(&location);
        Ok(())
    }

    /// Deletes a unit. Unit should be removed from map first.
    pub fn delete_unit(&mut self, id: UnitID) {
        self.units.remove(&id);
    }

    /// Destroys a unit.
    pub fn destroy_unit(&mut self, id: UnitID) -> Result<(), Error> {
        if self.get_unit(id)?.location.is_some() {
            self.remove_unit(id)?;
        }
        let units_to_destroy = if let UnitInfo::Rocket(ref rocket_info) = self.get_unit(id)?.unit_info {
            rocket_info.garrisoned_units.clone()
        } else {
            vec![]
        };
        for utd_id in units_to_destroy.iter() {
            self.destroy_unit(*utd_id)?;
        }
        self.delete_unit(id);
        Ok(())
    }

    // ************************************************************************
    // ************************* LOCATION METHODS *****************************
    // ************************************************************************

    /// Returns whether the square is clear for a new unit to occupy, either by movement or by construction.
    pub fn is_occupiable(&self, location: MapLocation) -> Result<bool, Error> {
        let planet_info = &self.get_planet_info(location.planet)?;
        Ok(planet_info.map.is_passable_terrain[location.y as usize][location.x as usize] &&
            !self.units_by_loc.contains_key(&location))
    }

    /// Tests whether the given unit can move.
    pub fn can_move(&self, id: UnitID, direction: Direction) -> Result<bool, Error> {
        let unit = self.get_unit(id)?;
        if let Some(location) = unit.location {
            Ok(unit.is_move_ready() && self.is_occupiable(location.add(direction))?)
        } else {
            Ok(false)
        }
    }

    // Given that moving an unit comprises many edits to the GameWorld, it makes sense to define this here.
    pub fn move_unit(&mut self, id: UnitID, direction: Direction) -> Result<(), Error> {
        let dest = self.get_unit(id)?.location.ok_or(GameError::InvalidAction)?.add(direction);
        if self.can_move(id, direction)? {
            self.remove_unit(id)?;
            self.place_unit(id, dest)?;
            Ok(())
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    // ************************************************************************
    // *************************** ATTACK METHODS *****************************
    // ************************************************************************

    /// Deals damage to any unit in the target square, potentially destroying it.
    pub fn damage_location(&mut self, location: MapLocation, damage: u32) -> Result<(), Error> {
        let id = if let Some(id) = self.units_by_loc.get(&location) {
            *id
        } else {
            return Ok(());
        };

        let should_destroy_unit = {
            let unit = self.get_unit_mut(id)?;
            // TODO: Knight damage resistance??
            unit.health -= cmp::min(damage, unit.health);
            unit.health == 0
        };

        if should_destroy_unit {
            self.destroy_unit(id)?;
        }
        Ok(())
    }

    // ************************************************************************
    // ************************** RESEARCH METHODS ****************************
    // ************************************************************************

    // ************************************************************************
    // *************************** ROCKET METHODS *****************************
    // ************************************************************************

    pub fn launch_rocket(&mut self, id: UnitID, destination: MapLocation) -> Result<(), Error> {
        {
            let map = &self.get_planet_info(destination.planet)?.map;
            if !map.on_map(&destination) || !map.is_passable_terrain[destination.y as usize][destination.x as usize] {
                Err(GameError::InvalidAction)?;
            }
        }

        self.remove_unit(id)?;
        let landing_round = self.round + self.weather.orbit.get_duration(self.round as i32) as u32;
        if self.rocket_landings.contains_key(&landing_round) {
            self.rocket_landings.get_mut(&landing_round).unwrap().push((id, destination));
        } else {
            self.rocket_landings.insert(landing_round, vec![(id, destination)]);
        }
        Ok(())
    }

    pub fn land_rocket(&mut self, id: UnitID, destination: MapLocation) -> Result<(), Error> {
        if self.units_by_loc.contains_key(&destination) {
            let victim_id = *self.units_by_loc.get(&destination).unwrap();
            let should_destroy_rocket = match self.get_unit(victim_id)?.unit_info {
                UnitInfo::Rocket(_) => true,
                UnitInfo::Factory(_) => true,
                _ => false,
            };
            if should_destroy_rocket {
                self.destroy_unit(id)?;
            }
            self.destroy_unit(victim_id)?;
        } else {
            self.place_unit(id, destination)?;
        }

        let mut dir = Direction::North;
        for _ in 0..8 {
            self.damage_location(destination.add(dir), ROCKET_BLAST_DAMAGE)?;
            dir = dir.rotate_right();
        }

        Ok(())
    }

    // ************************************************************************
    // ****************************** GAME LOOP *******************************
    // ************************************************************************

    pub fn next_turn(&mut self) -> Result<(), Error> {
        self.player_to_move = match self.player_to_move {
            Player { team: Team::Red, planet: Planet::Earth } => Player { team: Team::Blue, planet: Planet::Earth},
            Player { team: Team::Blue, planet: Planet::Earth } => Player { team: Team::Red, planet: Planet::Mars},
            Player { team: Team::Red, planet: Planet::Mars } => Player { team: Team::Blue, planet: Planet::Mars},
            Player { team: Team::Blue, planet: Planet::Mars } => {
                // This is the last player to move, so we can advance to the next round.
                self.next_round()?;
                Player { team: Team::Red, planet: Planet::Earth}
            },
        };
        Ok(())
    }

    pub fn next_round(&mut self) -> Result<(), Error> {
        self.round += 1;

        // Update unit cooldowns.
        for unit in &mut self.units.values_mut() {
            unit.movement_heat -= cmp::min(HEAT_LOSS_PER_ROUND, unit.movement_heat);
            unit.attack_heat -= cmp::min(HEAT_LOSS_PER_ROUND, unit.attack_heat);
        }

        // Land rockets.
        let landings = if let Some(landings) = self.rocket_landings.get(&self.round) {
            landings.clone()
        } else {
            vec![]
        };
        for &(id, location) in landings.iter() {
            self.land_rocket(id, location)?;
        }

        // Process any potential asteroid impacts.
        if self.weather.asteroids.get_asteroid(self.round).is_some() {
            let (location, karbonite) = {
                let asteroid = self.weather.asteroids.get_asteroid(self.round).unwrap();
                (asteroid.location, asteroid.karbonite)
            };
            let planet_info = self.get_planet_info_mut(location.planet)?;
            planet_info.karbonite[location.y as usize][location.x as usize] += karbonite;
        }

        Ok(())
    }

    pub fn apply(&mut self, delta: Delta) -> Result<(), Error> {
        match delta {
            Delta::EndTurn => self.next_turn(),
            Delta::Move{id, direction} => self.move_unit(id, direction),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GameWorld;
    use super::Team;
    use super::super::unit::UnitID;
    use super::super::unit::UnitType;
    use super::super::location::*;

    #[test]
    fn test_unit_create() {
        // Create the game world, and create and add some robots.
        let mut world = GameWorld::test_world();
        let loc_a = MapLocation::new(Planet::Earth, 0, 1);
        let loc_b = MapLocation::new(Planet::Earth, 0, 2);
        let id_a = world.create_unit(Team::Red, loc_a, UnitType::Knight).unwrap();
        let id_b = world.create_unit(Team::Red, loc_b, UnitType::Knight).unwrap();

        // The robots have different IDs.
        assert_ne!(id_a, id_b);

        // Both robots exist and are at the right locations.
        let unit_a = world.get_unit(id_a).unwrap();
        let unit_b = world.get_unit(id_b).unwrap();
        assert_eq!(unit_a.location.unwrap(), loc_a);
        assert_eq!(unit_b.location.unwrap(), loc_b);
    }

    #[test]
    fn test_unit_move() {
        // Create the game world.
        let mut world = GameWorld::test_world();
        let loc_a = MapLocation::new(Planet::Earth, 5, 5);
        let loc_b = MapLocation::new(Planet::Earth, 6, 5);

        // Create and add some robots. B is one square east of A.
        let a = world.create_unit(Team::Red, loc_a, UnitType::Knight).unwrap();
        let b = world.create_unit(Team::Red, loc_b, UnitType::Knight).unwrap();

        // Robot A cannot move east, as this is where B is. However,
        // it can move northeast.
        assert![!world.can_move(a, Direction::East).unwrap()];
        assert![world.can_move(a, Direction::Northeast).unwrap()];
        world.move_unit(a, Direction::Northeast).unwrap();

        // A is now one square north of B. B cannot move north to
        // A's new location, but can move west to A's old location.
        assert![!world.can_move(b, Direction::North).unwrap()];
        assert![world.can_move(b, Direction::West).unwrap()];
        world.move_unit(b, Direction::West).unwrap();

        // Finally, let's test that A cannot move back to its old square.
        assert![!world.can_move(a, Direction::Southwest).unwrap()];
        assert![world.can_move(a, Direction::South).unwrap()];
        world.move_unit(a, Direction::South).unwrap();
    }

    #[test]
    fn test_rocket_success() {
        // Create the game world.
        let mut world = GameWorld::test_world();
        let earth_loc = MapLocation::new(Planet::Earth, 5, 5);
        let mars_loc = MapLocation::new(Planet::Mars, 5, 5);
        let rocket = world.create_unit(Team::Red, earth_loc, UnitType::Rocket).unwrap();

        // Create units around the target location.
        let mut bystanders: Vec<UnitID> = vec![];
        let mut direction = Direction::North;
        for _ in 0..8 {
            bystanders.push(world.create_unit(Team::Red, mars_loc.add(direction), UnitType::Knight).unwrap());
            direction = direction.rotate_right();
        }

        // Launch the rocket, and force land it.
        world.launch_rocket(rocket, mars_loc).unwrap();
        world.land_rocket(rocket, mars_loc).unwrap();
        assert_eq![world.get_unit(rocket).unwrap().location.unwrap(), mars_loc];
        let damaged_knight_health = 200;
        for id in bystanders.iter() {
            assert_eq![world.get_unit(*id).unwrap().health, damaged_knight_health];
        }
    }

    #[test]
    fn test_rocket_failure() {
        // Create the game world.
        let mut world = GameWorld::test_world();
        let earth_loc_a = MapLocation::new(Planet::Earth, 0, 0);
        let earth_loc_b = MapLocation::new(Planet::Earth, 0, 1);
        let mars_loc_off_map = MapLocation::new(Planet::Mars, 10000, 10000);
        let mars_loc_impassable = MapLocation::new(Planet::Mars, 0, 0);
        world.get_planet_info_mut(Planet::Mars).unwrap().map.is_passable_terrain[0][0] = false;
        let mars_loc_knight = MapLocation::new(Planet::Mars, 0, 1);
        let mars_loc_factory = MapLocation::new(Planet::Mars, 0, 2);
        let rocket_a = world.create_unit(Team::Red, earth_loc_a, UnitType::Rocket).unwrap();
        let rocket_b = world.create_unit(Team::Red, earth_loc_b, UnitType::Rocket).unwrap();
        let knight = world.create_unit(Team::Blue, mars_loc_knight, UnitType::Knight).unwrap();
        let factory = world.create_unit(Team::Blue, mars_loc_factory, UnitType::Factory).unwrap();

        // Failed launches.
        assert![world.launch_rocket(rocket_a, mars_loc_off_map).is_err()];
        assert![world.launch_rocket(rocket_a, mars_loc_impassable).is_err()];

        // Rocket landing on a robot should destroy the robot.
        world.launch_rocket(rocket_a, mars_loc_knight).unwrap();
        world.land_rocket(rocket_a, mars_loc_knight).unwrap();
        assert![world.get_unit(rocket_a).is_ok()];
        assert![world.get_unit(knight).is_err()];

        // Rocket landing on a factory should destroy both units.
        world.launch_rocket(rocket_b, mars_loc_factory).unwrap();
        world.land_rocket(rocket_b, mars_loc_factory).unwrap();
        assert![world.get_unit(rocket_b).is_err()];
        assert![world.get_unit(factory).is_err()];
    }
}
