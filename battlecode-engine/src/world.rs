//! The core battlecode engine.

use fnv::FnvHashMap;
use std::cmp;

use super::constants::*;
use super::schema::Delta;
use super::id_generator::IDGenerator;
use super::location::*;
use super::map::*;
use super::unit;
use super::research;
use super::error::GameError;
use failure::Error;

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

    pub fn test_planet_info() -> PlanetInfo {
        PlanetInfo {
            map: PlanetMap::test_map(Planet::Earth),
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
    unit_infos: FnvHashMap<unit::UnitType, unit::UnitInfo>,

    /// The current status of the team's research. The values defines the level
    /// of the research, where 0 represents no progress.
    research_status: FnvHashMap<research::Branch, u32>,

    /// Research branches queued to be researched, including the current branch.
    research_queue: Vec<research::Branch>,

    /// The number of rounds to go until the first branch in the research
    /// queue is finished. 0 if the research queue is empty.
    research_rounds_left: u32,
}

impl TeamInfo {
    /// Construct a team with the default properties.
    pub fn new(team: Team, seed: u32) -> TeamInfo {
        // Initialize default unit infos
        let mut unit_infos = FnvHashMap::default();
        let unit_types = vec![unit::UnitType::Worker, unit::UnitType::Knight,
                              unit::UnitType::Ranger, unit::UnitType::Mage,
                              unit::UnitType::Healer, unit::UnitType::Factory,
                              unit::UnitType::Rocket];
        for unit_type in unit_types {
            let unit_info = unit_type.default();
            unit_infos.insert(unit_type, unit_info);
        }

        TeamInfo {
            team: team,
            id_generator: IDGenerator::new(team, seed),
            team_arrays: FnvHashMap::default(),
            unit_infos: unit_infos,
            research_status: FnvHashMap::default(),
            research_queue: Vec::new(),
            research_rounds_left: 0,
        }
    }

    pub fn get_unit_info(&self, unit_type: unit::UnitType) -> Result<&unit::UnitInfo, Error> {
        if let Some(unit_info) = self.unit_infos.get(&unit_type) {
            Ok(unit_info)
        } else {
            Err(GameError::NoSuchUnitType)?
        }
    }

    pub fn get_unit_info_mut(&mut self, unit_type: unit::UnitType)
                             -> Result<&mut unit::UnitInfo, Error> {
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
    pub round: u32,

    /// The player whose turn it is.
    pub player_to_move: Player,

    /// All the units on the map.
    units: FnvHashMap<unit::UnitID, unit::Unit>,

    /// All the units on the map, by location.
    units_by_loc: FnvHashMap<MapLocation, unit::UnitID>,

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
            weather: map.weather,
            planet_states: planet_states,
            team_states: team_states,
        })
    }

    /// Creates a GameWorld for testing purposes.
    pub fn test_world() -> GameWorld {
        let mut planet_states = FnvHashMap::default();
        planet_states.insert(Planet::Earth, PlanetInfo::test_planet_info());
        planet_states.insert(Planet::Mars, PlanetInfo::test_planet_info());

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
            weather: weather,
            planet_states: planet_states,
            team_states: team_states,
        }
    }

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

    pub fn get_unit(&self, id: unit::UnitID) -> Result<&unit::Unit, Error> {
        if let Some(unit) = self.units.get(&id) {
            Ok(unit)
        } else {
            Err(GameError::NoSuchUnit)?
        }
    }

    fn get_unit_mut(&mut self, id: unit::UnitID) -> Result<&mut unit::Unit, Error> {
        if let Some(unit) = self.units.get_mut(&id) {
            Ok(unit)
        } else {
            Err(GameError::NoSuchUnit)?
        }
    }

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
        for (_, unit) in &mut self.units {
            unit.movement_heat -= cmp::min(HEAT_LOSS_PER_ROUND, unit.movement_heat);
            unit.attack_heat -= cmp::min(HEAT_LOSS_PER_ROUND, unit.attack_heat);
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

    /// Places a unit onto the map at the given location. Assumes the given square is occupiable.
    pub fn place_unit(&mut self, id: unit::UnitID, location: MapLocation) -> Result<(), Error> {
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
                       unit_type: unit::UnitType) -> Result<unit::UnitID, Error> {
        let id = self.get_team_info_mut(team)?.id_generator.next_id();
        let unit_info = self.get_team_info(team)?.get_unit_info(unit_type)?.clone();
        let unit = unit::Unit::new(id, team, unit_info);

        self.units.insert(unit.id, unit);
        self.place_unit(id, location)?;
        Ok(id)
    }

    /// Removes a unit from the map. Assumes the unit is on the map.
    pub fn remove_unit(&mut self, id: unit::UnitID) -> Result<(), Error> {
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
    pub fn delete_unit(&mut self, id: unit::UnitID) {
        self.units.remove(&id);
    }

    /// Returns whether the square is clear for a new unit to occupy, either by movement or by construction.
    pub fn is_occupiable(&self, location: MapLocation) -> Result<bool, Error> {
        let planet_info = &self.get_planet_info(location.planet)?;
        Ok(planet_info.map.is_passable_terrain[location.y as usize][location.x as usize] &&
            !self.units_by_loc.contains_key(&location))
    }

    /// Tests whether the given unit can move.
    pub fn can_move(&self, id: unit::UnitID, direction: Direction) -> Result<bool, Error> {
        let unit = self.get_unit(id)?;
        if let Some(location) = unit.location {
            Ok(unit.is_move_ready() && self.is_occupiable(location.add(direction))?)
        } else {
            Ok(false)
        }
    }

    // Given that moving an unit comprises many edits to the GameWorld, it makes sense to define this here.
    pub fn move_unit(&mut self, id: unit::UnitID, direction: Direction) -> Result<(), Error> {
        let dest = self.get_unit(id)?.location.ok_or(GameError::InvalidAction)?.add(direction);
        if self.can_move(id, direction)? {
            self.remove_unit(id)?;
            self.place_unit(id, dest)?;
            Ok(())
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    pub fn launch_rocket(&mut self, id: unit::UnitID, destination: MapLocation) -> Result<(), Error> {
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
    use super::unit::UnitType;
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
}
