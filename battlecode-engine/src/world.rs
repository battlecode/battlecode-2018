//! The core battlecode engine.

use fnv::FnvHashMap;

use super::constants::*;
use super::schema::Delta;
use super::location::*;
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
}

impl PlanetInfo {
    pub fn new() -> PlanetInfo {
        PlanetInfo {
            map: Map::new(),
            karbonite: vec![vec![0; MAP_MAX_WIDTH]; MAP_MAX_HEIGHT],
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

    /// All the units on the map.
    units: FnvHashMap<unit::UnitID, unit::Unit>,

    /// All the units on the map, by location.
    units_by_loc: FnvHashMap<MapLocation, unit::UnitID>,

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
            units: FnvHashMap::default(),
            units_by_loc: FnvHashMap::default(),
            planet_states: planet_states,
            team_states: FnvHashMap::default(),
        }
    }

    fn get_planet_info(&self, planet: Planet) -> Result<&PlanetInfo, Error> {
        if let Some(planet_info) = self.planet_states.get(&planet) {
            Ok(planet_info)
        } else {
            Err(GameError::NoSuchPlanet)?
        }
    }
    
    fn get_planet_info_mut(&mut self, planet: Planet) -> Result<&mut PlanetInfo, Error> {
        if let Some(planet_info) = self.planet_states.get_mut(&planet) {
            Ok(planet_info)
        } else {
            Err(GameError::NoSuchPlanet)?
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

    /// Places a unit onto the map at the given location. Assumes the given square is occupiable.
    pub fn place_unit(&mut self, id: unit::UnitID, location: MapLocation) -> Result<(), Error> {
        if self.is_occupiable(location) {
            self.get_unit_mut(id)?.location = location;
            self.units_by_loc.insert(location, id);
            Ok(())
        } else {
            Err(GameError::InternalEngineError)?
        }
    }

    /// Removes a unit from the map. Assumes the unit is on the map.
    pub fn remove_unit(&mut self, id: unit::UnitID) -> Result<(), Error> {
        let location = {
            // TODO: unit locations should probably be an Option
            // to better handle unplaced units.
            let unit = self.get_unit_mut(id)?;
            let location = unit.location;
            unit.location = MapLocation::new(Planet::Earth, -1, -1);
            location
        };
        self.units_by_loc.remove(&location);
        Ok(())
    }

    /// Inserts a given unit into the GameWorld, so that it can be referenced by ID.
    pub fn register_unit(&mut self, unit: unit::Unit) {
        self.units.insert(unit.id, unit);
    }

    /// Deletes a unit. Unit should be removed from map first.
    pub fn delete_unit(&mut self, id: unit::UnitID) {
        self.units.remove(&id);
    }

    /// Returns whether the square is clear for a new unit to occupy, either by movement or by construction.
    pub fn is_occupiable(&self, location: MapLocation) -> bool {
        let planet_info = &self.planet_states[&location.planet];
        return planet_info.map.is_passable_terrain[location.y as usize][location.x as usize] &&
            !self.units_by_loc.contains_key(&location);
    }

    /// Tests whether the given unit can move.
    pub fn can_move(&self, id: unit::UnitID, direction: Direction) -> Result<bool, Error> {
        let unit = self.get_unit(id)?;
        Ok(unit.is_move_ready() && self.is_occupiable(unit.location.add(direction)))
    }

    // Given that moving an unit comprises many edits to the GameWorld, it makes sense to define this here.
    pub fn move_unit(&mut self, id: unit::UnitID, direction: Direction) -> Result<(), Error> {
        let dest = self.get_unit(id)?.location.add(direction);
        if self.can_move(id, direction)? {
            self.remove_unit(id)?;
            self.place_unit(id, dest)?;
            Ok(())
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    fn apply(&mut self, delta: Delta) -> Result<(), Error> {
        match delta {
            Delta::Move{id, direction} => self.move_unit(id, direction),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GameWorld;
    use super::unit::Unit;
    use super::super::location::*;

    #[test]
    fn it_works() {}

    #[test]
    fn test_unit_move() {
        // Create the game world, and create and register some robots.
        let mut world = GameWorld::new();
        let a = 0;
        let b = 1;
        let unit_a = Unit::new(a);
        let unit_b = Unit::new(b);
        world.register_unit(unit_a);
        world.register_unit(unit_b);

        // Place the robots onto the map. B is one square east of A.
        world.place_unit(a, MapLocation::new(Planet::Earth, 5, 5)).unwrap();
        world.place_unit(b, MapLocation::new(Planet::Earth, 6, 5)).unwrap();

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
