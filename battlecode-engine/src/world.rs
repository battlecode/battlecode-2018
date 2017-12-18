//! The core battlecode engine.

use fnv::FnvHashMap;

use super::schema::Delta;
use super::location;
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
    origin: location::MapLocation,

    /// Whether the specified square contains passable terrain. Is only
    /// false when the square contains impassable terrain (distinct from
    /// containing a building, for instance).
    ///
    /// Stored as a two-dimensional array, where the first index 
    /// represents a square's y-coordinate, and the second index its 
    /// x-coordinate. These coordinates are *relative to the origin*.
    is_passable_terrain: Vec<Vec<bool>>,
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
    units_by_loc: FnvHashMap<location::MapLocation, unit::Unit>,
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
    team_arrays: FnvHashMap<location::Planet, TeamArrayHistory>,

    /// The current status of the team's research. The values defines the level
    /// of the research, where 0 represents no progress.
    research_status: FnvHashMap<research::Branch, u32>,

    /// Research branches queued to be researched, including the current branch.
    research_queue: Vec<research::Branch>,

    /// The number of rounds to go until the first branch in the research
    /// queue is finished. 0 if the research queue is empty.
    research_rounds_left: u32,
}

/// A player represents a program controlling some group of units.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Player {
    /// The team of this player.
    pub team: Team,

    /// The planet for this player. Each team disjointly controls the robots on each planet.
    pub planet: location::Planet,
}

/// The full world of the Battlecode game.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameWorld {
    /// The current round, starting at 1.
    pub round: u32,

    /// The player whose turn it is.
    pub player_to_move: Player,

    pub planet_states: FnvHashMap<location::Planet, PlanetInfo>,
    pub team_states: FnvHashMap<Team, TeamInfo>,
}

impl GameWorld {
    pub fn new() -> GameWorld {
        GameWorld {
            round: 1,
            player_to_move: Player { team: Team::Red, planet: location::Planet::Earth },
            planet_states: FnvHashMap::default(),
            team_states: FnvHashMap::default(),
        }
    }

    fn get_unit(&mut self, id: unit::UnitID) -> Option<&unit::Unit> {
        if let Some(planet_info) = self.planet_states.get(&self.player_to_move.planet) {
            planet_info.units.get(&id)
        } else {
            None
        }
    }

    fn get_unit_mut(&mut self, id: unit::UnitID) -> Option<&mut unit::Unit> {
        if let Some(planet_info) = self.planet_states.get_mut(&self.player_to_move.planet) {
            planet_info.units.get_mut(&id)
        } else {
            None
        }
    }

    /// Returns whether the square is clear for a new unit to occupy, either by movement or by construction.
    fn is_occupiable(&self, location: &location::MapLocation) -> bool {
        true
    }

    // Given that moving an unit comprises many edits to the GameWorld, it makes sense to define this here.
    pub fn move_unit(&mut self, id: unit::UnitID, direction: location::Direction) -> Result<(), GameError> {
        let dest = if let Some(unit) = self.get_unit(id) {
            unit.location.add(direction)
        } else {
            return Err(GameError::NoSuchUnit);
        };
        if self.is_occupiable(&dest) {
            if let Some(unit) = self.get_unit_mut(id) {
                if unit.is_move_ready() {
                    unit.location = dest;
                    Ok(())
                } else {
                    Err(GameError::InvalidAction)
                }
            } else {
                // It should be impossible for this error to trigger, given that we've already
                // checked that the unit exists.
                Err(GameError::InternalEngineError)
            }
        } else {
            Err(GameError::InvalidAction)
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
    #[test]
    fn it_works() {}
}
