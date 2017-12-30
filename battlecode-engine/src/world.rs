//! The core battlecode engine.

use fnv::FnvHashMap;

use super::constants::*;
use super::schema::Delta;
use super::id_generator::IDGenerator;
use super::location::*;
use super::map::*;
use super::unit::*;
use super::unit::UnitType as Branch;
use super::research;
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

    /// The current state of research.
    research: ResearchInfo,
}

impl TeamInfo {
    /// Construct a team with the default properties.
    pub fn new(team: Team, seed: u32) -> TeamInfo {
        TeamInfo {
            team: team,
            id_generator: IDGenerator::new(team, seed),
            team_arrays: FnvHashMap::default(),
            research: ResearchInfo::new(),
        }
    }

    pub fn get_level(&self, unit_type: &UnitType) -> Level {
        self.get_research().get_level(&unit_type)
    }

    pub fn get_research(&self) -> &ResearchInfo {
        &self.research
    }

    pub fn get_research_mut(&mut self) -> &mut ResearchInfo {
        &mut self.research
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

impl Player {
    /// Constructs a new player.
    pub fn new(team: Team, planet: Planet) -> Player {
        Player { team: team, planet: planet }
    }
}

/// The full world of the Battlecode game.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameWorld {
    /// The current round, starting at 1.
    round: Rounds,

    /// The player whose turn it is.
    player_to_move: Player,

    /// All the units on the map.
    units: FnvHashMap<UnitID, Unit>,

    /// All the units on the map, by location.
    units_by_loc: FnvHashMap<MapLocation, UnitID>,

    /// Rocket landings. Maps round numbers to a list of rockets
    /// landing on the given round.
    rocket_landings: FnvHashMap<Rounds, Vec<(UnitID, MapLocation)>>,

    /// The weather patterns.
    weather: WeatherPattern,

    /// The state of each planet.
    planet_states: FnvHashMap<Planet, PlanetInfo>,

    /// The state of each team.
    team_states: FnvHashMap<Team, TeamInfo>,
}

impl GameWorld {
    /// Initialize a new game world with maps from both planets.
    ///
    /// * GameError::InvalidMapObject - the map is invalid, check the specs.
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
    fn test_world() -> GameWorld {
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

    fn get_planet_info(&self, planet: Planet) -> &PlanetInfo {
        if let Some(planet_info) = self.planet_states.get(&planet) {
            planet_info
        } else {
            unreachable!();
        }
    }
    
    fn get_planet_info_mut(&mut self, planet: Planet) -> &mut PlanetInfo {
        if let Some(planet_info) = self.planet_states.get_mut(&planet) {
            planet_info
        } else {
            unreachable!();
        }
    }

    fn get_team_info(&self, team: Team) -> &TeamInfo {
        if let Some(team_info) = self.team_states.get(&team) {
            team_info
        } else {
            unreachable!();
        }
    }

    fn get_team_info_mut(&mut self, team: Team) -> &mut TeamInfo {
        if let Some(team_info) = self.team_states.get_mut(&team) {
            team_info
        } else {
            unreachable!();
        }
    }

    fn get_unit(&self, id: UnitID) -> Result<&Unit, Error> {
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
    fn place_unit(&mut self, id: UnitID) -> Result<(), Error> {
        let location = self.get_unit(id)?.location().unwrap();
        if self.is_occupiable(location)? {
            self.units_by_loc.insert(location, id);
            Ok(())
        } else {
            Err(GameError::InternalEngineError)?
        }
    }

    /// Creates and inserts a new unit into the game world, so that it can be
    /// referenced by ID.
    fn create_unit(&mut self, team: Team, location: MapLocation,
                       unit_type: UnitType) -> Result<UnitID, Error> {
        let id = self.get_team_info_mut(team).id_generator.next_id();
        let level = self.get_team_info(team).get_level(&unit_type);
        let unit = Unit::new(id, team, unit_type, level, location)?;

        self.units.insert(unit.id(), unit);
        self.place_unit(id)?;
        Ok(id)
    }

    /// Removes a unit from the map. Assumes the unit is on the map.
    fn remove_unit(&mut self, id: UnitID) -> Result<(), Error> {
        let location = {
            let unit = self.get_unit_mut(id)?;
            let location = unit.location();
            // If location is None, then the unit was already removed.
            location.ok_or(GameError::InternalEngineError)?
        };
        self.units_by_loc.remove(&location);
        Ok(())
    }

    /// Deletes a unit. Unit should be removed from map first.
    fn delete_unit(&mut self, id: UnitID) {
        self.units.remove(&id);
    }

    /// Destroys a unit.
    fn destroy_unit(&mut self, id: UnitID) -> Result<(), Error> {
        if self.get_unit(id)?.location().is_some() {
            self.remove_unit(id)?;
            self.get_unit_mut(id)?.destroy();
        }
        if self.get_unit(id)?.unit_type() == UnitType::Rocket {
            let units_to_destroy = self.get_unit_mut(id)?.garrisoned_units()?;
            for utd_id in units_to_destroy.iter() {
                self.destroy_unit(*utd_id)?;
            }
        }
        self.delete_unit(id);
        Ok(())
    }

    // ************************************************************************
    // ************************* LOCATION METHODS *****************************
    // ************************************************************************

    /// Whether the location is clear for a unit to occupy, either by movement
    /// or by construction.
    ///
    /// * GameError::InvalidLocation - the location is off the map.
    fn is_occupiable(&self, location: MapLocation) -> Result<bool, Error> {
        let planet_info = &self.get_planet_info(location.planet);
        Ok(planet_info.map.is_passable_terrain_at(location)? &&
            !self.units_by_loc.contains_key(&location))
    }

    /// Whether the robot can move in the given direction. To move, the
    /// robots' movement heat must be sufficiently low, and the location must
    /// be occupiable and on the map.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist.
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a robot.
    pub fn can_move(&self, id: UnitID, direction: Direction) -> Result<bool, Error> {
        // TODO: should return false if off the map
        let unit = self.get_unit(id)?;
        if let Some(location) = unit.location() {
            Ok(unit.is_move_ready()? && self.is_occupiable(location.add(direction))?)
        } else {
            Ok(false)
        }
    }

    /// Moves the robot in the given direction.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist.
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a robot.
    /// * GameError::InvalidAction - the robot cannot move.
    pub fn move_robot(&mut self, id: UnitID, direction: Direction) -> Result<(), Error> {
        let dest = self.get_unit(id)?.location().ok_or(GameError::InvalidAction)?.add(direction);
        if self.can_move(id, direction)? {
            self.remove_unit(id)?;
            self.get_unit_mut(id)?.move_to(Some(dest))?;
            self.place_unit(id)?;
            Ok(())
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    // ************************************************************************
    // *************************** ATTACK METHODS *****************************
    // ************************************************************************

    /// Deals damage to any unit in the target square, potentially destroying it.
    fn damage_location(&mut self, location: MapLocation, damage: i32) -> Result<(), Error> {
        let id = if let Some(id) = self.units_by_loc.get(&location) {
            *id
        } else {
            return Ok(());
        };

        let should_destroy_unit = self.get_unit_mut(id)?.take_damage(damage);
        if should_destroy_unit {
            self.destroy_unit(id)?;
        }
        Ok(())
    }

    // ************************************************************************
    // *********************** COMMUNICATION METHODS **************************
    // ************************************************************************

    // ************************************************************************
    // ************************** WEATHER METHODS *****************************
    // ************************************************************************

    fn process_asteroids(&mut self) {
        if self.weather.asteroids.get_asteroid(self.round).is_some() {
            let (location, karbonite) = {
                let asteroid = self.weather.asteroids.get_asteroid(self.round).unwrap();
                (asteroid.location, asteroid.karbonite)
            };
            let planet_info = self.get_planet_info_mut(location.planet);
            planet_info.karbonite[location.y as usize][location.x as usize] += karbonite;
        }
    }

    // ************************************************************************
    // ************************* RESEARCH METHODS *****************************
    // ************************************************************************

    /// Returns research info of the current player.
    fn get_research(&self) -> ResearchInfo {
        let team = self.player_to_move.team;
        self.get_team_info(team).get_research().clone()
    }

    /// Returns mutable research info of the current player.
    fn get_research_mut(&mut self) -> &mut ResearchInfo {
        let team = self.player_to_move.team;
        self.get_team_info_mut(team).get_research_mut()
    }

    /// Returns the maximum level of the research branch.
    pub fn get_research_max_level(branch: &Branch) -> Level {
        research::get_max_level(branch)
    }

    /// Returns: the cost of a level, in rounds, of a research branch.
    ///
    /// * GameError::InvalidResearchLevel - the research level for this branch is invalid.
    pub fn get_research_cost(branch: &Branch, level: Level) -> Result<Rounds, Error> {
        research::get_cost(branch, level)
    }

    /// Returns the current level of the research branch.
    pub fn get_research_level(&self, branch: &Branch) -> Level {
        self.get_research().get_level(branch)
    }

    /// Returns the research queue, where the front of the queue is at the
    /// beginning of the list.
    pub fn get_research_queue(&self) -> Vec<Branch> {
        self.get_research().get_queue()
    }

    /// Returns the next branch to be researched, which is the branch at the
    /// front of the research queue. Returns None if the queue is empty.
    pub fn get_next_in_research_queue(&self) -> Option<Branch> {
        self.get_research().get_next_in_queue()
    }

    /// Returns the number of rounds left until the upgrade at the front of the
    /// research queue is applied, or None if the queue is empty.
    pub fn get_research_rounds_left(&self) -> Option<Rounds> {
        self.get_research().get_rounds_left()
    }

    /// Resets the research queue to be empty. Returns true if the queue was
    /// not empty before, and false otherwise.
    pub fn reset_research_queue(&mut self) -> bool {
        self.get_research_mut().reset_queue()
    }

    /// Adds a branch to the back of the queue, if it is a valid upgrade, and
    /// starts research if it is the first in the queue.
    ///
    /// Returns whether the branch was successfully added.
    pub fn add_to_research_queue(&mut self, branch: &Branch) -> bool {
        self.get_research_mut().add_to_queue(branch)
    }

    /// Update the current research and process any completed upgrades.
    fn process_research(&mut self, team: Team) -> Result<(), Error> {
        if let Some(branch) = self.get_team_info_mut(team).get_research_mut().update()? {
            for (_, unit) in self.units.iter_mut() {
                if unit.unit_type() == branch {
                    unit.research()?;
                }
            }
            Ok(())
        } else {
            Ok(())
        }
    }

    // ************************************************************************
    // *************************** WORKER METHODS *****************************
    // ************************************************************************

    // ************************************************************************
    // *************************** KNIGHT METHODS *****************************
    // ************************************************************************
    
    // ************************************************************************
    // *************************** RANGER METHODS *****************************
    // ************************************************************************
    
    // ************************************************************************
    // **************************** MAGE METHODS ******************************
    // ************************************************************************
    
    // ************************************************************************
    // *************************** HEALER METHODS *****************************
    // ************************************************************************

    // ************************************************************************
    // ************************** FACTORY METHODS *****************************
    // ************************************************************************

    // ************************************************************************
    // *************************** ROCKET METHODS *****************************
    // ************************************************************************

    /// Whether the robot can garrison inside the rocket. The robot must be
    /// ready to move and adjacent to the rocket. The rocket must be on the
    /// same team and have enough space.
    ///
    /// * GameError::NoSuchUnit - a unit does not exist.
    /// * GameError::TeamNotAllowed - the rocket is not on the current player's team.
    /// * GameError::InappropriateUnitType - the robot or rocket are the wrong type.
    pub fn can_garrison(&self, robot_id: UnitID, rocket_id: UnitID)
                        -> Result<bool, Error> {
        let robot = self.get_unit(robot_id)?;
        let rocket = self.get_unit(rocket_id)?;
        rocket.can_garrison(robot)
    }

    /// Moves the robot into the garrison of the rocket.
    ///
    /// * GameError::NoSuchUnit - a unit does not exist.
    /// * GameError::TeamNotAllowed - the robot or rocket is not on the current player's team.
    /// * GameError::InappropriateUnitType - the robot or rocket are the wrong type.
    /// * GameError::InvalidAction - the robot cannot garrison inside the rocket.
    pub fn garrison(&mut self, robot_id: UnitID, rocket_id: UnitID)
                    -> Result<(), Error> {
        if self.can_garrison(robot_id, rocket_id)? {
            self.get_unit_mut(rocket_id)?.garrison(robot_id)?;
            self.remove_unit(robot_id)?;
            self.get_unit_mut(robot_id)?.move_to(None)?;
            Ok(())
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    /// Tests whether the given rocket is able to degarrison a unit in the
    /// given direction. There must be space in that direction, and the unit
    /// must be ready to move.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist.
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a rocket.
    /// * GameError::InvalidLocation - the location is off the map.
    pub fn can_degarrison(&self, rocket_id: UnitID, direction: Direction)
                          -> Result<bool, Error> {
        let rocket = self.get_unit(rocket_id)?;
        if rocket.can_degarrison_unit()? {
            let robot = self.get_unit(rocket.garrisoned_units()?[0])?;
            let loc = rocket.location().unwrap().add(direction);
            Ok(self.is_occupiable(loc)? && robot.is_move_ready()?)
        } else {
            Ok(false)
        }
    }

    /// Degarrisons a robot from the garrison of the specified rocket. Robots
    /// are degarrisoned in the order they garrisoned.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist.
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a rocket.
    /// * GameError::InvalidLocation - the location is off the map.
    /// * GameError::InvalidAction - the rocket cannot degarrison a unit.
    pub fn degarrison(&mut self, rocket_id: UnitID, direction: Direction)
                      -> Result<(), Error> {
        if self.can_degarrison(rocket_id, direction)? {
            let (robot_id, rocket_loc) = {
                let rocket = self.get_unit_mut(rocket_id)?;
                (rocket.degarrison_unit()?, rocket.location().unwrap())
            };
            let robot_loc = rocket_loc.add(direction);
            self.get_unit_mut(robot_id)?.move_to(Some(robot_loc))?;
            self.place_unit(robot_id)
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    /// Whether the rocket can launch into space. The rocket can launch if the
    /// it has never been used before.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist.
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a rocket.
    pub fn can_launch_rocket(&mut self, id: UnitID, destination: MapLocation)
                             -> Result<bool, Error> {
        let rocket = self.get_unit(id)?;
        if rocket.is_rocket_used()? {
            return Ok(false);
        }
        let map = &self.get_planet_info(destination.planet).map;
        Ok(map.on_map(destination) && map.is_passable_terrain_at(destination)?)
    }

    /// Launches the rocket into space. If the destination is not on the map of
    /// the other planet, the rocket flies off, never to be seen again.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist.
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a rocket.
    /// * GameError::InvalidAction - the rocket cannot launch.
    pub fn launch_rocket(&mut self, id: UnitID, destination: MapLocation)
                         -> Result<(), Error> {
        if self.can_launch_rocket(id, destination)? {
            let takeoff_loc = self.get_unit(id)?.location().unwrap();
            self.remove_unit(id)?;
            self.get_unit_mut(id)?.launch_rocket()?;
            let landing_round = self.round + self.weather.orbit.get_duration(self.round as i32) as u32;
            if self.rocket_landings.contains_key(&landing_round) {
                self.rocket_landings.get_mut(&landing_round).unwrap().push((id, destination));
            } else {
                self.rocket_landings.insert(landing_round, vec![(id, destination)]);
            }
            for dir in Direction::all() {
                self.damage_location(takeoff_loc.add(dir), ROCKET_BLAST_DAMAGE)?;
            }
            Ok(())
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    /// Lands the rocket, damaging the units in adjacent squares. The rocket
    /// is destroyed if it lands on a factory, rocket, or impassable terrain.
    fn land_rocket(&mut self, id: UnitID, destination: MapLocation)
                   -> Result<(), Error> {
        if self.units_by_loc.contains_key(&destination) {
            let victim_id = *self.units_by_loc.get(&destination).unwrap();
            let should_destroy_rocket = match self.get_unit(victim_id)?.unit_type() {
                UnitType::Rocket => true,
                UnitType::Factory => true,
                _ => false,
            };
            if should_destroy_rocket {
                self.destroy_unit(id)?;
            }
            self.destroy_unit(victim_id)?;
        } else {
            self.get_unit_mut(id)?.land_rocket(destination)?;
            self.place_unit(id)?;
        }

        for dir in Direction::all() {
            self.damage_location(destination.add(dir), ROCKET_BLAST_DAMAGE)?;
        }

        Ok(())
    }

    // ************************************************************************
    // ****************************** GAME LOOP *******************************
    // ************************************************************************

    /// Updates the current player in the game. If a round of four turns has
    /// finished, also processes the end of the round. This includes updating
    /// unit cooldowns, rocket landings, asteroid strikes, research, etc.
    ///
    /// * GameError::InternalEngineError - something happened here...
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

    fn next_round(&mut self) -> Result<(), Error> {
        self.round += 1;

        // Update unit cooldowns.
        for unit in &mut self.units.values_mut() {
            unit.next_round();
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
        self.process_asteroids();

        // Update the current research and process any completed upgrades.
        self.process_research(Team::Red)?;
        self.process_research(Team::Blue)?;

        Ok(())
    }

    pub fn apply(&mut self, delta: Delta) -> Result<(), Error> {
        match delta {
            Delta::EndTurn => self.next_turn(),
            Delta::Move{id, direction} => self.move_robot(id, direction),
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
        assert_eq!(unit_a.location().unwrap(), loc_a);
        assert_eq!(unit_b.location().unwrap(), loc_b);
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
        world.move_robot(a, Direction::Northeast).unwrap();

        // A is now one square north of B. B cannot move north to
        // A's new location, but can move west to A's old location.
        assert![!world.can_move(b, Direction::North).unwrap()];
        assert![world.can_move(b, Direction::West).unwrap()];
        world.move_robot(b, Direction::West).unwrap();

        // A cannot move again until its cooldowns are reset.
        assert![!world.can_move(a, Direction::South).unwrap()];
        assert![world.move_robot(a, Direction::South).is_err()];
        assert![world.next_round().is_ok()];

        // Finally, let's test that A cannot move back to its old square.
        assert![!world.can_move(a, Direction::Southwest).unwrap()];
        assert![world.can_move(a, Direction::South).unwrap()];
        world.move_robot(a, Direction::South).unwrap();
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
            bystanders.push(world.create_unit(Team::Red, earth_loc.add(direction), UnitType::Knight).unwrap());
            bystanders.push(world.create_unit(Team::Red, mars_loc.add(direction), UnitType::Knight).unwrap());
            direction = direction.rotate_right();
        }

        // Launch the rocket, and force land it.
        world.launch_rocket(rocket, mars_loc).unwrap();
        assert![world.get_unit(rocket).unwrap().location().is_none()];
        world.land_rocket(rocket, mars_loc).unwrap();
        assert_eq![world.get_unit(rocket).unwrap().location().unwrap(), mars_loc];
        let damaged_knight_health = 200;
        for id in bystanders.iter() {
            assert_eq![world.get_unit(*id).unwrap().health(), damaged_knight_health];
        }
    }

    #[test]
    fn test_rocket_failure() {
        // Create the game world.
        let mut world = GameWorld::test_world();
        let earth_loc_a = MapLocation::new(Planet::Earth, 0, 0);
        let earth_loc_b = MapLocation::new(Planet::Earth, 0, 2);
        let mars_loc_off_map = MapLocation::new(Planet::Mars, 10000, 10000);
        let mars_loc_impassable = MapLocation::new(Planet::Mars, 0, 0);
        world.get_planet_info_mut(Planet::Mars).map.is_passable_terrain[0][0] = false;
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

    #[test]
    fn test_rocket_garrison() {
        // Create the game world and the rocket for this test.
        let mut world = GameWorld::test_world();
        let takeoff_loc = MapLocation::new(Planet::Earth, 10, 10);        
        let rocket = world.create_unit(Team::Red, takeoff_loc, UnitType::Rocket).unwrap();

        // Correct garrisoning.
        let valid_boarder = world.create_unit(Team::Red, takeoff_loc.add(Direction::North), UnitType::Knight).unwrap();
        assert![world.garrison(valid_boarder, rocket).is_ok()];
        assert_eq![world.get_unit(valid_boarder).unwrap().location(), None];

        // Boarding fails when too far from the rocket.
        let invalid_boarder_too_far = world.create_unit(Team::Red, takeoff_loc.add(Direction::North).add(Direction::North), UnitType::Knight).unwrap();
        assert![world.garrison(invalid_boarder_too_far, rocket).is_err()];

        // Boarding fails when the robot has already moved.
        // uncomment when movement cooldown is implemented
        //assert![world.move_robot(invalid_boarder_too_far, Direction::South).is_ok()];
        //let invalid_boarder_already_moved = invalid_boarder_too_far;
        //assert![!world.get_unit(invalid_boarder_already_moved).unwrap().is_move_ready()];
        //assert![world.garrison(invalid_boarder_already_moved, rocket).is_err()];

        // Factories and rockets cannot board rockets.
        let invalid_boarder_factory = world.create_unit(Team::Red, takeoff_loc.add(Direction::Southeast), UnitType::Factory).unwrap();
        assert![world.garrison(invalid_boarder_factory, rocket).is_err()];
        let invalid_boarder_rocket = world.create_unit(Team::Red, takeoff_loc.add(Direction::South), UnitType::Rocket).unwrap();
        assert![world.garrison(invalid_boarder_rocket, rocket).is_err()];

        // Rockets can be loaded up to their capacity...
        for _ in 1..8 {
            let valid_extra_boarder = world.create_unit(Team::Red, takeoff_loc.add(Direction::East), UnitType::Knight).unwrap();
            assert![world.garrison(valid_extra_boarder, rocket).is_ok()];
        }

        // ... but not beyond their capacity.
        let invalid_boarder_rocket_full = world.create_unit(Team::Red, takeoff_loc.add(Direction::East), UnitType::Knight).unwrap();
        assert![world.garrison(invalid_boarder_rocket_full, rocket).is_err()];

        // A unit should not be able to board another team's rocket.
        let blue_takeoff_loc = MapLocation::new(Planet::Earth, 5, 5);
        let blue_rocket = world.create_unit(Team::Blue, blue_takeoff_loc, UnitType::Rocket).unwrap();
        let invalid_boarder_wrong_team = world.create_unit(Team::Red, blue_takeoff_loc.add(Direction::North), UnitType::Knight).unwrap();
        assert![world.garrison(invalid_boarder_wrong_team, blue_rocket).is_err()];
    }

    #[test]
    fn test_rocket_degarrison() {
        // Create the game world and the rocket for this test.
        let mut world = GameWorld::test_world();
        let takeoff_loc = MapLocation::new(Planet::Earth, 10, 10);        
        let rocket = world.create_unit(Team::Red, takeoff_loc, UnitType::Rocket).unwrap();
        
        // Load the rocket with robots.
        for _ in 0..2 {
            let robot = world.create_unit(Team::Red, takeoff_loc.add(Direction::North), UnitType::Knight).unwrap();
            assert![world.garrison(robot, rocket).is_ok()];
        }

        // Fly the rocket to Mars.
        let landing_loc = MapLocation::new(Planet::Mars, 10, 10);
        assert![world.launch_rocket(rocket, landing_loc).is_ok()];
        assert![world.land_rocket(rocket, landing_loc).is_ok()];

        // Cannot degarrison in the same round.
        assert![world.degarrison(rocket, Direction::North).is_err()];

        // Correct degarrisoning.
        world.next_round().unwrap();
        assert![world.degarrison(rocket, Direction::North).is_ok()];

        // Cannot degarrison into an occupied square.
        assert![world.degarrison(rocket, Direction::North).is_err()];

        // Cannot degarrison into an impassable square.
        world.get_planet_info_mut(Planet::Mars).map.is_passable_terrain[10][11] = false;
        assert![world.degarrison(rocket, Direction::East).is_err()];

        // Correct degarrisoning, again.
        assert![world.degarrison(rocket, Direction::South).is_ok()];

        // Cannot degarrison an empty rocket.
        assert![world.degarrison(rocket, Direction::West).is_err()];
    }
}
