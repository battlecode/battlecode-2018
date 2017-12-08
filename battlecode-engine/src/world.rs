//! The core battlecode engine.

use std::collections::HashMap;

use super::schema::Delta;
use super::location;
use super::unit;
use super::research;

/// There are two teams in Battlecode: Red and Blue.
#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Team {
    Red,
    Blue,
}

/// The map for one of the planets in the Battlecode world. This information
/// defines the terrain and dimensions of the planet.
#[derive(Debug, Serialize, Deserialize)]
pub struct Map {
    /// The height of this map, in squares.
    height: usize,

    /// The width of this map, in squares.
    width: usize,

    /// The coordinates of the bottom-left corner. Essentially, the
    /// minimum x and y coordinates for this map. Each lies within
    /// [-10,000, 10,000].
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

/// The game state for one of the planets in a game.
///
/// Stores neutral map info (map dimension, terrain, and karbonite deposits)
/// and non-neutral unit info (robots, factories, rockets). This information
/// is generally readable by both teams, and is ephemeral.
#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    /// The map of the game.
    map: Map,

    /// The amount of Karbonite deposited on the specified square.
    ///
    /// Stored as a two-dimensional array, where the first index 
    /// represents a square's y-coordinate, and the second index its 
    /// x-coordinate. These coordinates are *relative to the origin*.
    karbonite: Vec<Vec<u32>>,

    /// Robots on the map.
    robots: Vec<unit::RobotInfo>,

    /// War factories on the map.
    factories: Vec<unit::FactoryInfo>,

    /// Rockets on the map.
    rockets: Vec<unit::RocketInfo>,
}

/// A team-shared communication array.
pub type TeamArray = Vec<i32>;

/// A history of communication arrays. Read from the back of the queue on the
/// current planet, and the front of the queue on the other planet.
pub type TeamArrayHistory = Vec<TeamArray>;

/// Persistent info specific to a single team. Teams are only able to access
/// the team state of their own team.
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamState {
    /// Communication array histories for each planet.
    team_arrays: HashMap<location::Planet, TeamArrayHistory>,

    /// The current status of the team's research. The values defines the level
    /// of the research, where 0 represents no progress.
    research_status: HashMap<research::Branch, u32>,

    /// Research branches queued to be researched, including the current branch.
    research_queue: Vec<research::Branch>,

    /// The number of rounds to go until the first branch in the research
    /// queue is finished. 0 if the research queue is empty.
    research_rounds_left: u32,
}

/// The full world of the Battlecode game.
#[derive(Debug, Serialize, Deserialize)]
pub struct GameWorld {
    /// The current round, starting at 1.
    round: u32,

    game_states: HashMap<location::Planet, GameState>,
    team_states: HashMap<Team, TeamState>,
}
