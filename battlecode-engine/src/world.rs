//! The core battlecode engine.

use fnv::FnvHashMap;

use super::constants::*;
use super::schema::Delta;
use super::location::*;
use super::unit;
use super::research;
use super::error::GameError;

/// There are two teams in Battlecode: Red and Blue.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Team {
    Red,
    Blue,
}

/// The map for one of the planets in the Battlecode world. This information
/// defines the terrain and dimensions of the planet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Map {
    /// The height of this map, in squares. Must be in the range
    /// [constants::MAP_MIN_HEIGHT, constants::MAP_MAX_HEIGHT], inclusive.
    height: usize,

    /// The width of this map, in squares. Must be in the range
    /// [constants::MAP_MIN_WIDTH, constants::MAP_MAX_WIDTH], inclusive.
    width: usize,

    /// The coordinates of the bottom-left corner. Essentially, the
    /// minimum x and y coordinates for this map. Each lies within
    /// [constants::MAP_MIN_COORDINATE, constants::MAP_MAX_COORDINATE],
    /// inclusive.
    origin: MapLocation,

    /// Whether the specified square contains passable terrain. Is only
    /// false when the square contains impassable terrain (distinct from
    /// containing a building, for instance).
    ///
    /// Stored as a two-dimensional array, where the first index 
    /// represents a square's y-coordinate, and the second index its 
    /// x-coordinate. These coordinates are *relative to the origin*.
    is_passable_terrain: Vec<Vec<bool>>,
}

impl Map {
    pub fn new() -> Map {
        Map {
            height: MAP_MIN_HEIGHT,
            width: MAP_MIN_WIDTH,
            origin: MapLocation::new(Planet::Earth, 0, 0),
            is_passable_terrain: vec![vec![true; MAP_MIN_WIDTH]; MAP_MIN_HEIGHT],
        }
    }
}

/// The state for one of the planets in a game.
///
/// Stores neutral map info (map dimension, terrain, and karbonite deposits)
/// and non-neutral unit info (robots, factories, rockets). This information
/// is generally readable by both teams, and is ephemeral.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanetInfo {
    /// The map of the game.
    map: Map,

    /// The amount of Karbonite deposited on the specified square.
    ///
    /// Stored as a two-dimensional array, where the first index 
    /// represents a square's y-coordinate, and the second index its 
    /// x-coordinate. These coordinates are *relative to the origin*.
    karbonite: Vec<Vec<u32>>,

    /// All the units on the map.
    units: FnvHashMap<unit::UnitID, unit::Unit>,

    /// All the units on the map, by location.
    units_by_loc: FnvHashMap<MapLocation, unit::Unit>,
}

impl PlanetInfo {
    pub fn new() -> PlanetInfo {
        PlanetInfo {
            map: Map::new(),
            karbonite: vec![vec![0; MAP_MAX_WIDTH]; MAP_MAX_HEIGHT],
            units: FnvHashMap::default(),
            units_by_loc: FnvHashMap::default(),
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
    /// Communication array histories for each planet.
    team_arrays: FnvHashMap<Planet, TeamArrayHistory>,

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
    pub fn new() -> TeamInfo {
        TeamInfo {
            team_arrays: FnvHashMap::default(),
            research_status: FnvHashMap::default(),
            research_queue: Vec::new(),
            research_rounds_left: 0,
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

    pub planet_states: FnvHashMap<Planet, PlanetInfo>,
    pub team_states: FnvHashMap<Team, TeamInfo>,
}

impl GameWorld {
    pub fn new() -> GameWorld {
        let mut planet_states = FnvHashMap::default();
        planet_states.insert(Planet::Earth, PlanetInfo::new());
        GameWorld {
            round: 1,
            player_to_move: Player { team: Team::Red, planet: Planet::Earth },
            planet_states: planet_states,
            team_states: FnvHashMap::default(),
        }
    }

    fn get_planet_info(&self) -> &PlanetInfo {
        if let Some(planet_info) = self.planet_states.get(&self.player_to_move.planet) {
            planet_info
        } else {
            panic!("Internal engine error");
        }
    }
    
    fn get_planet_info_mut(&mut self) -> &mut PlanetInfo {
        if let Some(planet_info) = self.planet_states.get_mut(&self.player_to_move.planet) {
            planet_info
        } else {
            panic!("Internal engine error");
        }
    }

    fn get_unit(&self, id: unit::UnitID) -> Option<&unit::Unit> {
        self.get_planet_info().units.get(&id)
    }

    fn get_unit_mut(&mut self, id: unit::UnitID) -> Option<&mut unit::Unit> {
        self.get_planet_info_mut().units.get_mut(&id)
    }

    /// Returns whether the square is clear for a new unit to occupy, either by movement or by construction.
    pub fn is_occupiable(&self, location: &MapLocation) -> bool {
        let planet_info = &self.planet_states[&location.planet];
        return planet_info.map.is_passable_terrain[location.y as usize][location.x as usize] &&
            !planet_info.units_by_loc.contains_key(location);
    }
    
    // Given that moving an unit comprises many edits to the GameWorld, it makes sense to define this here.
    pub fn move_unit(&mut self, id: unit::UnitID, direction: Direction) -> Result<(), GameError> {
        let (src, dest) = if let Some(unit) = self.get_unit(id) {
            if unit.is_move_ready() {
                (unit.location, unit.location.add(direction))
            } else {
                return Err(GameError::InvalidAction);
            }
        } else {
            return Err(GameError::NoSuchUnit);
        };
        if self.is_occupiable(&dest) {
            if let Some(unit) = self.get_unit_mut(id) {
                if unit.is_move_ready() {
                    unit.location = dest;
                } else {
                    return Err(GameError::InvalidAction);
                }
            } else {
                // It should be impossible for this error to trigger, given that we've already
                // checked that the unit exists.
                return Err(GameError::InternalEngineError);
            }
        } else {
            return Err(GameError::InvalidAction);
        }
        let planet_info = self.get_planet_info_mut();
        if let Some(unit) = planet_info.units_by_loc.remove(&src) {
            planet_info.units_by_loc.insert(dest, unit);
            Ok(())
        } else {
            Err(GameError::InternalEngineError)
        }
    }

    fn apply(&mut self, delta: Delta) -> Result<(), GameError> {
        match delta {
            Delta::Move{id, direction} => self.move_unit(id, direction),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GameWorld;

    #[test]
    fn it_works() {}

    #[test]
    fn test_is_occupiable() {
        let mut world = GameWorld::new()
    }
}
