//! The core battlecode engine.

use fnv::FnvHashMap;
use std::cmp;
use std::cmp::Ordering;
use std::collections::HashMap;

use super::constants::*;
use super::schema::*;
use super::id_generator::IDGenerator;
use super::location::*;
use super::location::Location::*;
use super::map::*;
use super::unit::*;
use super::unit::UnitType as Branch;
use super::research::*;
use super::rockets::*;
use super::team_array::*;
use super::error::GameError;
use failure::Error;

/// A round consists of a turn from each player.
pub type Rounds = u32;

/// There are two teams in Battlecode: Red and Blue.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Team {
    Red = 0,
    Blue = 1,
}

impl Team {
    /// The other team.
    pub fn other(&self) -> Team {
        match *self {
            Team::Red => Team::Blue,
            Team::Blue => Team::Red,
        }
    }
}

/// The state for one of the planets in a game.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct PlanetInfo {
    /// Visible locations. True if and only if visible.
    ///
    /// Stored as a two-dimensional array, where the first index
    /// represents a square's y-coordinate, and the second index its
    /// x-coordinate.
    visible_locs: Vec<Vec<bool>>,

    /// The unit controllers in the vision range.
    ///
    /// Invariants:
    /// 1. Every entry has a corresponding entry in `unit_infos`.
    /// 2. In the Player engine, only has units on the current team.
    units: FnvHashMap<UnitID, Unit>,

    /// The units in the vision range. (Not every unit info may have a unit.)
    ///
    /// Invariants:
    /// 1. Has every unit with a visible location on this planet.
    /// 2. Has every unit in a visible structure on this planet. In the Player
    ///    Engine, this is only true for structures on the current team. This
    ///    is because one team should not know the existence of units in the
    ///    structures of other teams.
    unit_infos: FnvHashMap<UnitID, UnitInfo>,

    /// All the units on the map, by map location. Cached for performance.
    ///
    /// Invariants:
    /// 1. Has every unit with a visible location on this planet.
    /// 2. Every entry has a corresponding entry in `unit_infos`.
    units_by_loc: FnvHashMap<MapLocation, UnitID>,

    /// The amount of Karbonite deposited on the specified square.
    ///
    /// Stored as a two-dimensional array, where the first index 
    /// represents a square's y-coordinate, and the second index its 
    /// x-coordinate.
    karbonite: Vec<Vec<u32>>,
}

impl PlanetInfo {
    /// Construct a planet with the given map, where the current karbonite
    /// deposits are initialized with the map's initial deposits.
    pub fn new(map: &PlanetMap) -> PlanetInfo {
        PlanetInfo {
            visible_locs: vec![vec![true; map.width]; map.height],
            units: FnvHashMap::default(),
            unit_infos: FnvHashMap::default(),
            units_by_loc: FnvHashMap::default(),
            karbonite: map.initial_karbonite.clone(),
        }
    }
}

/// Persistent info specific to a single team. Teams are only able to access
/// the team info of their own team.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct TeamInfo {
    /// Communication array histories for each planet.
    team_arrays: TeamArrayInfo,

    /// Rocket landings for this team.
    rocket_landings: RocketLandingInfo,

    /// The current state of research.
    research: ResearchInfo,

    /// The units on this team in space, or in a rocket that is in space.
    units_in_space: FnvHashMap<UnitID, Unit>,

    /// The karbonite in the team's resource pool.
    karbonite: u32,
}

impl TeamInfo {
    /// Construct a team with the default properties.
    fn new() -> TeamInfo {
        TeamInfo {
            team_arrays: TeamArrayInfo::new(),
            rocket_landings: RocketLandingInfo::new(),
            research: ResearchInfo::new(),
            units_in_space: FnvHashMap::default(),
            karbonite: KARBONITE_STARTING,
        }
    }
}

/// A player represents a program controlling some group of units.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
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

    /// All players, in the order they go in.
    pub fn all() -> Vec<Player> {
        vec![
            Player::new(Team::Red, Planet::Earth),
            Player::new(Team::Blue, Planet::Earth),
            Player::new(Team::Red, Planet::Mars),
            Player::new(Team::Blue, Planet::Mars),
        ]
    }

    /// The first player to move.
    pub fn first_to_move() -> Player {
        Player::new(Team::Red, Planet::Earth)
    }
}

/// The full world of the Battlecode game.
///
/// The contents of the game world differ depending on whether it exists in the
/// Teh Devs engine or the Player engine. Do not be concerned - this ensures
/// that your player the visibility it's supposed to have!
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GameWorld {
    /// The current round, starting at 1.
    round: Rounds,

    /// The player whose turn it is.
    player_to_move: Player,

    /// Unit ID generator.
    id_generator: IDGenerator,

    /// The asteroid strike pattern on Mars.
    asteroids: AsteroidPattern,

    /// The orbit pattern that determines a rocket's flight duration.
    orbit: OrbitPattern,

    /// The map of each planet.
    planet_maps: FnvHashMap<Planet, PlanetMap>,

    /// The state of each planet.
    planet_states: FnvHashMap<Planet, PlanetInfo>,

    /// The state of each team.
    team_states: FnvHashMap<Team, TeamInfo>,

    /// Cached game worlds per player, to calculate start turn messages.
    /// These worlds were filtered at the start of the turn.
    cached_world: HashMap<Player, GameWorld>,

    /// A list of additional messages to be sent to the viewer. Flushed
    /// at the end of each round.
    viewer_changes: Vec<ViewerDelta>,
}

impl GameWorld {
    /// Initialize a new game world with maps from both planets.
    pub(crate) fn new(map: GameMap) -> GameWorld {
        let mut planet_states = FnvHashMap::default();
        planet_states.insert(Planet::Earth, PlanetInfo::new(&map.earth_map));
        planet_states.insert(Planet::Mars, PlanetInfo::new(&map.mars_map));

        let mut team_states = FnvHashMap::default();
        team_states.insert(Team::Red, TeamInfo::new());
        team_states.insert(Team::Blue, TeamInfo::new());

        let mut planet_maps = FnvHashMap::default();
        planet_maps.insert(Planet::Earth, map.earth_map.clone());
        planet_maps.insert(Planet::Mars, map.mars_map.clone());

        let mut world = GameWorld {
            round: 1,
            player_to_move: Player { team: Team::Red, planet: Planet::Earth },
            id_generator: IDGenerator::new(map.seed),
            asteroids: map.asteroids,
            orbit: map.orbit,
            planet_maps: planet_maps,
            planet_states: planet_states,
            team_states: team_states,
            cached_world: HashMap::default(),
            viewer_changes: Vec::new(),
        };

        // Insert initial units.
        for unit in &map.earth_map.initial_units {
            world.insert_unit(unit.clone());
        }
        for unit in &map.mars_map.initial_units {
            world.insert_unit(unit.clone());
        }

        // Cache the initial filtered states.
        let mut cached_world = HashMap::default();
        for player in Player::all() {
            cached_world.insert(player, world.filter(player));
        }
        world.cached_world = cached_world;
        world
    }

    /// Generate a test world with empty maps.
    #[cfg(test)]
    pub(crate) fn test_world() -> GameWorld {
        let map = GameMap::test_map();

        let mut planet_states = FnvHashMap::default();
        planet_states.insert(Planet::Earth, PlanetInfo::new(&map.earth_map));
        planet_states.insert(Planet::Mars, PlanetInfo::new(&map.mars_map));

        let mut team_states = FnvHashMap::default();
        team_states.insert(Team::Red, TeamInfo::new());
        team_states.insert(Team::Blue, TeamInfo::new());

        let mut planet_maps = FnvHashMap::default();
        planet_maps.insert(Planet::Earth, map.earth_map);
        planet_maps.insert(Planet::Mars, map.mars_map);

        let mut world = GameWorld {
            round: 1,
            player_to_move: Player { team: Team::Red, planet: Planet::Earth },
            id_generator: IDGenerator::new(map.seed),
            asteroids: map.asteroids,
            orbit: map.orbit,
            planet_maps: planet_maps,
            planet_states: planet_states,
            team_states: team_states,
            cached_world: HashMap::default(),
            viewer_changes: Vec::new(),
        };

        // Cache the initial filtered states.
        let mut cached_world = HashMap::default();
        for player in Player::all() {
            cached_world.insert(player, world.filter(player));
        }
        world.cached_world = cached_world;
        world
    }

    /// Filters the game world from the perspective of the current player. All
    /// units are within the player's vision range. Information on the opposing
    /// player's units are all stored in `UnitInfo` structs, not `Unit`. Private
    /// player information like communication arrays and rockets in space should
    /// only be stored for the current player.
    ///
    /// As an invariant, the game world filtered once should be the same as the
    /// game world filtered multiple times.
    pub(crate) fn filter(&self, player: Player) -> GameWorld {
        let team = player.team;
        let planet = player.planet;
        let map = self.starting_map(planet);

        // Filter the unit controllers, including the units in garrisons.
        let mut units: FnvHashMap<UnitID, Unit> = FnvHashMap::default();
        for (id, unit) in self.get_planet(planet).units.clone().into_iter() {
            if unit.team() == team {
                units.insert(id, unit);
            }
        }

        // Calculate the visible locations on this team that are on the map.
        let mut visible_locs = vec![vec![false; map.width]; map.height];
        for unit in units.values().into_iter() {
            if !unit.location().on_map() {
                continue;
            }

            let loc = unit.location().map_location().expect("unit is not on the map");
            let locs = self.all_locations_within(loc, unit.vision_range());
            for loc in locs {
                visible_locs[loc.y as usize][loc.x as usize] = true;
            }
        }

        // Filter the unit infos and unit by location.
        let mut unit_infos: FnvHashMap<UnitID, UnitInfo> = FnvHashMap::default();
        let mut units_by_loc: FnvHashMap<MapLocation, UnitID> = FnvHashMap::default();
        for (id, unit) in self.get_planet(planet).unit_infos.clone().into_iter() {
            if let OnMap(loc) = unit.location {
                if !visible_locs[loc.y as usize][loc.x as usize] {
                    continue;
                }
                units_by_loc.insert(loc, id);
                unit_infos.insert(id, unit);
            } else if unit.team == team {
                // Units in garrisons are only visible if on your team
                unit_infos.insert(id, unit);
            }
        };

        // Filter the team states.
        let mut team_states: FnvHashMap<Team, TeamInfo> = FnvHashMap::default();
        let old_team_state = self.get_team(team);
        let new_team_state = TeamInfo {
            team_arrays: old_team_state.team_arrays.filter(planet),
            rocket_landings: old_team_state.rocket_landings.clone(),
            research: old_team_state.research.clone(),
            units_in_space: old_team_state.units_in_space.clone(),
            karbonite: old_team_state.karbonite,
        };
        team_states.insert(team, new_team_state);

        // Planet state.
        let mut planet_states: FnvHashMap<Planet, PlanetInfo> = FnvHashMap::default();
        let planet_info = PlanetInfo {
            visible_locs: visible_locs,
            units: units,
            unit_infos: unit_infos,
            units_by_loc: units_by_loc,
            karbonite: self.get_planet(planet).karbonite.clone(),
        };
        planet_states.insert(planet, planet_info);

        GameWorld {
            round: self.round,
            player_to_move: player,
            id_generator: self.id_generator.clone(),
            asteroids: self.asteroids.clone(),
            orbit: self.orbit.clone(),
            planet_maps: self.planet_maps.clone(),
            planet_states: planet_states,
            team_states: team_states,
            cached_world: HashMap::default(),
            viewer_changes: Vec::new(),
        }
    }

    // ************************************************************************
    // ************************** GENERAL METHODS *****************************
    // ************************************************************************

    /// The current round, starting at round 1 and up to `ROUND_LIMIT` rounds.
    /// A round consists of a turn from each team on each planet.
    pub fn round(&self) -> Rounds {
        self.round
    }

    /// The current planet.
    pub fn planet(&self) -> Planet {
        self.player_to_move.planet
    }

    /// The team whose turn it is.
    pub fn team(&self) -> Team {
        self.player_to_move.team
    }

    /// The starting map of the given planet. Includes the map's planet,
    /// dimensions, impassable terrain, and initial units and karbonite.
    pub fn starting_map(&self, planet: Planet) -> &PlanetMap {
        if let Some(map) = self.planet_maps.get(&planet) {
            map
        } else {
            unreachable!();
        }
    }

    /// The karbonite in the team's resource pool.
    pub fn karbonite(&self) -> u32 {
        self.my_team().karbonite
    }

    // ************************************************************************
    // ************************** SENSING METHODS *****************************
    // ************************************************************************

    /// The unit controller for the unit of this ID. Use this method to get
    /// detailed statistics on a unit in your team: heat, cooldowns, and
    /// properties of special abilities like units garrisoned in a rocket.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    pub fn unit_controller(&self, id: UnitID) -> Result<&Unit, Error> {
        self.my_unit(id)
    }

    /// The single unit with this ID.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    pub fn unit(&self, id: UnitID) -> Result<UnitInfo, Error> {
        self.unit_info(id)
    }

    /// All the units within the vision range, in no particular order.
    /// Does not include units in space.
    pub fn units(&self) -> Vec<&UnitInfo> {
        self.my_planet().unit_infos.values().collect::<Vec<&UnitInfo>>()
    }

    /// All the units within the vision range, by ID.
    /// Does not include units in space.
    pub fn units_by_id(&self) -> FnvHashMap<UnitID, UnitInfo> {
        self.my_planet().unit_infos.clone()
    }

    /// All the units within the vision range, by location.
    /// Does not include units in garrisons or in space.
    pub fn units_by_loc(&self) -> FnvHashMap<MapLocation, UnitID> {
        self.my_planet().units_by_loc.clone()
    }

    /// All the units visible in space.
    pub fn units_in_space(&self) -> Vec<UnitInfo> {
        let mut units = vec![];
        for unit in self.my_team().units_in_space.values().into_iter() {
            units.push(unit.info());
        }
        units
    }

    /// The karbonite at the given location.
    ///
    /// * GameError::InvalidLocation - the location is outside the vision range.
    pub fn karbonite_at(&self, location: MapLocation) -> Result<u32, Error> {
        if self.can_sense_location(location) {
            Ok(self.my_planet().karbonite[location.y as usize][location.x as usize])
        } else {
            Err(GameError::LocationNotVisible)?
        }
    }

    /// Returns an array of all locations within a certain radius squared of
    /// this location that are on the map.
    ///
    /// The locations are ordered first by the x-coordinate, then the
    /// y-coordinate. The radius squared is inclusive.
    pub fn all_locations_within(&self, location: MapLocation,
                                radius_squared: u32) -> Vec<MapLocation> {
        let mut locations = vec![];
        let map = self.starting_map(location.planet);

        let radius = (radius_squared as f32).sqrt() as i32;
        let min_x = cmp::max(location.x - radius, 0);
        let max_x = cmp::min(location.x + radius, map.width as i32 - 1);
        let min_y = cmp::max(location.y - radius, 0);
        let max_y = cmp::min(location.y + radius, map.height as i32 - 1);

        for x in min_x..max_x + 1 {
            for y in min_y..max_y + 1 {
                let loc = MapLocation::new(location.planet, x, y);
                if location.distance_squared_to(loc) <= radius_squared {
                    locations.push(loc);
                }
            }
        }

        locations
    }

    /// Whether the location is within the vision range.
    pub fn can_sense_location(&self, location: MapLocation) -> bool {
        if self.planet() != location.planet {
            return false;
        }

        if location.x < 0 || location.y < 0 {
            return false;
        }

        let map = self.starting_map(location.planet);
        if location.x >= map.width as i32 || location.y >= map.height as i32 {
            return false;
        }

        let x = location.x as usize;
        let y = location.y as usize;
        self.my_planet().visible_locs[y][x]
    }

    /// Whether there is a unit with this ID within the vision range.
    pub fn can_sense_unit(&self, id: UnitID) -> bool {
        self.unit_info(id).is_ok()
    }

    /// Sense units near the location within the given radius, inclusive, in
    /// distance squared. The units are within the vision range.
    pub fn sense_nearby_units(&self, location: MapLocation, radius: u32)
                              -> Vec<UnitInfo> {
        let mut units: Vec<UnitInfo> = vec![];
        for nearby_loc in self.all_locations_within(location, radius) {
            if let Some(id) = self.my_planet().units_by_loc.get(&nearby_loc) {
                units.push(self.unit_info(*id).expect("unit exists"));
            }
        }
        units
    }

    /// Sense units near the location within the given radius, inclusive, in
    /// distance squared. The units are within the vision range. Additionally
    /// filters the units by team.
    pub fn sense_nearby_units_by_team(&self, location: MapLocation,
                                      radius: u32, team: Team) -> Vec<UnitInfo> {
        self.sense_nearby_units(location, radius).into_iter()
            .filter(|unit| unit.team == team)
            .collect::<Vec<UnitInfo>>()
    }

    /// Sense units near the location within the given radius, inclusive, in
    /// distance squared. The units are within the vision range. Additionally
    /// filters the units by unit type.
    pub fn sense_nearby_units_by_type(&self, location: MapLocation,
                                      radius: u32, unit_type: UnitType) -> Vec<UnitInfo> {
        self.sense_nearby_units(location, radius).into_iter()
            .filter(|unit| unit.unit_type == unit_type)
            .collect::<Vec<UnitInfo>>()
    }

    /// The unit at the location, if it exists.
    ///
    /// * GameError::InvalidLocation - the location is outside the vision range.
    pub fn sense_unit_at_location(&self, location: MapLocation)
                                  -> Result<Option<UnitInfo>, Error> {
        if self.can_sense_location(location) {
            let unit_id = self.my_planet().units_by_loc.get(&location);
            Ok(unit_id.map(|id| self.unit_info(*id).expect("unit exists")))
        } else {
            Err(GameError::LocationNotVisible)?
        }
    }

    // ************************************************************************
    // ************************** WEATHER METHODS *****************************
    // ************************************************************************

    /// The asteroid strike pattern on Mars.
    pub fn asteroid_pattern(&self) -> AsteroidPattern {
        self.asteroids.clone()
    }

    /// The orbit pattern that determines a rocket's flight duration.
    pub fn orbit_pattern(&self) -> OrbitPattern {
        self.orbit.clone()
    }

    /// The current duration of flight if a rocket were to be launched this
    /// round. Does not take into account any research done on rockets.
    pub fn current_duration_of_flight(&self) -> Rounds {
        self.orbit.duration(self.round)
    }

    fn process_asteroids(&mut self) {
        if self.asteroids.asteroid(self.round).is_some() {
            let (location, karbonite) = {
                let asteroid = self.asteroids.asteroid(self.round).unwrap();
                (asteroid.location, asteroid.karbonite)
            };
            self.viewer_changes.push(ViewerDelta::AsteroidStrike { location });
            let planet_info = self.get_planet_mut(location.planet);
            planet_info.karbonite[location.y as usize][location.x as usize] += karbonite;
        }
    }

    // ************************************************************************
    // *********************** COMMUNICATION METHODS **************************
    // ************************************************************************

    /// Gets a read-only version of this planet's team array. If the given
    /// planet is different from the planet of the player, reads the version
    /// of the planet's team array from COMMUNICATION_DELAY rounds prior.
    pub fn get_team_array(&self, planet: Planet) -> &TeamArray {
        if planet == self.planet() {
            self.my_team().team_arrays.first_array(planet)
        } else {
            self.my_team().team_arrays.last_array(planet)
        }
    }

    /// Writes the value at the index of this planet's team array.
    ///
    /// * GameError::ArrayOutOfBounds - the index of the array is out of
    ///   bounds. It must be within [0, COMMUNICATION_ARRAY_LENGTH).
    pub fn write_team_array(&mut self, index: usize, value: i32) -> Result<(), Error> {
        let planet = self.planet();
        self.my_team_mut().team_arrays.write(planet, index, value)
    }

    // ************************************************************************
    // ****************************** ACCESSORS *******************************
    // ************************************************************************

    fn my_planet(&self) -> &PlanetInfo {
        let planet = self.planet();
        if let Some(planet_info) = self.planet_states.get(&planet) {
            planet_info
        } else {
            unreachable!();
        }
    }

    fn my_planet_mut(&mut self) -> &mut PlanetInfo {
        let planet = self.planet();
        if let Some(planet_info) = self.planet_states.get_mut(&planet) {
            planet_info
        } else {
            unreachable!();
        }
    }

    fn my_team(&self) -> &TeamInfo {
        let team = self.team();
        if let Some(team_info) = self.team_states.get(&team) {
            team_info
        } else {
            unreachable!();
        }
    }

    fn my_team_mut(&mut self) -> &mut TeamInfo {
        let team = self.team();
        if let Some(team_info) = self.team_states.get_mut(&team) {
            team_info
        } else {
            unreachable!();
        }
    }

    fn get_planet(&self, planet: Planet) -> &PlanetInfo {
        if let Some(planet_info) = self.planet_states.get(&planet) {
            planet_info
        } else {
            unreachable!();
        }
    }

    fn get_planet_mut(&mut self, planet: Planet) -> &mut PlanetInfo {
        if let Some(planet_info) = self.planet_states.get_mut(&planet) {
            planet_info
        } else {
            unreachable!();
        }
    }

    fn get_team(&self, team: Team) -> &TeamInfo {
        if let Some(team_info) = self.team_states.get(&team) {
            team_info
        } else {
            unreachable!();
        }
    }

    fn get_team_mut(&mut self, team: Team) -> &mut TeamInfo {
        if let Some(team_info) = self.team_states.get_mut(&team) {
            team_info
        } else {
            unreachable!();
        }
    }

    fn unit_info(&self, id: UnitID) -> Result<UnitInfo, Error> {
        if let Some(unit) = self.my_planet().unit_infos.get(&id) {
            Ok(unit.clone())
        } else if let Some(unit) = self.my_team().units_in_space.get(&id) {
            Ok(unit.info())
        } else {
            Err(GameError::NoSuchUnit)?
        }
    }

    fn unit_info_mut(&mut self, id: UnitID) -> Result<&mut UnitInfo, Error> {
        if self.my_planet().unit_infos.contains_key(&id) {
            Ok(self.my_planet_mut().unit_infos.get_mut(&id).expect("key exists"))
        } else {
            Err(GameError::NoSuchUnit)?
        }
    }

    /// Gets this unit from space or the current planet. Checks that its team
    /// is the same as the current team.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    fn my_unit(&self, id: UnitID) -> Result<&Unit, Error> {
        if self.unit_info(id)?.team != self.team() {
            return Err(GameError::TeamNotAllowed)?;
        }

        if let Some(unit) = self.my_planet().units.get(&id) {
            Ok(unit)
        } else if let Some(unit) = self.my_team().units_in_space.get(&id) {
            Ok(unit)
        } else {
            unreachable!();
        }
    }

    /// Gets a mutable version of this unit from space or the current planet.
    /// Checks that its team is the same as the current team.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    fn my_unit_mut(&mut self, id: UnitID) -> Result<&mut Unit, Error> {
        if self.unit_info(id)?.team != self.team() {
            return Err(GameError::TeamNotAllowed)?;
        }

        if self.my_planet().units.contains_key(&id) {
            Ok(self.my_planet_mut().units.get_mut(&id).expect("key exists"))
        } else if self.my_team().units_in_space.contains_key(&id) {
            Ok(self.my_team_mut().units_in_space.get_mut(&id).expect("key exists"))
        } else {
            unreachable!();
        }
    }

    /// Gets this unit from space or the current planet.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    fn get_unit(&self, id: UnitID) -> Result<&Unit, Error> {
        if let Some(unit) = self.my_planet().units.get(&id) {
            Ok(unit)
        } else if let Some(unit) = self.my_team().units_in_space.get(&id) {
            Ok(unit)
        } else {
            Err(GameError::NoSuchUnit)?
        }
    }

    /// Gets a mutable version of this unit from space or the current planet.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    fn get_unit_mut(&mut self, id: UnitID) -> Result<&mut Unit, Error> {
        if self.my_planet().units.contains_key(&id) {
            Ok(self.my_planet_mut().units.get_mut(&id).expect("key exists"))
        } else if self.my_team().units_in_space.contains_key(&id) {
            Ok(self.my_team_mut().units_in_space.get_mut(&id).expect("key exists"))
        } else {
            Err(GameError::NoSuchUnit)?
        }
    }

    // ************************************************************************
    // **************** UNIT CREATION / DESTRUCTION METHODS *******************
    // ************************************************************************

    /// Places the unit in location-based indexing and/or marks the unit info.
    /// Must be called after changing a unit's location within a planet.
    fn place_unit(&mut self, id: UnitID) {
        match self.my_unit(id)
                  .expect("Unit does not exist and cannot be placed.")
                  .location() {
            OnMap(map_loc) => {
                self.my_planet_mut().units_by_loc.insert(map_loc, id);
                self.unit_info_mut(id)
                    .expect("unit exists").location = OnMap(map_loc);
            },
            InGarrison(structure_id) => {
                self.unit_info_mut(id)
                    .expect("unit exists").location = InGarrison(structure_id);
            },
            _ => panic!("Unit is not on a planet and cannot be placed."),
        }
    }

    /// Temporarily removes this unit from any location-based indexing.
    /// Must be on the current planet and team before being removed.
    fn remove_unit(&mut self, id: UnitID) {
        match self.my_unit(id)
                  .expect("Unit does not exist and cannot be removed.")
                  .location() {
            OnMap(loc) => {
                self.my_planet_mut().units_by_loc.remove(&loc);
            },
            _ => panic!("Unit is not on a map and cannot be removed."),
        }
    }

    /// Moves this rocket and any location-based indexing to space. Must be
    /// on the current planet and team. Also moves all the units inside it.
    fn move_to_space(&mut self, rocket_id: UnitID) {
        self.remove_unit(rocket_id);

        let rocket = self.my_planet_mut().units.remove(&rocket_id).expect("unit exists");
        self.my_planet_mut().unit_infos.remove(&rocket_id).expect("unit exists");

        for id in rocket.structure_garrison().expect("unit is a rocket") {
            let unit = self.my_planet_mut().units.remove(&id).expect("unit exists");
            self.my_planet_mut().unit_infos.remove(&id).expect("unit exists");
            self.my_team_mut().units_in_space.insert(id, unit);
        }
        self.my_team_mut().units_in_space.insert(rocket_id, rocket);
    }

    /// Moves this rocket and any location-based indexing from space to this
    /// planet. Must currently be in space. Also move all the units inside it.
    fn move_from_space(&mut self, rocket_id: UnitID) {
        let rocket = self.my_team_mut().units_in_space.remove(&rocket_id).expect("unit exists");

        for id in rocket.structure_garrison().expect("unit is a rocket") {
            let unit = self.my_team_mut().units_in_space.remove(&id).expect("unit exists");
            self.my_planet_mut().unit_infos.insert(id, unit.info());
            self.my_planet_mut().units.insert(id, unit);
        }

        self.my_planet_mut().unit_infos.insert(rocket_id, rocket.info());
        self.my_planet_mut().units.insert(rocket_id, rocket);
        self.place_unit(rocket_id);
    }

    /// Inserts a new unit into the internal data structures of the game world,
    /// assuming it is on the map.
    fn insert_unit(&mut self, unit: Unit) {
        let id = unit.id();
        let location = unit.location().map_location().expect("unit is on map");
        self.get_planet_mut(location.planet).unit_infos.insert(id, unit.info());
        self.get_planet_mut(location.planet).units.insert(id, unit);
        self.get_planet_mut(location.planet).units_by_loc.insert(location, id);
    }

    /// Creates and inserts a new unit into the game world, so that it can be
    /// referenced by ID. Used for testing only!!!
    pub(crate) fn create_unit(&mut self, team: Team, location: MapLocation,
                       unit_type: UnitType) -> Result<UnitID, Error> {
        let id = self.id_generator.next_id();
        let level = self.get_team(team).research.get_level(&unit_type);
        let unit = Unit::new(id, team, unit_type, level, OnMap(location))?;

        self.insert_unit(unit);
        Ok(id)
    }

    /// Destroys a unit. Removes any traces of it.
    ///
    /// If the unit is a rocket or factory, also destroys units in its garrison.
    fn destroy_unit(&mut self, id: UnitID) {
        // We need to call unit_info() here to ensure that this unit exists in
        // both the player engine and the dev engine.
        match self.unit_info(id)
                  .expect("Unit does not exist and cannot be destroyed.")
                  .location {
            OnMap(loc) => {
                self.my_planet_mut().units_by_loc.remove(&loc);
            },
            InSpace => {
                // Units only die in space after a landing on their turn.
                // Thus we are guaranteed that my_unit() will find the unit.
                for utd_id in self.my_unit(id).unwrap().structure_garrison()
                                  .expect("only rockets can die in space") {
                    self.my_team_mut().units_in_space.remove(&utd_id);
                }
                self.my_team_mut().units_in_space.remove(&id);
                return;
            },
            _ => panic!("Unit is in ???, this should not be possible"),
        };

        // If this unit's garrison is visible, destroy those units too.
        if self.get_unit(id).is_ok() {
            let unit_type = self.get_unit(id).unwrap().unit_type();
            if unit_type == UnitType::Rocket || unit_type == UnitType::Factory {
                let units_to_destroy = self.get_unit_mut(id).unwrap()
                                           .structure_garrison().unwrap();
                for utd_id in units_to_destroy.iter() {
                    self.my_planet_mut().units.remove(&utd_id);
                    self.my_planet_mut().unit_infos.remove(&utd_id);
                }
            }
        }

        self.my_planet_mut().units.remove(&id);
        self.my_planet_mut().unit_infos.remove(&id);
    }

    /// Disintegrates the unit and removes it from the map. If the unit is a
    /// factory or a rocket, also disintegrates any units garrisoned inside it.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    pub fn disintegrate_unit(&mut self, id: UnitID) -> Result<(), Error> {
        self.my_unit(id)?;
        self.destroy_unit(id);
        Ok(())
    }

    // ************************************************************************
    // ************************* LOCATION METHODS *****************************
    // ************************************************************************

    /// Whether the location is clear for a unit to occupy, either by movement
    /// or by construction.
    ///
    /// * GameError::InvalidLocation - the location is outside the vision range.
    pub fn is_occupiable(&self, location: MapLocation) -> Result<bool, Error> {
        if !self.is_on_map(location) {
            return Err(GameError::LocationOffMap)?;
        }
        if !self.can_sense_location(location) {
            return Err(GameError::LocationNotVisible)?;
        }

        let planet_map = &self.starting_map(location.planet);
        Ok(planet_map.is_passable_terrain_at(location)? &&
            !self.my_planet().units_by_loc.contains_key(&location))
    }

    pub fn is_on_map(&self, location: MapLocation) -> bool {
        if let Some(map) = self.planet_maps.get(&self.planet()) {
            map.on_map(location)
        } else {
            false
        }
    }

    fn ok_if_can_move(&self, robot_id: UnitID, direction: Direction) -> Result<(), Error> {
        let unit = self.my_unit(robot_id)?;
        let new_location = unit.location().map_location()?.add(direction);
        if !self.starting_map(new_location.planet).on_map(new_location) {
            Err(GameError::LocationOffMap)?;
        }
        if !self.is_occupiable(new_location)? {
            Err(GameError::LocationNotEmpty)?;
        }
        Ok(())
    }

    /// Whether the robot can move in the given direction, without taking into
    /// account the unit's movement heat. Takes into account only the map
    /// terrain, positions of other robots, and the edge of the game map.
    pub fn can_move(&self, robot_id: UnitID, direction: Direction) -> bool {
        self.ok_if_can_move(robot_id, direction).is_ok()
    }

    fn ok_if_move_ready(&self, robot_id: UnitID) -> Result<(), Error> {
        self.my_unit(robot_id)?.ok_if_move_ready()?;
        Ok(())
    }

    /// Whether the robot is ready to move. Tests whether the robot's attack
    /// heat is sufficiently low.
    pub fn is_move_ready(&self, robot_id: UnitID) -> bool {
        self.ok_if_move_ready(robot_id).is_ok()
    }

    /// Moves the robot in the given direction.
    ///
    /// * GameError::InappropriateUnitType - the unit is not a robot.
    /// * GameError::LocationNotEmpty - the location in the given direction is occupied.
    /// * GameError::LocationOffMap - the location in the given direction is off the map.
    /// * GameError::NoSuchUnit - the robot does not exist (within the vision range).
    /// * GameError::Overheated - the robot is not ready to move again.
    /// * GameError::TeamNotAllowed - the robot is not on the current player's team.
    /// * GameError::UnitNotOnMap - the robot is not on the map.
    pub fn move_robot(&mut self, robot_id: UnitID, direction: Direction) -> Result<(), Error> {
        self.ok_if_can_move(robot_id, direction)?;
        self.ok_if_move_ready(robot_id)?;
        let dest = match self.my_unit(robot_id)?.location() {
            OnMap(loc) => loc.add(direction),
            _ => unreachable!(),
        };
        self.move_to(robot_id, dest)
    }

    fn move_to(&mut self, _robot_id: UnitID, _location: MapLocation) -> Result<(), Error> {
        self.remove_unit(_robot_id);
        self.my_unit_mut(_robot_id)?.move_to(_location);
        self.place_unit(_robot_id);
        Ok(())
    }

    // ************************************************************************
    // *************************** ATTACK METHODS *****************************
    // ************************************************************************

    fn damage_unit(&mut self, unit_id: UnitID, damage: i32) {
        // The unit controller is always in the dev engine, but is only in the
        // player engine if the unit is on this team.
        if self.get_unit_mut(unit_id).is_ok() {
            self.get_unit_mut(unit_id).unwrap().take_damage(damage);
        }

        let should_destroy_unit = {
            let unit_info = self.unit_info_mut(unit_id).expect("unit exists");
            unit_info.health = ((unit_info.health as i32) - damage) as u32;
            unit_info.health == 0
        };

        if should_destroy_unit {
            self.destroy_unit(unit_id);
        }
    }

    /// Deals damage to any unit in the target square, potentially destroying it.
    fn damage_location(&mut self, location: MapLocation, damage: i32) {
        let id = if let Some(id) = self.my_planet().units_by_loc.get(&location) {
            *id
        } else {
            return;
        };

        self.damage_unit(id, damage)
    }

    fn ok_if_can_attack(&self, robot_id: UnitID, target_id: UnitID) -> Result<(), Error> {
        self.my_unit(robot_id)?.ok_if_on_map()?;
        let target_loc = self.unit_info(target_id)?.location;
        if !target_loc.on_map() {
            Err(GameError::UnitNotOnMap)?;
        }
        self.my_unit(robot_id)?.ok_if_within_attack_range(target_loc)?;
        Ok(())
    }

    /// Whether the robot can attack the given unit, without taking into
    /// account the unit's attack heat. Takes into account only the unit's
    /// attack range, and the location of the unit.
    pub fn can_attack(&self, robot_id: UnitID, target_id: UnitID) -> bool {
        self.ok_if_can_attack(robot_id, target_id).is_ok()
    }

    fn ok_if_attack_ready(&self, robot_id: UnitID) -> Result<(), Error> {
        self.my_unit(robot_id)?.ok_if_attack_ready()?;
        Ok(())
    }

    /// Whether the robot is ready to attack. Tests whether the robot's attack
    /// heat is sufficiently low.
    pub fn is_attack_ready(&self, robot_id: UnitID) -> bool {
        self.ok_if_attack_ready(robot_id).is_ok()
    }

    /// Commands a robot to attack a unit, dealing the 
    /// robot's standard amount of damage.
    ///
    /// * GameError::InappropriateUnitType - the unit is not a robot.
    /// * GameError::NoSuchUnit - either unit does not exist (inside the vision range).
    /// * GameError::OutOfRange - the target does not lie within attack range of the robot.
    /// * GameError::Overheated - the robot is not ready to attack again.
    /// * GameError::TeamNotAllowed - the robot is not on the current player's team.
    /// * GameError::UnitNotOnMap - either unit is not on the map.
    pub fn attack(&mut self, robot_id: UnitID, target_id: UnitID) -> Result<(), Error> {
        self.ok_if_can_attack(robot_id, target_id)?;
        self.ok_if_attack_ready(robot_id)?;
        let damage = self.my_unit_mut(robot_id)?.use_attack();
        if self.my_unit(robot_id)?.unit_type() == UnitType::Mage {
            let epicenter = self.unit_info(target_id)?.location.map_location()?;
            for direction in Direction::all().iter() {
                self.damage_location(epicenter.add(*direction), damage);
            }
        }
        self.damage_unit(target_id, damage);
        Ok(())
    }

    // ************************************************************************
    // ************************* RESEARCH METHODS *****************************
    // ***********************f*************************************************

    /// Returns research info of the current player.
    fn my_research(&self) -> ResearchInfo {
        self.my_team().research.clone()
    }

    /// Returns mutable research info of the current player.
    fn my_research_mut(&mut self) -> &mut ResearchInfo {
        &mut self.my_team_mut().research
    }

    /// The research info of the current team, including what branch is
    /// currently being researched, the number of rounds left.
    pub fn research_info(&self) -> ResearchInfo {
        self.my_research()
    }

    /// Resets the research queue to be empty. Returns true if the queue was
    /// not empty before, and false otherwise.
    pub fn reset_research(&mut self) -> bool {
        self.my_research_mut().reset_queue()
    }

    /// Adds a branch to the back of the queue, if it is a valid upgrade, and
    /// starts research if it is the first in the queue.
    ///
    /// Returns whether the branch was successfully added.
    pub fn queue_research(&mut self, branch: Branch) -> bool {
        self.my_research_mut().add_to_queue(&branch)
    }

    /// Update the current research and process any completed upgrades.
    fn process_research(&mut self, team: Team) {
        if let Some(branch) = self.get_team_mut(team).research.end_round() {
            for (_, unit) in self.get_planet_mut(Planet::Earth).units.iter_mut() {
                if unit.unit_type() == branch {
                    unit.research().expect("research level is valid");
                }
            }
            for (_, unit) in self.get_planet_mut(Planet::Mars).units.iter_mut() {
                if unit.unit_type() == branch {
                    unit.research().expect("research level is valid");
                }
            }
        }
    }

    // ************************************************************************
    // *************************** WORKER METHODS *****************************
    // ************************************************************************

    fn ok_if_can_harvest(&self, worker_id: UnitID, direction: Direction) -> Result<(), Error> {
        let unit = self.my_unit(worker_id)?;
        unit.ok_if_can_worker_act()?;
        let harvest_loc = unit.location().map_location()?.add(direction);
        // Check to see if we can sense the harvest location, (e.g. it is on the map).
        if !self.is_on_map(harvest_loc) {
            Err(GameError::LocationOffMap)?;
        }
        if self.karbonite_at(harvest_loc)? == 0 {
            Err(GameError::KarboniteDepositEmpty)?;
        }
        Ok(())
    }

    /// Whether the worker is ready to harvest, and the given direction contains
    /// karbonite to harvest. The worker cannot already have performed an action 
    /// this round.
    pub fn can_harvest(&self, worker_id: UnitID, direction: Direction) -> bool {
        self.ok_if_can_harvest(worker_id, direction).is_ok()
    }

    /// Harvests up to the worker's harvest amount of karbonite from the given
    /// location, adding it to the team's resource pool.
    ///
    /// * GameError::InappropriateUnitType - the unit is not a worker.
    /// * GameError::KarboniteDepositEmpty - the location described contains no Karbonite.
    /// * GameError::LocationOffMap - the location in the target direction is off the map.
    /// * GameError::NoSuchUnit - the worker does not exist (within the vision range).
    /// * GameError::Overheated - the worker has already performed an action this turn.
    /// * GameError::TeamNotAllowed - the worker is not on the current player's team.
    /// * GameError::UnitNotOnMap - the worker is not on the map.
    pub fn harvest(&mut self, worker_id: UnitID, direction: Direction)
                   -> Result<(), Error> {
        self.ok_if_can_harvest(worker_id, direction)?;
        let (harvest_loc, harvest_amount) = {
            let worker = self.my_unit_mut(worker_id)?;
            worker.worker_act();
            (worker.location().map_location()?.add(direction), worker.worker_harvest_amount()?)
        };
        let amount_mined = cmp::min(self.karbonite_at(harvest_loc)?, harvest_amount);
        self.my_team_mut().karbonite += amount_mined;
        self.my_planet_mut().karbonite[harvest_loc.y as usize][harvest_loc.x as usize] -= amount_mined;
        Ok(())
    }

    fn ok_if_can_blueprint(&self, worker_id: UnitID, unit_type: UnitType,
                         direction: Direction) -> Result<(), Error> {
        // Players should never attempt to build a non-structure.
        if !unit_type.is_structure() {
            Err(GameError::InappropriateUnitType)?;
        }
        let unit = self.my_unit(worker_id)?;
        unit.ok_if_can_worker_act()?;
        let build_loc = unit.location().map_location()?.add(direction);
        // Check to see if we can sense the build location, (e.g. it is on the map).
        if !self.can_sense_location(build_loc) {
            Err(GameError::LocationNotVisible)?;
        }
        // The build location must be unoccupied.
        if !self.is_occupiable(build_loc)? {
            Err(GameError::LocationNotEmpty)?;
        }
        // Structures can never be built on Mars.
        if build_loc.planet == Planet::Mars {
            Err(GameError::CannotBuildOnMars)?;
        }
        // If building a rocket, Rocketry must be unlocked.
        if unit_type == UnitType::Rocket && self.my_research().get_level(&unit_type) < 1 {
            Err(GameError::ResearchNotUnlocked)?;
        }
        // Finally, the team must have sufficient karbonite.
        if self.karbonite() < unit_type.blueprint_cost()? {
            Err(GameError::InsufficientKarbonite)?;
        }
        Ok(())
    }

    /// Whether the worker can blueprint a structure of the given type. The worker
    /// can only blueprint factories, and rockets if Rocketry has been
    /// researched. The team must have sufficient karbonite in its resource
    /// pool. The worker cannot already have performed an action this round.
    pub fn can_blueprint(&self, worker_id: UnitID, unit_type: UnitType,
                         direction: Direction) -> bool {
        self.ok_if_can_blueprint(worker_id, unit_type, direction).is_ok()
    }

    /// Blueprints a unit of the given type in the given direction. Subtract
    /// cost of that unit from the team's resource pool.
    ///
    /// * GameError::CannotBuildOnMars - you cannot blueprint a structure on Mars.
    /// * GameError::InappropriateUnitType - the unit is not a worker, or the unit type
    ///   is not a structure.
    /// * GameError::InsufficientKarbonite - your team does not have enough Karbonite to
    ///   build the requested structure.
    /// * GameError::LocationOffMap - the location in the target direction is off the map.
    /// * GameError::LocationNotEmpty - the location in the target direction is already
    ///   occupied.
    /// * GameError::NoSuchUnit - the worker does not exist (within the vision range).
    /// * GameError::Overheated - the worker has already performed an action this turn.
    /// * GameError::ResearchNotUnlocked - you do not have the needed research to blueprint rockets.
    /// * GameError::TeamNotAllowed - the worker is not on the current player's team.
    /// * GameError::UnitNotOnMap - the unit is not on the map.
    pub fn blueprint(&mut self, worker_id: UnitID, unit_type: UnitType,
                     direction: Direction) -> Result<(), Error> {
        self.ok_if_can_blueprint(worker_id, unit_type, direction)?;
        let build_loc = {
            let worker = self.my_unit_mut(worker_id)?;
            worker.worker_act();
            worker.location().map_location()?.add(direction)
        };
        let team = self.team();
        self.create_unit(team, build_loc, unit_type)?;
        self.my_team_mut().karbonite -= unit_type.blueprint_cost()?;
        Ok(())
    }

    fn ok_if_can_build(&self, worker_id: UnitID, blueprint_id: UnitID)
                       -> Result<(), Error> {
        let worker = self.my_unit(worker_id)?;
        let blueprint = self.my_unit(blueprint_id)?;
        // The worker must be on the map.
        worker.ok_if_on_map()?;
        // The worker must be able to act.
        worker.ok_if_can_worker_act()?;
        // The worker must be adjacent to the blueprint.
        if !worker.location().is_adjacent_to(blueprint.location()) {
            Err(GameError::OutOfRange)?;
        }
        // The blueprint must be incomplete.
        if blueprint.structure_is_built()? {
            Err(GameError::StructureAlreadyBuilt)?;
        }
        Ok(())
    }

    /// Whether the worker can build a blueprint with the given ID. The worker
    /// and the blueprint must be adjacent to each other. The worker cannot
    /// already have performed an action this round.
    pub fn can_build(&self, worker_id: UnitID, blueprint_id: UnitID) -> bool {
        self.ok_if_can_build(worker_id, blueprint_id).is_ok()
    }

    /// Builds a given blueprint, increasing its health by the worker's build
    /// amount. If raised to maximum health, the blueprint becomes a completed
    /// structure.
    ///
    /// * GameError::InappropriateUnitType - the unit is not a worker, or the blueprint
    ///   is not a structure.
    /// * GameError::NoSuchUnit - either unit does not exist (within the vision range).
    /// * GameError::OutOfRange - the worker is not adjacent to the blueprint.
    /// * GameError::Overheated - the worker has already performed an action this turn.
    /// * GameError::StructureAlreadyBuilt - the blueprint has already been completed.
    /// * GameError::TeamNotAllowed - either unit is not on the current player's team.
    /// * GameError::UnitNotOnMap - the worker is not on the map.
    pub fn build(&mut self, worker_id: UnitID, blueprint_id: UnitID)
                 -> Result<(), Error> {
        self.ok_if_can_build(worker_id, blueprint_id)?;
        let build_health = {
            let worker = self.my_unit_mut(worker_id)?;
            worker.worker_act();
            worker.worker_build_health()?
        };
        self.my_unit_mut(blueprint_id)?.be_built(build_health);
        Ok(())
    }

    fn ok_if_can_repair(&self, worker_id: UnitID, structure_id: UnitID) -> Result<(), Error> {
        let worker = self.my_unit(worker_id)?;
        let structure = self.my_unit(structure_id)?;
        worker.ok_if_on_map()?;
        worker.ok_if_can_worker_act()?;
        if !worker.location().is_adjacent_to(structure.location()) {
            Err(GameError::OutOfRange)?;
        }
        if !structure.structure_is_built()? {
            Err(GameError::StructureNotYetBuilt)?;
        }
        Ok(())
    }

    /// Whether the given worker can repair the given strucutre. Tests that the worker
    /// is able to execute a worker action, that the structure is built, and that the
    /// structure is within range.
    pub fn can_repair(&self, worker_id: UnitID, structure_id: UnitID) -> bool {
        self.ok_if_can_repair(worker_id, structure_id).is_ok()
    }

    /// Commands the worker to repair a structure, repleneshing health to it. This
    /// can only be done to structures which have been fully built.
    ///
    /// * GameError::InappropriateUnitType - the unit is not a worker, or the target
    ///   is not a structure.
    /// * GameError::NoSuchUnit - either unit does not exist (within the vision range).
    /// * GameError::OutOfRange - the worker is not adjacent to the structure.
    /// * GameError::Overheated - the worker has already performed an action this turn.
    /// * GameError::StructureNotYetBuilt - the structure has not been completed.
    /// * GameError::TeamNotAllowed - either unit is not on the current player's team.
    /// * GameError::UnitNotOnMap - the worker is not on the map.
    pub fn repair(&mut self, worker_id: UnitID, structure_id: UnitID) -> Result<(), Error> {
        self.ok_if_can_repair(worker_id, structure_id)?;
        self.my_unit_mut(worker_id)?.worker_act();
        self.my_unit_mut(structure_id)?.be_healed(WORKER_REPAIR_AMOUNT);
        Ok(())
    }

    fn ok_if_can_replicate(&self, worker_id: UnitID, direction: Direction) 
                           -> Result<(), Error> {
        let worker = self.my_unit(worker_id)?;
        worker.ok_if_on_map()?;
        worker.ok_if_ability_ready()?;
        if self.karbonite() < worker.unit_type().replicate_cost()? {
            Err(GameError::InsufficientKarbonite)?;
        }
        let replicate_loc = worker.location().map_location()?.add(direction);
        if !self.is_on_map(replicate_loc) {
            Err(GameError::LocationOffMap)?;
        }
        if !self.is_occupiable(replicate_loc)? {
            Err(GameError::LocationNotEmpty)?;
        }
        Ok(())
    }

    /// Whether the worker is ready to replicate. Tests that the worker's
    /// ability heat is sufficiently low, that the team has sufficient
    /// karbonite in its resource pool, and that the square in the given
    /// direction is empty.
    pub fn can_replicate(&self, worker_id: UnitID, direction: Direction) -> bool {
        self.ok_if_can_replicate(worker_id, direction).is_ok()
    }

    /// Replicates a worker in the given direction. Subtracts the cost of the
    /// worker from the team's resource pool.
    ///
    /// * GameError::InappropriateUnitType - the unit is not a worker.
    /// * GameError::InsufficientKarbonite - your team does not have enough Karbonite for
    ///   the worker to replicate.
    /// * GameError::LocationOffMap - the location in the target direction is off the map.
    /// * GameError::LocationNotEmpty - the location in the target direction is already
    ///   occupied.
    /// * GameError::NoSuchUnit - the worker does not exist (within the vision range).
    /// * GameError::Overheated - the worker is not ready to replicate again.
    /// * GameError::TeamNotAllowed - the worker is not on the current player's team.
    /// * GameError::UnitNotOnMap - the worker is not on the map.
    pub fn replicate(&mut self, worker_id: UnitID, direction: Direction)
                     -> Result<(), Error> {
        self.ok_if_can_replicate(worker_id, direction)?;
        self.my_unit_mut(worker_id)?.worker_act();
        self.my_unit_mut(worker_id)?.replicate();
        let (team, location) = {
            let worker = self.my_unit(worker_id)?;
            (worker.team(), worker.location().map_location()?.add(direction))
        };
        self.create_unit(team, location, UnitType::Worker)?;
        self.my_team_mut().karbonite -= UnitType::Worker.replicate_cost()?;
        Ok(())
    }

    // ************************************************************************
    // *************************** KNIGHT METHODS *****************************
    // ************************************************************************

    fn ok_if_can_javelin(&self, knight_id: UnitID, target_id: UnitID) -> Result<(), Error> {
        let knight = self.my_unit(knight_id)?;
        let target = self.unit_info(target_id)?;
        knight.ok_if_on_map()?;
        knight.ok_if_javelin_unlocked()?;
        knight.ok_if_within_ability_range(target.location)?;
        Ok(())
    }

    /// Whether the knight can javelin the given robot, without taking into
    /// account the knight's ability heat. Takes into account only the knight's
    /// ability range, and the location of the robot.
    pub fn can_javelin(&self, knight_id: UnitID, target_id: UnitID) -> bool {
        self.ok_if_can_javelin(knight_id, target_id).is_ok()
    }

    fn ok_if_javelin_ready(&self, knight_id: UnitID) -> Result<(), Error> {
        let knight = self.my_unit(knight_id)?;
        knight.ok_if_javelin_unlocked()?;
        knight.ok_if_ability_ready()?;
        Ok(())
    }

    /// Whether the knight is ready to javelin. Tests whether the knight's
    /// ability heat is sufficiently low.
    pub fn is_javelin_ready(&self, knight_id: UnitID) -> bool {
       self.ok_if_javelin_ready(knight_id).is_ok()
    }

    /// Javelins the robot, dealing the knight's standard damage.
    ///
    /// * GameError::InappropriateUnitType - the unit is not a knight.
    /// * GameError::NoSuchUnit - either unit does not exist (inside the vision range).
    /// * GameError::OutOfRange - the target does not lie within ability range of the knight.
    /// * GameError::Overheated - the knight is not ready to use javelin again.
    /// * GameError::ResearchNotUnlocked - you do not have the needed research to use javelin.
    /// * GameError::TeamNotAllowed - the knight is not on the current player's team.
    /// * GameError::UnitNotOnMap - the knight is not on the map.
    pub fn javelin(&mut self, knight_id: UnitID, target_id: UnitID) -> Result<(), Error> {
        self.ok_if_can_javelin(knight_id, target_id)?;
        self.ok_if_javelin_ready(knight_id)?;
        let damage = self.my_unit_mut(knight_id)?.javelin();
        self.damage_unit(target_id, damage);
        Ok(())
    }

    // ************************************************************************
    // *************************** RANGER METHODS *****************************
    // ************************************************************************

    fn ok_if_begin_snipe_ready(&self, ranger_id: UnitID) -> Result<(), Error> {
        let ranger = self.my_unit(ranger_id)?;
        ranger.ok_if_snipe_unlocked()?;
        ranger.ok_if_ability_ready()?;
        Ok(())
    }

    /// Whether the ranger is ready to begin snipe. Tests whether the ranger's
    /// ability heat is sufficiently low.
    pub fn is_begin_snipe_ready(&self, ranger_id: UnitID) -> bool {
        self.ok_if_begin_snipe_ready(ranger_id).is_ok()
    }

    /// Begins the countdown to snipe a given location. Maximizes the units
    /// attack and movement heats until the ranger has sniped. The ranger may
    /// begin the countdown at any time, including resetting the countdown
    /// to snipe a different location.
    ///
    /// * GameError::InappropriateUnitType - the unit is not a ranger.
    /// * GameError::LocationOffMap - the target location is not on this planet's map.
    /// * GameError::NoSuchUnit - the ranger does not exist (inside the vision range).
    /// * GameError::Overheated - the ranger is not ready to use snipe again.
    /// * GameError::ResearchNotUnlocked - you do not have the needed research to use snipe.
    /// * GameError::TeamNotAllowed - the ranger is not on the current player's team.
    /// * GameError::UnitNotOnMap - the ranger is not on the map.
    pub fn begin_snipe(&mut self, ranger_id: UnitID, location: MapLocation)
                       -> Result<(), Error> {
        if !self.is_on_map(location) {
            Err(GameError::LocationOffMap)?
        }
        self.my_unit(ranger_id)?.ok_if_on_map()?;
        self.ok_if_begin_snipe_ready(ranger_id)?;
        self.my_unit_mut(ranger_id)?.begin_snipe(location);
        Ok(())
    }

    fn process_rangers(&mut self, planet: Planet) {
        let mut rangers: Vec<UnitID> = vec![];
        for unit in self.get_planet(planet).units.values() {
            if unit.unit_type() == UnitType::Ranger {
                rangers.push(unit.id());
            }
        }

        for id in rangers {
            let target_location = self.get_planet_mut(planet).units.get_mut(&id).unwrap().process_snipe();
            if target_location.is_some() {
                let damage = self.get_planet(planet).units.get(&id).unwrap().damage().unwrap();
                self.damage_location(target_location.unwrap(), damage);
                self.viewer_changes.push(ViewerDelta::RangerSnipe { 
                    ranger_id: id, 
                    target_location: target_location.unwrap(), 
                });
            }
        }
    }

    // ************************************************************************
    // **************************** MAGE METHODS ******************************
    // ************************************************************************
    
    fn ok_if_can_blink(&self, mage_id: UnitID, location: MapLocation) -> Result<(), Error> {
        let mage = self.my_unit(mage_id)?;
        mage.ok_if_on_map()?;
        mage.ok_if_blink_unlocked()?;
        mage.ok_if_within_ability_range(OnMap(location))?;
        if !self.is_occupiable(location)? {
            Err(GameError::LocationNotEmpty)?;
        }
        Ok(())
    }

    /// Whether the mage can blink to the given location, without taking into
    /// account the mage's ability heat. Takes into account only the mage's
    /// ability range, the map terrain, positions of other units, and the edge
    /// of the game map.
    pub fn can_blink(&self, mage_id: UnitID, location: MapLocation) -> bool {
        self.ok_if_can_blink(mage_id, location).is_ok()
    }

    fn ok_if_blink_ready(&self, mage_id: UnitID) -> Result<(), Error> {
        let mage = self.my_unit(mage_id)?;
        mage.ok_if_blink_unlocked()?;
        mage.ok_if_ability_ready()?;
        Ok(())
    }

    /// Whether the mage is ready to blink. Tests whether the mage's ability
    /// heat is sufficiently low.
    pub fn is_blink_ready(&self, mage_id: UnitID) -> bool {
        self.ok_if_blink_ready(mage_id).is_ok()
    }

    /// Blinks the mage to the given location.
    ///
    /// * GameError::InappropriateUnitType - the unit is not a mage.
    /// * GameError::LocationOffMap - the target location is not on this planet's map.
    /// * GameError::LocationNotEmpty - the target location is already occupied.
    /// * GameError::NoSuchUnit - the mage does not exist (inside the vision range).
    /// * GameError::OutOfRange - the target does not lie within ability range of the mage.
    /// * GameError::Overheated - the mage is not ready to use blink again.
    /// * GameError::ResearchNotUnlocked - you do not have the needed research to use blink.
    /// * GameError::TeamNotAllowed - the mage is not on the current player's team.
    /// * GameError::UnitNotOnMap - the mage is not on the map.
    pub fn blink(&mut self, mage_id: UnitID, location: MapLocation) -> Result<(), Error> {
        self.ok_if_can_blink(mage_id, location)?;
        self.ok_if_blink_ready(mage_id)?;
        self.remove_unit(mage_id);
        self.my_unit_mut(mage_id)?.blink(location);
        self.place_unit(mage_id);
        Ok(())
    }

    // ************************************************************************
    // *************************** HEALER METHODS *****************************
    // ************************************************************************

    fn ok_if_can_heal(&self, healer_id: UnitID, robot_id: UnitID) -> Result<(), Error> {
        self.ok_if_can_attack(healer_id, robot_id)?;
        self.my_unit(robot_id)?.ok_if_robot()?;
        Ok(())
    }

    /// Whether the healer can heal the given robot, without taking into
    /// account the healer's attack heat. Takes into account only the healer's
    /// attack range, and the location of the robot.
    pub fn can_heal(&self, healer_id: UnitID, robot_id: UnitID) -> bool {
        self.ok_if_can_heal(healer_id, robot_id).is_ok()
    }

    fn ok_if_heal_ready(&self, healer_id: UnitID) -> Result<(), Error> {
        Ok(self.ok_if_attack_ready(healer_id)?)
    }

    /// Whether the healer is ready to heal. Tests whether the healer's attack
    /// heat is sufficiently low.
    pub fn is_heal_ready(&self, healer_id: UnitID) -> bool {
        self.ok_if_heal_ready(healer_id).is_ok()
    }

    /// Commands the healer to heal the target robot.
    ///
    /// * GameError::InappropriateUnitType - the unit is not a healer, or the target is not
    ///   a robot.
    /// * GameError::NoSuchUnit - either unit does not exist (inside the vision range).
    /// * GameError::OutOfRange - the target does not lie within "attack" range of the healer.
    /// * GameError::Overheated - the healer is not ready to heal again.
    /// * GameError::TeamNotAllowed - either robot is not on the current player's team.
    /// * GameError::UnitNotOnMap - the healer is not on the map.
    pub fn heal(&mut self, healer_id: UnitID, robot_id: UnitID) -> Result<(), Error> {
        self.ok_if_can_heal(healer_id, robot_id)?;
        self.ok_if_heal_ready(healer_id)?;
        self.attack(healer_id, robot_id)?;
        Ok(())
    }

    fn ok_if_can_overcharge(&self, healer_id: UnitID, robot_id: UnitID)
                            -> Result<(), Error> {
        let healer = self.my_unit(healer_id)?;
        let robot = self.my_unit(robot_id)?;
        healer.ok_if_on_map()?;
        healer.ok_if_overcharge_unlocked()?;
        healer.ok_if_within_ability_range(robot.location())?;
        robot.ok_if_robot()?;
        Ok(())
    }

    /// Whether the healer can overcharge the given robot, without taking into
    /// account the healer's ability heat. Takes into account only the healer's
    /// ability range, and the location of the robot.
    pub fn can_overcharge(&self, healer_id: UnitID, robot_id: UnitID) -> bool {
        self.ok_if_can_overcharge(healer_id, robot_id).is_ok()                      
    }

    fn ok_if_overcharge_ready(&self, healer_id: UnitID) -> Result<(), Error> {
        let healer = self.my_unit(healer_id)?;
        healer.ok_if_overcharge_unlocked()?;
        healer.ok_if_ability_ready()?;
        Ok(())
    }

    /// Whether the healer is ready to overcharge. Tests whether the healer's
    /// ability heat is sufficiently low.
    pub fn is_overcharge_ready(&self, healer_id: UnitID) -> bool {
        self.ok_if_overcharge_ready(healer_id).is_ok()
    }

    /// Overcharges the robot, resetting the robot's cooldowns.
    ///
    /// * GameError::InappropriateUnitType - the unit is not a healer, or the target is not
    ///   a robot.
    /// * GameError::NoSuchUnit - either unit does not exist (inside the vision range).
    /// * GameError::OutOfRange - the target does not lie within ability range of the healer.
    /// * GameError::Overheated - the healer is not ready to use overcharge again.
    /// * GameError::ResearchNotUnlocked - you do not have the needed research to use overcharge.
    /// * GameError::TeamNotAllowed - either robot is not on the current player's team.
    /// * GameError::UnitNotOnMap - the healer is not on the map.
    pub fn overcharge(&mut self, healer_id: UnitID, robot_id: UnitID)
                      -> Result<(), Error> {
        self.ok_if_can_overcharge(healer_id, robot_id)?;
        self.ok_if_overcharge_ready(healer_id)?;
        self.my_unit_mut(healer_id)?.overcharge();
        self.my_unit_mut(robot_id)?.be_overcharged();
        Ok(())
    }

    // ************************************************************************
    // ************************* STRUCTURE METHODS ****************************
    // ************************************************************************

    fn ok_if_can_load(&self, structure_id: UnitID, robot_id: UnitID)
                      -> Result<(), Error> {
        let robot = self.my_unit(robot_id)?;
        let structure = self.my_unit(structure_id)?;
        robot.ok_if_on_map()?;
        structure.ok_if_on_map()?;
        robot.ok_if_move_ready()?;
        structure.ok_if_can_load()?;
        if !structure.location().is_adjacent_to(robot.location()) {
            Err(GameError::OutOfRange)?;
        }
        Ok(())
    }

    /// Whether the robot can be loaded into the given structure's garrison. The robot
    /// must be ready to move and must be adjacent to the structure. The structure
    /// and the robot must be on the same team, and the structure must have space.
    pub fn can_load(&self, structure_id: UnitID, robot_id: UnitID) -> bool {
        self.ok_if_can_load(structure_id, robot_id).is_ok()
    }

    /// Loads the robot into the garrison of the structure.
    ///
    /// * GameError::GarrisonFull - the structure's garrison is already full.
    /// * GameError::InappropriateUnitType - the first unit is not a structure, or the
    ///   second unit is not a robot.
    /// * GameError::NoSuchUnit - either unit does not exist (inside the vision range).
    /// * GameError::Overheated - the robot is not ready to move again.
    /// * GameError::OutOfRange - the robot is not adjacent to the structure.
    /// * GameError::TeamNotAllowed - either unit is not on the current player's team.
    /// * GameError::StructureNotYetBuilt - the structure has not yet been completed.
    /// * GameError::UnitNotOnMap - either unit is not on the map.
    pub fn load(&mut self, structure_id: UnitID, robot_id: UnitID)
                    -> Result<(), Error> {
        self.ok_if_can_load(structure_id, robot_id)?;
        self.remove_unit(robot_id);
        self.my_unit_mut(structure_id)?.load(robot_id);
        self.my_unit_mut(robot_id)?.board_rocket(structure_id);
        self.place_unit(robot_id);
        Ok(())
    }

    fn ok_if_can_unload(&self, structure_id: UnitID, direction: Direction)
                        -> Result<(), Error> {
        let structure = self.my_unit(structure_id)?;
        structure.ok_if_on_map()?;
        structure.ok_if_can_unload_unit()?;
        let robot = self.my_unit(structure.structure_garrison()?[0])?;
        let loc = structure.location().map_location()?.add(direction);
        if !self.is_occupiable(loc)? {
            Err(GameError::LocationNotEmpty)?;
        }
        robot.ok_if_move_ready()?;
        Ok(())
    }

    /// Tests whether the given structure is able to unload a unit in the
    /// given direction. There must be space in that direction, and the unit
    /// must be ready to move.
    pub fn can_unload(&self, structure_id: UnitID, direction: Direction) -> bool {
        self.ok_if_can_unload(structure_id, direction).is_ok()
    }

    /// Unloads a robot from the garrison of the specified structure into an 
    /// adjacent space. Robots are unloaded in the order they were loaded.
    ///
    /// * GameError::GarrisonEmpty - the structure's garrison is already empty.
    /// * GameError::InappropriateUnitType - the unit is not a structure.
    /// * GameError::LocationOffMap - the location in the target direction is off the map.
    /// * GameError::LocationNotEmpty - the location in the target direction is already
    ///   occupied.
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::Overheated - the robot inside the structure is not ready to move again.
    /// * GameError::StructureNotYetBuilt - the structure has not yet been completed.
    /// * GameError::TeamNotAllowed - either unit is not on the current player's team.
    /// * GameError::UnitNotOnMap - the structure is not on the map.
    pub fn unload(&mut self, structure_id: UnitID, direction: Direction)
                  -> Result<(), Error> {
        self.ok_if_can_unload(structure_id, direction)?;
        let (robot_id, structure_loc) = {
            let structure = self.my_unit_mut(structure_id)?;
            (structure.unload_unit(), structure.location().map_location()?)
        };
        let robot_loc = structure_loc.add(direction);
        self.my_unit_mut(robot_id)?.move_to(robot_loc);
        self.place_unit(robot_id);
        Ok(())
    }

    // ************************************************************************
    // ************************** FACTORY METHODS *****************************
    // ************************************************************************

    fn ok_if_can_produce_robot(&self, factory_id: UnitID, robot_type: UnitType)
                                 -> Result<(), Error> {
        let factory = self.my_unit(factory_id)?;
        factory.ok_if_can_produce_robot(robot_type)?;
        let cost = robot_type.factory_cost().expect("unit type is ok");
        if self.karbonite() < cost {
            Err(GameError::InsufficientKarbonite)?;
        }
        Ok(())
    }

    /// Whether the factory can produce a robot of the given type. The factory
    /// must not currently be producing a robot, and the team must have
    /// sufficient resources in its resource pool.
    pub fn can_produce_robot(&mut self, factory_id: UnitID, robot_type: UnitType) -> bool {
        self.ok_if_can_produce_robot(factory_id, robot_type).is_ok()        
    }

    /// Starts producing the robot of the given type.
    ///
    /// * GameError::FactoryBusy - the factory is already producing a unit.
    /// * GameError::InappropriateUnitType - the unit is not a factory, or the unit type
    ///   is not a robot.
    /// * GameError::InsufficientKarbonite - your team does not have enough Karbonite to
    ///   produce the given robot.
    /// * GameError::NoSuchUnit - the factory does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the factory is not on the current player's team.
    /// * GameError::StructureNotYetBuilt - the factory has not yet been completed.
    pub fn produce_robot(&mut self, factory_id: UnitID, robot_type: UnitType)
                       -> Result<(), Error> {
        self.ok_if_can_produce_robot(factory_id, robot_type)?;
        self.my_team_mut().karbonite -= robot_type.factory_cost().expect("unit type is ok");
        let factory = self.my_unit_mut(factory_id).expect("factory exists");
        factory.produce_robot(robot_type);
        Ok(())
    }

    /// Process the end of the turn for factories. If a factory added a unit
    /// to its garrison, also mark that unit down in the game world.
    ///
    /// Note that factores cannot be built on Mars, so we only process Earth.
    fn process_factories(&mut self) {
        let planet = Planet::Earth;
        let mut factory_ids: Vec<UnitID> = vec![];
        for unit in self.get_planet(planet).unit_infos.values().into_iter() {
            if unit.unit_type == UnitType::Factory {
                factory_ids.push(unit.id);
            }
        }

        for factory_id in factory_ids {
            let (unit_type, team) = {
                let factory = self.get_unit_mut(factory_id).expect("unit exists");
                let new_unit_type = factory.process_factory_round();
                if new_unit_type.is_none() {
                    continue;
                }
                (new_unit_type.unwrap(), factory.team())
            };

            let id = self.id_generator.next_id();
            let level = self.get_team(team).research.get_level(&unit_type);
            let new_unit = Unit::new(id, team, unit_type, level, InGarrison(factory_id))
                .expect("research_level is valid");

            self.get_planet_mut(planet).unit_infos.insert(id, new_unit.info());
            self.get_planet_mut(planet).units.insert(id, new_unit);
            self.get_unit_mut(factory_id).unwrap().load(id);
        }
    }

    // ************************************************************************
    // *************************** ROCKET METHODS *****************************
    // ************************************************************************

    /// The landing rounds and locations of rockets in space that belong to the
    /// current team.
    pub fn rocket_landings(&self) -> RocketLandingInfo {
        self.my_team().rocket_landings.clone()
    }

    fn ok_if_can_launch_rocket(&self, rocket_id: UnitID, destination: MapLocation)
                               -> Result<(), Error> {
        if destination.planet == self.planet() {
            Err(GameError::SamePlanet)?;
        }
        let rocket = self.my_unit(rocket_id)?;
        rocket.ok_if_can_launch_rocket()?;
        let map = &self.starting_map(destination.planet);
        if !map.on_map(destination) {
            Err(GameError::LocationOffMap)?;
        }
        if !map.is_passable_terrain_at(destination)? {
            Err(GameError::LocationNotEmpty)?;
        }
        Ok(())
    }

    /// Whether the rocket can launch into space to the given destination. The
    /// rocket can launch if the it has never been used before. The destination
    /// is valid if it contains passable terrain on the other planet.
    pub fn can_launch_rocket(&self, rocket_id: UnitID, destination: MapLocation)
                             -> bool {
        self.ok_if_can_launch_rocket(rocket_id, destination).is_ok()
    }

    /// Launches the rocket into space, damaging the units adjacent to the
    /// takeoff location.
    ///
    /// * GameError::InappropriateUnitType - the unit is not a rocket.
    /// * GameError::LocationOffMap - the given location is off the map.
    /// * GameError::LocationNotEmpty - the given location contains impassable terrain.
    /// * GameError::NoSuchUnit - the rocket does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the rocket is not on the current player's team.
    /// * GameError::RocketUsed - the rocket has already been used.
    /// * GameError::SamePlanet - the rocket cannot fly to a location on the same planet.
    /// * GameError::StructureNotYetBuilt - the rocket has not yet been completed.
    pub fn launch_rocket(&mut self, rocket_id: UnitID, destination: MapLocation)
                         -> Result<(), Error> {
        self.ok_if_can_launch_rocket(rocket_id, destination)?;
        let takeoff_loc = self.my_unit(rocket_id)?.location().map_location()?;
        for dir in Direction::all() {
            self.damage_location(takeoff_loc.add(dir), ROCKET_BLAST_DAMAGE);
        }
        self.move_to_space(rocket_id);
        self.my_unit_mut(rocket_id)?.launch_rocket();

        let landing_round = self.round + self.orbit.duration(self.round)
            + self.my_unit(rocket_id)?.rocket_travel_time_decrease().unwrap();
        self.my_team_mut().rocket_landings.add_landing(
            landing_round, RocketLanding::new(rocket_id, destination)
        );
        Ok(())
    }

    /// Lands the rocket, damaging the units in adjacent squares. The rocket
    /// is destroyed if it lands on a factory, rocket, or impassable terrain.
    fn land_rocket(&mut self, rocket_id: UnitID, destination: MapLocation) {
        if self.my_planet().units_by_loc.contains_key(&destination) {
            let victim_id = *self.my_planet().units_by_loc.get(&destination).unwrap();
            let should_destroy_rocket = match self.unit_info(victim_id).unwrap().unit_type {
                UnitType::Rocket => true,
                UnitType::Factory => true,
                _ => false,
            };
            if should_destroy_rocket {
                self.destroy_unit(rocket_id);
            }
            self.destroy_unit(victim_id);
        } else {
            self.my_unit_mut(rocket_id).unwrap().land_rocket(destination);
            self.move_from_space(rocket_id);
        }

        for dir in Direction::all() {
            self.damage_location(destination.add(dir), ROCKET_BLAST_DAMAGE);
        }
    }

    fn process_rockets(&mut self, team: Team) {
        let landings = self.get_team(team).rocket_landings.landings_on(self.round);
        for landing in landings.iter() {
            self.land_rocket(landing.rocket_id, landing.destination);
            self.viewer_changes.push(ViewerDelta::RocketLanding { 
                rocket_id: landing.rocket_id, 
                location: landing.destination 
            });
        }
    }

    // ************************************************************************
    // ***************************** MANAGER API ******************************
    // ************************************************************************

    pub(crate) fn cached_world(&self, player: Player) -> &GameWorld {
        if let Some(world) = self.cached_world.get(&player) {
            world
        } else {
            unreachable!();
        }
    }

    pub(crate) fn initial_start_turn_message(&self) -> StartTurnMessage {
        let initial_player = Player::first_to_move();
        let world = self.cached_world(initial_player);
        if world.round != 1 {
            panic!("You should only get the initial STM on round 1.");
        }

        StartTurnMessage {
            round: world.round,
            visible_locs: world.my_planet().visible_locs.clone(),
            units_changed: vec![],
            units_vanished: vec![],
            unit_infos_changed: vec![],
            unit_infos_vanished: vec![],
            karbonite_changed: vec![],
            id_generator: world.id_generator.clone(),
            units_in_space_changed: vec![],
            units_in_space_vanished: vec![],
            other_array_changed: vec![],
            rocket_landings: world.my_team().rocket_landings.clone(),
            research: world.my_team().research.clone(),
            karbonite: world.my_team().karbonite,
        }
    }

    /// Updates the current player in the game. If a round of four turns has
    /// finished, also processes the end of the round. This includes updating
    /// unit cooldowns, rocket landings, asteroid strikes, research, etc. Returns 
    /// the next player to move, and whether the round was also ended.
    pub(crate) fn end_turn(&mut self) -> StartTurnMessage {
        use self::Team::*;
        use self::Planet::*;

        self.player_to_move = match self.player_to_move {
            Player { team: Red, planet: Earth } => Player::new(Blue, Earth),
            Player { team: Blue, planet: Earth } => Player::new(Red, Mars),
            Player { team: Red, planet: Mars } => Player::new(Blue, Mars),
            Player { team: Blue, planet: Mars } => {
                // This is the last player to move, so we can advance to the next round.
                self.end_round();
                Player::new(Red, Earth)
            },
        };

        let player = self.player_to_move;
        let world = self.filter(player);
        let mut stm = StartTurnMessage {
            round: world.round,
            visible_locs: world.my_planet().visible_locs.clone(),
            units_changed: vec![],
            units_vanished: vec![],
            unit_infos_changed: vec![],
            unit_infos_vanished: vec![],
            karbonite_changed: vec![],
            id_generator: world.id_generator.clone(),
            units_in_space_changed: vec![],
            units_in_space_vanished: vec![],
            other_array_changed: vec![],
            rocket_landings: world.my_team().rocket_landings.clone(),
            research: world.my_team().research.clone(),
            karbonite: world.my_team().karbonite,
        };
        {
            let old_world = self.cached_world.get(&player).unwrap();
            for (id, unit) in world.my_planet().units.iter() {
                if !old_world.my_planet().units.contains_key(&id) ||
                (old_world.my_planet().units.get(&id) != Some(&unit)) {
                    stm.units_changed.push(unit.clone());
                }
            }
            for id in old_world.my_planet().units.keys().into_iter() {
                if !world.my_planet().units.contains_key(&id) {
                    stm.units_vanished.push(*id);
                }
            }
            for (id, unit) in world.my_planet().unit_infos.iter() {
                if !old_world.my_planet().unit_infos.contains_key(&id) ||
                (old_world.my_planet().unit_infos.get(&id) != Some(&unit)) {
                    stm.unit_infos_changed.push(unit.clone());
                }
            }
            for id in old_world.my_planet().unit_infos.keys().into_iter() {
                if !world.my_planet().unit_infos.contains_key(&id) {
                    stm.unit_infos_vanished.push(*id);
                }
            }
            for (id, unit) in world.my_team().units_in_space.iter() {
                if !old_world.my_team().units_in_space.contains_key(&id) ||
                (old_world.my_team().units_in_space.get(&id) != Some(&unit)) {
                    stm.units_in_space_changed.push(unit.clone());
                }
            }
            for id in old_world.my_team().units_in_space.keys().into_iter() {
                if !world.my_team().units_in_space.contains_key(&id) {
                    stm.unit_infos_vanished.push(*id);
                }
            }
            let old_array = old_world.get_team_array(player.planet.other());
            let new_array = world.get_team_array(player.planet.other());
            for index in 0..COMMUNICATION_ARRAY_LENGTH {
                if old_array[index] != new_array[index] {
                    stm.other_array_changed.push((index, new_array[index]));
                }
            }
            let map = self.starting_map(player.planet);
            for y in 0..map.height {
                for x in 0..map.width {
                    let karbonite = world.my_planet().karbonite[y][x];
                    if karbonite != old_world.my_planet().karbonite[y][x] {
                        let loc = MapLocation::new(player.planet, x as i32, y as i32);
                        stm.karbonite_changed.push((loc, karbonite));
                    }
                }
            }
        }
        self.cached_world.insert(player, world);

        stm
    }

    fn end_round(&mut self) {
        self.round += 1;

        // Annihilate Earth, if necessary.
        if self.round == APOCALYPSE_ROUND {
            // Destroy all units by clearing Earth's unit data structures.
            let earth = self.get_planet_mut(Planet::Earth);
            earth.units.clear();
            earth.unit_infos.clear();
            earth.units_by_loc.clear();
        }

        // Update unit cooldowns.
        for unit in &mut self.get_planet_mut(Planet::Earth).units.values_mut() {
            unit.end_round();
        }
        for unit in &mut self.get_planet_mut(Planet::Mars).units.values_mut() {
            unit.end_round();
        }

        // Discard the oldest version of each team array.
        self.get_team_mut(Team::Red).team_arrays.end_round();
        self.get_team_mut(Team::Blue).team_arrays.end_round();

        // Process ranger snipes.
        self.process_rangers(Planet::Earth);
        self.process_rangers(Planet::Mars);

        // Add produced factory robots to the garrison.
        self.process_factories();

        // Land rockets.
        self.process_rockets(Team::Red);
        self.process_rockets(Team::Blue);

        // Process any potential asteroid impacts.
        self.process_asteroids();

        // Update the current research and process any completed upgrades.
        self.process_research(Team::Red);
        self.process_research(Team::Blue);
    }

    /// Applies a single delta to this GameWorld.
    pub(crate) fn apply(&mut self, delta: &Delta) -> Result<(), Error> {
        match *delta {
            Delta::Attack {robot_id, target_unit_id} => self.attack(robot_id, target_unit_id),
            Delta::BeginSnipe {ranger_id, location} => self.begin_snipe(ranger_id, location),
            Delta::Blueprint {worker_id, structure_type, direction} => self.blueprint(worker_id, structure_type, direction),
            Delta::Blink {mage_id, location} => self.blink(mage_id, location),
            Delta::Build {worker_id, blueprint_id} => self.build(worker_id, blueprint_id),
            Delta::Disintegrate {unit_id} => self.disintegrate_unit(unit_id),
            Delta::Harvest {worker_id, direction} => self.harvest(worker_id, direction),
            Delta::Heal {healer_id, target_robot_id} => self.heal(healer_id, target_robot_id),
            Delta::Javelin {knight_id, target_unit_id} => self.javelin(knight_id, target_unit_id),
            Delta::LaunchRocket {rocket_id, location} => self.launch_rocket(rocket_id, location),
            Delta::Load {structure_id, robot_id} => self.load(structure_id, robot_id),
            Delta::Move {robot_id, direction} => self.move_robot(robot_id, direction),
            Delta::Overcharge {healer_id, target_robot_id} => self.overcharge(healer_id, target_robot_id),
            Delta::ProduceRobot {factory_id, robot_type} => self.produce_robot(factory_id, robot_type),
            Delta::QueueResearch {branch} => { self.queue_research(branch); Ok(()) },
            Delta::Repair {worker_id, structure_id} => self.repair(worker_id, structure_id),
            Delta::Replicate {worker_id, direction} => self.replicate(worker_id, direction),
            Delta::ResetResearchQueue => { self.reset_research(); Ok(()) },
            Delta::Unload {structure_id, direction} => self.unload(structure_id, direction),
            Delta::WriteTeamArray {index, value} => self.write_team_array(index, value),
            Delta::Nothing => Ok(()),
        }
    }

    /// Applies a turn message to this GameWorld, and ends the current turn. Returns
    /// the next player to move, and whether the current round was also ended.
    pub(crate) fn apply_turn(&mut self, turn: &TurnMessage) -> StartTurnMessage {
        for delta in turn.changes.iter() {
            self.apply(delta).unwrap();
        }
        self.end_turn()
    }

    /// Determines if the game has ended, returning the winning team if so.
    pub(crate) fn is_game_over(&self) -> Option<Team> {
        // Calculate the value of all units.
        let mut red_units_value = 0;
        let mut blue_units_value = 0;
        for unit in self.get_planet(Planet::Earth).units.values() {
            match unit.team() {
                Team::Red => { red_units_value += unit.unit_type().value(); },
                Team::Blue => { blue_units_value += unit.unit_type().value(); },
            }
        }
        for unit in self.get_planet(Planet::Mars).units.values() {
            match unit.team() {
                Team::Red => { red_units_value += unit.unit_type().value(); },
                Team::Blue => { blue_units_value += unit.unit_type().value(); },
            }
        }
        for unit in self.get_team(Team::Red).units_in_space.values() {
            red_units_value += unit.unit_type().value();
        }
        for unit in self.get_team(Team::Blue).units_in_space.values() {
            blue_units_value += unit.unit_type().value();
        }

        // The game should not end if both teams still have units, and we are
        // not at the round limit.
        if self.round() <= MAX_GAME_LEN && red_units_value > 0 && blue_units_value > 0 {
            return None;
        }

        // Tiebreakers proceed in the following order:
        // 1. Highest combined value of all living units
        match red_units_value.cmp(&blue_units_value) {
            Ordering::Less => { return Some(Team::Blue); },
            Ordering::Equal => {},
            Ordering::Greater => { return Some(Team::Red); },
        }

        // 2. Most Karbonite
        match self.get_team(Team::Red).karbonite.cmp(&self.get_team(Team::Blue).karbonite) {
            Ordering::Less => { return Some(Team::Blue); },
            Ordering::Equal => {},
            Ordering::Greater => { return Some(Team::Red); },
        }

        // 3. "RNG"
        // TODO: create an unpredictably seeded RNG for this
        match 6147 % 2 {
            0 => Some(Team::Blue),
            1 => Some(Team::Red),
            _ => unreachable!(),
        }
    }

    /// Get the additional changes that have been generated for the viewer
    /// since this function was last called.
    pub(crate) fn flush_viewer_changes(&mut self) -> Vec<ViewerDelta> {
        let changes = self.viewer_changes.clone();
        self.viewer_changes = Vec::new();
        changes
    }

    /// Get the list of units, with some info truncated, to send to the viewer.
    pub(crate) fn get_viewer_units(&self) -> Vec<ViewerUnitInfo> {
        let mut units = Vec::new();
        for unit in self.get_planet(Planet::Earth).units.values() {
            units.push(ViewerUnitInfo {
                id: unit.id(),
                unit_type: unit.unit_type(),
                health: unit.health(),
                location: unit.location(),
            });
        }
        for unit in self.get_planet(Planet::Mars).units.values() {
            units.push(ViewerUnitInfo {
                id: unit.id(),
                unit_type: unit.unit_type(),
                health: unit.health(),
                location: unit.location(),
            });
        }
        for unit in self.get_team(Team::Red).units_in_space.values() {
            units.push(ViewerUnitInfo {
                id: unit.id(),
                unit_type: unit.unit_type(),
                health: unit.health(),
                location: unit.location(),
            });
        }
        for unit in self.get_team(Team::Blue).units_in_space.values() {
            units.push(ViewerUnitInfo {
                id: unit.id(),
                unit_type: unit.unit_type(),
                health: unit.health(),
                location: unit.location(),
            });
        }
        units
    }

    // ************************************************************************
    // ****************************** PLAYER API ******************************
    // ************************************************************************

    /// Applies a start turn message to reflect the changes in the GameWorld
    /// since the player's last turn, but with the player's limited visibility.
    ///
    /// Aside from applying the changes in the message, this function must
    /// also increment the round and reindex units by location.
    pub(crate) fn start_turn(&mut self, turn: StartTurnMessage) {
        self.round = turn.round;
        self.my_planet_mut().visible_locs = turn.visible_locs;
        for unit in turn.units_changed {
            self.my_planet_mut().units.insert(unit.id(), unit);
        }
        for unit_id in turn.units_vanished {
            self.my_planet_mut().units.remove(&unit_id);
        }
        for unit in turn.unit_infos_changed {
            self.my_planet_mut().unit_infos.insert(unit.id, unit);
        }
        for unit_id in turn.unit_infos_vanished {
            self.my_planet_mut().unit_infos.remove(&unit_id);
        }
        for (location, karbonite) in turn.karbonite_changed {
            let x = location.x as usize;
            let y = location.y as usize;
            self.my_planet_mut().karbonite[y][x] = karbonite;
        }
        for unit in turn.units_in_space_changed {
            self.my_team_mut().units_in_space.insert(unit.id(), unit);
        }
        for unit_id in turn.units_in_space_vanished {
            self.my_team_mut().units_in_space.remove(&unit_id);
        }
        let planet = self.planet().other();
        for (index, value) in turn.other_array_changed {
            self.my_team_mut().team_arrays.write(planet, index, value).unwrap();
        }
        self.id_generator = turn.id_generator;
        self.my_team_mut().rocket_landings = turn.rocket_landings;
        self.my_team_mut().research = turn.research;
        self.my_team_mut().karbonite = turn.karbonite;

        let mut units_by_loc = FnvHashMap::default();
        for (id, unit) in self.my_planet().unit_infos.iter() {
            if let OnMap(loc) = unit.location {
                units_by_loc.insert(loc, *id);
            }
        }
        self.my_planet_mut().units_by_loc = units_by_loc;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _print_visible_locs(locs: &Vec<Vec<bool>>) {
        for bool_row in locs {
            let mut int_row: Vec<u8> = vec![];
            for entry in bool_row {
                int_row.push(*entry as u8);
            }
            println!("{:?}", int_row);
        }
    }

    #[test]
    fn test_all_locations_within() {
        let world = GameWorld::test_world();
        let loc = MapLocation::new(Planet::Earth, 2, 4);
        let locs = world.all_locations_within(loc, 16);
        assert_eq!(locs.len(), 43, "43 locations within 16 distance squared");
        for new_loc in locs {
            assert_lte!(loc.distance_squared_to(new_loc), 16);
        }
        assert_eq!(world.all_locations_within(loc, 0), vec![loc]);
    }

    #[test]
    fn test_end_turn_trivial() {
        let mut world = GameWorld::new(GameMap::test_map());
        let old_worlds = [
            world.cached_world(Player::new(Team::Blue, Planet::Earth)).clone(),
            world.cached_world(Player::new(Team::Red, Planet::Mars)).clone(),
            world.cached_world(Player::new(Team::Blue, Planet::Mars)).clone(),
            world.cached_world(Player::new(Team::Red, Planet::Earth)).clone(),
        ];
        let new_rounds = [1, 1, 1, 2];

        // There should be no changes in each of the first four turns between
        // the initial filtered map and the next turn's filtered map.
        for i in 0..4 {
            let stm = world.end_turn();
            assert_eq!(stm.round, new_rounds[i]);
            assert_eq!(stm.visible_locs, old_worlds[i].my_planet().visible_locs);
            assert_eq!(stm.units_changed.len(), 0);
            assert_eq!(stm.units_vanished.len(), 0);
            assert_eq!(stm.unit_infos_changed.len(), 0);
            assert_eq!(stm.unit_infos_vanished.len(), 0);
            assert_eq!(stm.karbonite_changed.len(), 0);
            assert_eq!(stm.units_in_space_changed.len(), 0);
            assert_eq!(stm.units_in_space_vanished.len(), 0);
            assert_eq!(stm.other_array_changed.len(), 0);
            assert_eq!(stm.rocket_landings, old_worlds[i].my_team().rocket_landings);
            assert_eq!(stm.research, old_worlds[i].my_team().research);
            assert_eq!(stm.karbonite, old_worlds[i].my_team().karbonite);
        }
    }

    #[test]
    fn test_filter_visibility() {
        let initial_units_earth = vec![
            Unit::new(1, Team::Red, UnitType::Worker, 0, OnMap(MapLocation::new(Planet::Earth, 0, 0))).unwrap(),
            Unit::new(2, Team::Red, UnitType::Mage, 0, OnMap(MapLocation::new(Planet::Earth, 10, 11))).unwrap(),
            Unit::new(3, Team::Red, UnitType::Rocket, 0, OnMap(MapLocation::new(Planet::Earth, 10, 10))).unwrap(),
            Unit::new(4, Team::Blue, UnitType::Mage, 0, OnMap(MapLocation::new(Planet::Earth, 11, 10))).unwrap(),
            Unit::new(5, Team::Blue, UnitType::Worker, 0, OnMap(MapLocation::new(Planet::Earth, 29, 29))).unwrap(),
        ];

        let mut map = GameMap::test_map();
        map.earth_map = PlanetMap {
            planet: Planet::Earth,
            height: 30,
            width: 30,
            initial_units: initial_units_earth,
            is_passable_terrain: vec![vec![true; 30]; 30],
            initial_karbonite: vec![vec![0; 30]; 30],
        };
        let mut world = GameWorld::new(map);
        let red_world = world.cached_world(Player::new(Team::Red, Planet::Earth)).clone();
        let mut blue_world = world.cached_world(Player::new(Team::Blue, Planet::Earth)).clone();

        // The Devs engine can see all the units.
        assert!(world.unit_controller(1).is_ok());
        assert!(world.unit_controller(2).is_ok());
        assert!(world.unit_controller(3).is_ok());

        // The Blue units are also visible, but the team is not allowed.
        assert_err!(world.unit_controller(4), GameError::TeamNotAllowed);
        assert_err!(world.unit_controller(5), GameError::TeamNotAllowed);

        // The Red Earth engine cannot see 5, which is not in range.
        assert!(red_world.unit_controller(1).is_ok());
        assert!(red_world.unit_controller(2).is_ok());
        assert!(red_world.unit_controller(3).is_ok());
        assert_err!(red_world.unit_controller(4), GameError::TeamNotAllowed);
        assert_err!(red_world.unit_controller(5), GameError::NoSuchUnit);

        // The Blue Earth engine cannot see 1, which is not in range.
        blue_world.start_turn(world.end_turn());
        assert_err!(blue_world.unit_controller(1), GameError::NoSuchUnit);
        assert_err!(blue_world.unit_controller(2), GameError::TeamNotAllowed);
        assert_err!(blue_world.unit_controller(3), GameError::TeamNotAllowed);
        assert!(blue_world.unit_controller(4).is_ok());
        assert!(blue_world.unit_controller(5).is_ok());
    }

    #[test]
    fn test_sensing_with_filter() {
        // Create a world with some units on Earth, on Mars, and in space.
        let initial_units_earth = vec![
            Unit::new(1, Team::Red, UnitType::Worker, 0, OnMap(MapLocation::new(Planet::Earth, 9, 10))).unwrap(),
            Unit::new(2, Team::Red, UnitType::Mage, 0, OnMap(MapLocation::new(Planet::Earth, 10, 11))).unwrap(),
            Unit::new(3, Team::Red, UnitType::Rocket, 0, OnMap(MapLocation::new(Planet::Earth, 10, 10))).unwrap(),
            Unit::new(4, Team::Blue, UnitType::Mage, 0, OnMap(MapLocation::new(Planet::Earth, 11, 10))).unwrap(),
            Unit::new(5, Team::Blue, UnitType::Worker, 0, OnMap(MapLocation::new(Planet::Earth, 29, 29))).unwrap(),
        ];
        let mut map = GameMap::test_map();
        map.earth_map = PlanetMap {
            planet: Planet::Earth,
            height: 30,
            width: 30,
            initial_units: initial_units_earth,
            is_passable_terrain: vec![vec![true; 30]; 30],
            initial_karbonite: vec![vec![0; 30]; 30],
        };
        let mut world = GameWorld::new(map);
        world.get_unit_mut(3).unwrap().be_built(1000);

        // Red can see 4 units initially on Earth.
        let mut red_world = world.filter(world.player_to_move);
        assert_eq!(red_world.units().len(), 4);

        // After a unit is loaded, it's no longer indexed by location.
        assert!(red_world.load(3, 1).is_ok());
        assert_eq!(red_world.units_by_loc().values().len(), 3);

        // After the rocket launches, it and the garrisoned unit enter space.
        assert!(red_world.launch_rocket(3, MapLocation::new(Planet::Mars, 10, 10)).is_ok());
        assert_eq!(red_world.units().len(), 2);
        assert_eq!(red_world.units_in_space().len(), 2);
        assert_eq!(red_world.units_by_id().values().len(), 2);
        assert_eq!(red_world.units_by_loc().values().len(), 2);

        // Those 4 units are the same units that Red can sense.
        assert!(red_world.can_sense_unit(1));
        assert!(red_world.can_sense_unit(2));
        assert!(red_world.can_sense_unit(3));
        assert!(red_world.can_sense_unit(4));
        assert!(!red_world.can_sense_unit(5));

        // Red can see locations within Unit 2's sensing range.
        let loc_off_map_1 = MapLocation::new(Planet::Mars, 10, 10);
        let loc_off_map_2 = MapLocation::new(Planet::Earth, 2, -2);
        let loc_off_map_3 = MapLocation::new(Planet::Earth, 10, 30);
        let loc_out_of_red_range = MapLocation::new(Planet::Earth, 15, 15);
        let loc_in_red_range = MapLocation::new(Planet::Earth, 14, 14);
        assert!(!red_world.can_sense_location(loc_off_map_1));
        assert!(!red_world.can_sense_location(loc_off_map_2));
        assert!(!red_world.can_sense_location(loc_off_map_3));
        assert!(!red_world.can_sense_location(loc_out_of_red_range));
        assert!(red_world.can_sense_location(loc_in_red_range));

        // Nearby sensing functions should work as expected.
        let red_mage_loc = MapLocation::new(Planet::Earth, 10, 10);
        assert_eq!(red_world.sense_nearby_units(red_mage_loc, 10).len(), 2);
        assert_eq!(red_world.sense_nearby_units_by_team(red_mage_loc, 10, Team::Red).len(), 1);
        assert_eq!(red_world.sense_nearby_units_by_team(red_mage_loc, 10, Team::Blue).len(), 1);
        assert_eq!(red_world.sense_nearby_units_by_type(red_mage_loc, 10, UnitType::Mage).len(), 2);

        // Red cannot see the Blue worker, but it can see the Blue mage.
        assert_err!(red_world.sense_unit_at_location(
            MapLocation::new(Planet::Earth, 29, 29)), GameError::LocationNotVisible);
        assert!(red_world.sense_unit_at_location(
            MapLocation::new(Planet::Earth, 11, 10)).unwrap().is_some());
    }

    #[test]
    fn test_unit_disintegrate() {
        let mut world = GameWorld::test_world();
        let loc_a = MapLocation::new(Planet::Earth, 0, 1);
        let loc_b = MapLocation::new(Planet::Earth, 0, 2);
        let id_a = world.create_unit(Team::Red, loc_a, UnitType::Knight).unwrap();
        let id_b = world.create_unit(Team::Blue, loc_b, UnitType::Knight).unwrap();

        // Red can disintegrate a red unit.
        assert!(world.disintegrate_unit(id_a).is_ok());

        // Red cannot disintegrate a blue unit.
        assert_err!(world.disintegrate_unit(id_b), GameError::TeamNotAllowed);

        // But the Dev engine can "destroy" a blue unit if necessary.
        world.destroy_unit(id_b);

        // Either way, no one can disintegrate a unit that does not exist.
        assert_err!(world.disintegrate_unit(id_b), GameError::NoSuchUnit);
    }

    #[test]
    fn test_unit_destroy_with_filter() {
        let mut world = GameWorld::test_world();
        let mut blue_world = world.cached_world(Player::new(Team::Blue, Planet::Earth)).clone();
        let loc_a = MapLocation::new(Planet::Earth, 0, 1);
        let loc_b = MapLocation::new(Planet::Earth, 0, 2);
        let loc_c = MapLocation::new(Planet::Earth, 0, 3);
        let id_a = world.create_unit(Team::Red, loc_a, UnitType::Rocket).unwrap();
        world.get_unit_mut(id_a).unwrap().be_built(1000);
        let id_b = world.create_unit(Team::Red, loc_b, UnitType::Knight).unwrap();
        world.create_unit(Team::Blue, loc_c, UnitType::Knight).unwrap();

        // Load the rocket with a unit.
        assert!(world.load(id_a, id_b).is_ok());

        // Filter the world on Blue's turn.
        blue_world.start_turn(world.end_turn());

        // Destroy the loaded rocket in the Dev engine.
        assert_eq!(world.my_planet().units.len(), 3);
        assert_eq!(world.my_planet().unit_infos.len(), 3);
        assert_eq!(world.my_planet().units_by_loc.len(), 2);
        world.destroy_unit(id_a);
        assert_eq!(world.my_planet().units.len(), 1);
        assert_eq!(world.my_planet().unit_infos.len(), 1);
        assert_eq!(world.my_planet().units_by_loc.len(), 1);

        // Destroy the loaded rocket in the Blue engine.
        assert_eq!(blue_world.my_planet().units.len(), 1);
        assert_eq!(blue_world.my_planet().unit_infos.len(), 2);
        assert_eq!(blue_world.my_planet().units_by_loc.len(), 2);
        blue_world.destroy_unit(id_a);
        assert_eq!(blue_world.my_planet().units.len(), 1);
        assert_eq!(blue_world.my_planet().unit_infos.len(), 1);
        assert_eq!(blue_world.my_planet().units_by_loc.len(), 1);
    }

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
        let unit_a = world.my_unit(id_a).unwrap();
        let unit_b = world.my_unit(id_b).unwrap();
        assert_eq!(unit_a.location(), OnMap(loc_a));
        assert_eq!(unit_b.location(), OnMap(loc_b));
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
        assert![world.is_move_ready(a)];
        assert![!world.can_move(a, Direction::East)];
        assert![world.can_move(a, Direction::Northeast)];
        world.move_robot(a, Direction::Northeast).unwrap();

        // A is now one square north of B. B cannot move north to
        // A's new location, but can move west to A's old location.
        assert![world.is_move_ready(b)];
        assert![!world.can_move(b, Direction::North)];
        assert![world.can_move(b, Direction::West)];
        world.move_robot(b, Direction::West).unwrap();

        // A robot cannot move again until its cooldowns are reset.
        assert![!world.is_move_ready(a)];
        assert![world.can_move(a, Direction::South)];
        assert![world.move_robot(a, Direction::South).is_err()];
        world.end_round();

        // Finally, let's test that A cannot move back to its old square.
        assert![world.is_move_ready(a)];
        assert![!world.can_move(a, Direction::Southwest)];
        assert![world.can_move(a, Direction::South)];
        world.move_robot(a, Direction::South).unwrap();
    }

    #[test]
    fn test_knight_javelin() {
        // Create the game world.
        let mut world = GameWorld::test_world();

        // Unlock knight's javelin ability through research.
        let unlock_level = 3;
        let rounds = 200;

        for _ in 0..unlock_level {
            let my_research = world.my_research_mut();
            assert!(my_research.add_to_queue(&Branch::Knight));
            for _ in 0..rounds {
                my_research.end_round();
            }
        }

        // Create knight and target robots
        let loc_a = MapLocation::new(Planet::Earth, 0, 0);
        let loc_b = MapLocation::new(Planet::Earth, 0, 1);
        let loc_c = MapLocation::new(Planet::Earth, 0, 20);
        let knight = world.create_unit(Team::Red, loc_a, UnitType::Knight).unwrap();
        let robot_a = world.create_unit(Team::Red, loc_b, UnitType::Knight).unwrap();
        let robot_b = world.create_unit(Team::Red, loc_c, UnitType::Knight).unwrap();
    
        // Knight Javelin is ready
        assert!(world.is_javelin_ready(knight));

        // Knight should not be able to javelin target outside of range
        assert!(!world.can_javelin(knight, robot_b));

        // Knight should be able to javelin target within range
        assert!(world.can_javelin(knight, robot_a));

        // Javelin target. 
        let robot_max_health = 250;
        let robot_damaged_health = 165; 
        assert_eq!(world.get_unit(robot_a).unwrap().health(), robot_max_health);
        assert!(world.javelin(knight, robot_a).is_ok());
        assert_eq!(world.get_unit(robot_a).unwrap().health(), robot_damaged_health);
        assert!(!world.is_javelin_ready(knight));
    }

    #[test]
    fn test_mage_blink() {
        // Create the game world.
        let mut world = GameWorld::test_world();

        // Unlock mage's blink ability through research.
        let unlock_level = 4;
        let rounds = 200;

        for _ in 0..unlock_level {
            let my_research = world.my_research_mut();
            assert!(my_research.add_to_queue(&Branch::Mage));
            for _ in 0..rounds {
                my_research.end_round();
            }
        }

        // Create mage.
        let loc_a = MapLocation::new(Planet::Earth, 0, 0);
        let loc_b = MapLocation::new(Planet::Earth, 0, 1);
        let loc_c = MapLocation::new(Planet::Earth, 0, 20);
        let mage = world.create_unit(Team::Red, loc_a, UnitType::Mage).unwrap();
        
        // Mage blink is ready.
        assert!(world.is_blink_ready(mage));

        // Mage should not be able to blink to target location outside of range.
        assert!(!world.can_blink(mage, loc_c));

        // Mage should be able to blink to target location within range.
        assert!(world.can_blink(mage, loc_b));

        // Blink moves mage to new location.
        assert_eq!(world.get_unit(mage).unwrap().location(), OnMap(loc_a));
        assert!(world.blink(mage, loc_b).is_ok());
        assert_eq!(world.get_unit(mage).unwrap().location(), OnMap(loc_b));
        assert!(!world.is_blink_ready(mage));
    }

    #[test]
    fn test_ranger_snipe() {
        // Create the game world.
        let mut world = GameWorld::test_world();

        // Unlock mage's blink ability through research.
        let unlock_level = 3;
        let rounds = 200;

        for _ in 0..unlock_level {
            let my_research = world.my_research_mut();
            assert!(my_research.add_to_queue(&Branch::Ranger));
            for _ in 0..rounds {
                my_research.end_round();
            }
        }

        // Create ranger and target robot.
        let loc_a = MapLocation::new(Planet::Earth, 0, 0);
        let loc_b = MapLocation::new(Planet::Earth, 0, 1);
        let loc_c = MapLocation::new(Planet::Mars, 0, 20);
        let ranger = world.create_unit(Team::Red, loc_a, UnitType::Ranger).unwrap();
        let robot = world.create_unit(Team::Red, loc_b,UnitType::Knight).unwrap();
        // Ranger should not be able to snipe target location on a different planet.
        assert!(world.begin_snipe(ranger, loc_c).is_err());

        // Ranger begins to snipe a location.
        assert!(world.begin_snipe(ranger, loc_b).is_ok());

        // Enough rounds pass where Ranger's snipe is processed
        let rounds = 200;
        for _ in 0..rounds {
            world.end_round();
        }
        
        // Robot at sniped location should take damage
        let robot_damaged_health = 185;
        assert_eq!(world.get_unit(robot).unwrap().health(), robot_damaged_health);
    }

    #[test]
    fn test_healer_overcharge() {
        // Create the game world.
        let mut world = GameWorld::test_world();

        // Unlock healer's overcharge ability through research.
        let unlock_level = 3;
        let rounds = 200;

        for _ in 0..unlock_level {
            let my_research = world.my_research_mut();
            assert!(my_research.add_to_queue(&Branch::Healer));
            for _ in 0..rounds {
                my_research.end_round();
            }
        }

        // Unlock robot's ability through research.
        let unlock_level = 3;
        let rounds = 200;

        for _ in 0..unlock_level {
            let my_research = world.my_research_mut();
            assert!(my_research.add_to_queue(&Branch::Knight));
            for _ in 0..rounds {
                my_research.end_round();
            }
        }

        // Create healer and target robots.
        let loc_a = MapLocation::new(Planet::Earth, 0, 0);
        let loc_b = MapLocation::new(Planet::Earth, 0, 1);
        let loc_c = MapLocation::new(Planet::Earth, 0, 20);
        let healer = world.create_unit(Team::Red, loc_a, UnitType::Healer).unwrap();
        let robot_a = world.create_unit(Team::Red, loc_b, UnitType::Knight).unwrap();
        let robot_b = world.create_unit(Team::Red, loc_c, UnitType::Knight).unwrap();

        // Healer overcharge is ready.
        assert!(world.is_overcharge_ready(healer));

        // Healer should not be able to overcharge target robot outside of range.
        assert!(!world.can_overcharge(healer, robot_b));
        
        // Healer should be able to overcharge target robot within range.
        assert!(world.can_overcharge(healer, robot_a));

        // Robot uses ability.
        let loc_d = MapLocation::new(Planet::Earth, 0, 2);
        assert!(world.move_to(robot_b, loc_d).is_ok());
        assert!(world.javelin(robot_a, robot_b).is_ok());
        assert!(!world.get_unit(robot_a).unwrap().ok_if_ability_ready().is_ok());


        // Healer uses overcharge to reset robot's ablity cooldown
        assert!(world.overcharge(healer, robot_a).is_ok());
        assert!(world.get_unit(robot_a).unwrap().ok_if_ability_ready().is_ok());
    }

    #[test]
    fn test_rocket_success() {
        // Create the game world.
        let mut world = GameWorld::test_world();
        let earth_loc = MapLocation::new(Planet::Earth, 5, 5);
        let mars_loc = MapLocation::new(Planet::Mars, 5, 5);
        let rocket = world.create_unit(Team::Red, earth_loc, UnitType::Rocket).unwrap();
        world.get_unit_mut(rocket).unwrap().be_built(1000);

        // Create units around the target location.
        let mut earth_bystanders: Vec<UnitID> = vec![];
        let mut mars_bystanders: Vec<UnitID> = vec![];
        for direction in Direction::all() {
            earth_bystanders.push(world.create_unit(Team::Red, earth_loc.add(direction), UnitType::Knight).unwrap());
            mars_bystanders.push(world.create_unit(Team::Red, mars_loc.add(direction), UnitType::Knight).unwrap());
        }

        // Launch the rocket.
        assert![world.can_launch_rocket(rocket, mars_loc)];
        world.launch_rocket(rocket, mars_loc).unwrap();
        assert_eq![world.my_unit(rocket).unwrap().location(), InSpace];
        let damaged_knight_health = 205;
        for id in earth_bystanders.iter() {
            assert_eq![world.my_unit(*id).unwrap().health(), damaged_knight_health];
        }

        // Go forward two turns so that we're on Mars.
        world.end_turn();
        world.end_turn();

        // Force land the rocket.
        world.land_rocket(rocket, mars_loc);
        assert_eq![world.my_unit(rocket).unwrap().location(), OnMap(mars_loc)];
            for id in mars_bystanders.iter() {
            assert_eq![world.my_unit(*id).unwrap().health(), damaged_knight_health];
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
        world.planet_maps.get_mut(&Planet::Mars).unwrap().is_passable_terrain[0][0] = false;
        let mars_loc_knight = MapLocation::new(Planet::Mars, 0, 1);
        let mars_loc_factory = MapLocation::new(Planet::Mars, 0, 2);
        let rocket_a = world.create_unit(Team::Red, earth_loc_a, UnitType::Rocket).unwrap();
        world.get_unit_mut(rocket_a).unwrap().be_built(1000);
        let rocket_b = world.create_unit(Team::Red, earth_loc_b, UnitType::Rocket).unwrap();
        world.get_unit_mut(rocket_b).unwrap().be_built(1000);
        let knight = world.create_unit(Team::Blue, mars_loc_knight, UnitType::Knight).unwrap();
        let factory = world.create_unit(Team::Blue, mars_loc_factory, UnitType::Factory).unwrap();

        // Failed launches.
        assert![!world.can_launch_rocket(rocket_a, earth_loc_b)];
        assert_err![world.launch_rocket(rocket_a, earth_loc_b), GameError::SamePlanet];
        assert![!world.can_launch_rocket(rocket_a, mars_loc_off_map)];
        assert_err![world.launch_rocket(rocket_a, mars_loc_off_map), GameError::LocationOffMap];
        assert![!world.can_launch_rocket(rocket_a, mars_loc_impassable)];
        assert_err![world.launch_rocket(rocket_a, mars_loc_impassable), GameError::LocationNotEmpty];

        // Rocket landing on a robot should destroy the robot.
        assert![world.can_launch_rocket(rocket_a, mars_loc_knight)];
        assert![world.launch_rocket(rocket_a, mars_loc_knight).is_ok()];
        world.end_turn();
        world.end_turn();
        world.land_rocket(rocket_a, mars_loc_knight);
        assert![world.my_unit(rocket_a).is_ok()];
        world.end_turn();
        assert_err![world.my_unit(knight), GameError::NoSuchUnit];

        // Launch the rocket on Earth.
        world.end_turn();
        assert![world.can_launch_rocket(rocket_b, mars_loc_factory)];
        assert![world.launch_rocket(rocket_b, mars_loc_factory).is_ok()];

        // Go forward two turns so that we're on Mars.
        world.end_turn();
        world.end_turn();

        // Rocket landing on a factory should destroy both units.
        world.land_rocket(rocket_b, mars_loc_factory);
        assert_err![world.my_unit(rocket_b), GameError::NoSuchUnit];
        assert_err![world.my_unit(factory), GameError::NoSuchUnit];
    }

    #[test]
    fn test_rocket_load() {
        // Create the game world and the rocket for this test.
        let mut world = GameWorld::test_world();
        let takeoff_loc = MapLocation::new(Planet::Earth, 10, 10);        
        let rocket = world.create_unit(Team::Red, takeoff_loc, UnitType::Rocket).unwrap();
        world.get_unit_mut(rocket).unwrap().be_built(1000);

        // Correct loading.
        let valid_boarder = world.create_unit(Team::Red, takeoff_loc.add(Direction::North), UnitType::Knight).unwrap();
        assert![world.can_load(rocket, valid_boarder)];
        assert![world.load(rocket, valid_boarder).is_ok()];
        assert_eq![world.my_unit(valid_boarder).unwrap().location(), InGarrison(rocket)];

        // Boarding fails when too far from the rocket.
        let invalid_boarder_too_far = world.create_unit(Team::Red, takeoff_loc.add(Direction::North).add(Direction::North), UnitType::Knight).unwrap();
        assert![!world.can_load(rocket, valid_boarder)];
        assert_err![world.load(rocket, invalid_boarder_too_far), GameError::OutOfRange];

        // Boarding fails when the robot has already moved.
        assert![world.move_robot(invalid_boarder_too_far, Direction::South).is_ok()];
        let invalid_boarder_already_moved = invalid_boarder_too_far;
        assert![!world.is_move_ready(invalid_boarder_already_moved)];
        assert![!world.can_load(rocket, invalid_boarder_already_moved)];
        assert_err![world.load(rocket, invalid_boarder_already_moved), GameError::Overheated];

        // Factories and rockets cannot board rockets.
        let invalid_boarder_factory = world.create_unit(Team::Red, takeoff_loc.add(Direction::Southeast), UnitType::Factory).unwrap();
        assert![!world.can_load(rocket, invalid_boarder_factory)];
        assert_err![world.load(rocket, invalid_boarder_factory), GameError::InappropriateUnitType];
        let invalid_boarder_rocket = world.create_unit(Team::Red, takeoff_loc.add(Direction::South), UnitType::Rocket).unwrap();
        assert![!world.can_load(rocket, invalid_boarder_rocket)];
        assert_err![world.load(rocket, invalid_boarder_rocket), GameError::InappropriateUnitType];

        // Rockets can be loaded up to their capacity...
        for _ in 1..8 {
            let valid_extra_boarder = world.create_unit(Team::Red, takeoff_loc.add(Direction::East), UnitType::Knight).unwrap();
            assert![world.can_load(rocket, valid_extra_boarder)];
            assert![world.load(rocket, valid_extra_boarder).is_ok()];
        }

        // ... but not beyond their capacity.
        let invalid_boarder_rocket_full = world.create_unit(Team::Red, takeoff_loc.add(Direction::East), UnitType::Knight).unwrap();
        assert![!world.can_load(rocket, invalid_boarder_rocket_full)];
        assert_err![world.load(rocket, invalid_boarder_rocket_full), GameError::GarrisonFull];

        // A unit should not be able to board another team's rocket.
        let blue_takeoff_loc = MapLocation::new(Planet::Earth, 5, 5);
        let blue_rocket = world.create_unit(Team::Blue, blue_takeoff_loc, UnitType::Rocket).unwrap();
        let invalid_boarder_wrong_team = world.create_unit(Team::Red, blue_takeoff_loc.add(Direction::North), UnitType::Knight).unwrap();
        assert![!world.can_load(blue_rocket, invalid_boarder_wrong_team)];
        assert_err![world.load(blue_rocket, invalid_boarder_wrong_team), GameError::TeamNotAllowed];
    }

    #[test]
    fn test_rocket_unload() {
        // Create the game world and the rocket for this test.
        let mut world = GameWorld::test_world();
        let takeoff_loc = MapLocation::new(Planet::Earth, 10, 10);        
        let rocket = world.create_unit(Team::Red, takeoff_loc, UnitType::Rocket).unwrap();
        world.get_unit_mut(rocket).unwrap().be_built(1000);
        
        // Load the rocket with robots.
        for _ in 0..2 {
            let robot = world.create_unit(Team::Red, takeoff_loc.add(Direction::North), UnitType::Knight).unwrap();
            assert![world.can_load(rocket, robot)];
            assert![world.load(rocket, robot).is_ok()];
        }

        // Fly the rocket to Mars.
        let landing_loc = MapLocation::new(Planet::Mars, 0, 0);
        assert![world.launch_rocket(rocket, landing_loc).is_ok()];

        // Go forward two turns so that we're on Mars.
        world.end_turn();
        world.end_turn();
        world.land_rocket(rocket, landing_loc);

        // Cannot unload in the same round. But can after one turn.
        assert![!world.can_unload(rocket, Direction::North)];
        assert_err![world.unload(rocket, Direction::North), GameError::Overheated];
        world.end_round();

        // Correct unloading.
        assert![world.can_unload(rocket, Direction::North)];
        assert![world.unload(rocket, Direction::North).is_ok()];

        // Cannot unload into an occupied square.
        assert![!world.can_unload(rocket, Direction::North)];
        assert![world.unload(rocket, Direction::North).is_err()];

        // Cannot unload into an impassable square.
        world.planet_maps.get_mut(&Planet::Mars).unwrap().is_passable_terrain[0][1] = false;
        assert![!world.can_unload(rocket, Direction::East)];
        assert_err![world.unload(rocket, Direction::East), GameError::LocationNotEmpty];

        // Error unloading off the map.
        assert![!world.can_unload(rocket, Direction::South)];
        assert_err![world.unload(rocket, Direction::South), GameError::LocationOffMap];

        // Error unloading not a rocket.
        let robot_loc = MapLocation::new(Planet::Mars, 10, 10);
        let robot = world.create_unit(Team::Red, robot_loc, UnitType::Mage).unwrap();
        assert![!world.can_unload(robot, Direction::East)];
        assert_err![world.unload(robot, Direction::East), GameError::InappropriateUnitType];

        // Correct unloading, again.
        world.planet_maps.get_mut(&Planet::Mars).unwrap().is_passable_terrain[0][1] = true;
        assert![world.can_unload(rocket, Direction::East)];
        assert![world.unload(rocket, Direction::East).is_ok()];

        // Cannot unload an empty rocket.
        assert![!world.can_unload(rocket, Direction::East)];
        assert_err![world.unload(rocket, Direction::East), GameError::GarrisonEmpty];
    }

    #[test]
    fn test_worker_harvest() {
        // Create the game world, which by default has 10 karbonite everywhere.
        let mut world = GameWorld::test_world();

        // Select a deposit, and test that it can be mined out as expected.
        let deposit = MapLocation::new(Planet::Earth, 0, 0);
        let expected_karbonite = [10, 7, 4, 1, 0];
        let expected_team_karbonite = [100, 103, 106, 109, 110];
        for i in 0..4 {
            let worker = world.create_unit(Team::Red, deposit.add(Direction::North), 
                                                UnitType::Worker).unwrap();
            assert![world.can_harvest(worker, Direction::South)];
            assert_eq![world.karbonite_at(deposit).unwrap(), expected_karbonite[i]];
            assert_eq![world.karbonite(), expected_team_karbonite[i]];
            assert![world.harvest(worker, Direction::South).is_ok()];
            // The robot can no longer harvest, as it has already done so.
            assert![!world.can_harvest(worker, Direction::South)];
            assert_eq![world.karbonite_at(deposit).unwrap(), expected_karbonite[i+1]];
            assert_eq![world.karbonite(), expected_team_karbonite[i+1]];
            world.destroy_unit(worker);
        }

        // The deposit has been mined out, so it cannot be harvested.
        let worker = world.create_unit(Team::Red, deposit.add(Direction::North),
                                            UnitType::Worker).unwrap();
        assert![!world.can_harvest(worker, Direction::South)];

        // Other deposits can still be harvested, including in the robot's own space.
        assert![world.can_harvest(worker, Direction::Center)];

        // Deposits off the edge of the map can obviously not be harvested, but checking
        // this should not error.
        assert![!world.can_harvest(worker, Direction::West)];
    }

    #[test]
    fn test_worker_blueprint_and_build() {
        // Create the game world.
        let mut world = GameWorld::test_world();

        // Select a location to build a factory, and create a worker.
        let factory_loc = MapLocation::new(Planet::Earth, 0, 0);
        let worker_a = world.create_unit(Team::Red, factory_loc.add(Direction::North), UnitType::Worker).unwrap();

        // You cannot blueprint a robot.
        assert![!world.can_blueprint(worker_a, UnitType::Knight, Direction::South)];

        // A factory cannot be blueprinted yet, because there are not enough resources.
        assert![!world.can_blueprint(worker_a, UnitType::Factory, Direction::South)];

        // After adding more resources to the team pool, blueprinting a factory is possible.
        world.get_team_mut(Team::Red).karbonite = 1000;
        assert![world.can_blueprint(worker_a, UnitType::Factory, Direction::South)];

        // However, a factory cannot be blueprinted to the west, as this is off the map.
        assert![!world.can_blueprint(worker_a, UnitType::Factory, Direction::West)];

        assert![world.blueprint(worker_a, UnitType::Factory, Direction::South).is_ok()];
        let factory = world.get_planet_mut(Planet::Earth).units_by_loc[&factory_loc];

        // The factory cannot be built by the same worker, because it has already acted.
        assert![!world.can_build(worker_a, factory)];
        world.destroy_unit(worker_a);

        // It takes 150 build actions, with default research, to complete a factory.
        for i in 0..150 {
            // Create a worker two squares north of the factory blueprint.
            let worker_b = world.create_unit(Team::Red, 
                                             factory_loc.add(Direction::North).add(Direction::North), 
                                             UnitType::Worker).unwrap();

            // The worker is initially too far away to build the factory.
            assert![!world.can_build(worker_b, factory)];
            assert![world.move_robot(worker_b, Direction::South).is_ok()];

            // The worker is now able to build the factory.
            assert![world.can_build(worker_b, factory)];
            assert![world.build(worker_b, factory).is_ok()];
            assert_eq![world.get_unit(factory).unwrap().health(), 255 + 5*i];

            // The worker has already acted, and cannot build again.
            assert![!world.can_build(worker_b, factory)];
            world.destroy_unit(worker_b);
        }
        assert![world.get_unit(factory).unwrap().structure_is_built().unwrap()];

        // Subsequent attempts to build the factory should fail.
        let worker_c = world.create_unit(Team::Red, factory_loc.add(Direction::North), UnitType::Worker).unwrap();
        assert![!world.can_build(worker_c, factory)];
        world.destroy_unit(worker_c);

        // It should not be possible to blueprint a rocket until researching Rocketry.
        let rocket_loc = MapLocation::new(Planet::Earth, 1, 0);
        let worker_d = world.create_unit(Team::Red, rocket_loc.add(Direction::North), UnitType::Worker).unwrap();
        assert![!world.can_blueprint(worker_d, UnitType::Rocket, Direction::South)];

        // Force-research Rocketry.
        assert![world.queue_research(Branch::Rocket)];
        for _ in 0..1000 {
            world.process_research(Team::Red);
        }

        // Rockets can now be built!
        assert![world.can_blueprint(worker_d, UnitType::Rocket, Direction::South)];
        assert![world.blueprint(worker_d, UnitType::Rocket, Direction::South).is_ok()];

        // Blueprinting is never possible on Mars.
        world.end_turn();
        world.end_turn();
        let mars_factory_loc = MapLocation::new(Planet::Mars, 0, 0);
        let worker_e = world.create_unit(Team::Red, mars_factory_loc.add(Direction::North), UnitType::Worker).unwrap();
        assert![!world.can_blueprint(worker_e, UnitType::Factory, Direction::South)];
    }

    #[test]
    fn test_factory_production() {
        let mut world = GameWorld::test_world();
        let loc = MapLocation::new(Planet::Earth, 10, 10);
        let factory = world.create_unit(Team::Red, loc, UnitType::Factory).unwrap();
        world.get_unit_mut(factory).unwrap().be_built(1000);
        let mage_cost = UnitType::Mage.factory_cost().unwrap();

        // The factory can produce a robot only if it's not already busy.
        assert!(world.can_produce_robot(factory, UnitType::Mage));
        assert!(world.produce_robot(factory, UnitType::Mage).is_ok());
        assert!(!world.can_produce_robot(factory, UnitType::Mage));
        assert_err!(world.produce_robot(factory, UnitType::Mage), GameError::FactoryBusy);
        assert_eq!(world.my_team().karbonite, KARBONITE_STARTING - mage_cost);

        // After a few rounds, the mage is added to the world.
        for _ in 0..FACTORY_NUM_ROUNDS {
            world.end_round();
        }
        assert_eq!(world.my_unit(factory).unwrap().structure_garrison().unwrap().len(), 1);
        assert_eq!(world.my_planet().units.len(), 2);
        assert_eq!(world.my_planet().unit_infos.len(), 2);
        assert_eq!(world.my_planet().units_by_loc.len(), 1);

        // Karbonite is a limiting factor for producing robots.
        assert!(world.can_produce_robot(factory, UnitType::Mage));
        world.my_team_mut().karbonite = 0;
        assert!(!world.can_produce_robot(factory, UnitType::Mage));
    }

    #[test]
    fn test_robot_attack_and_heal() {
        let mut world = GameWorld::test_world();
        let ranger = world.create_unit(Team::Red, MapLocation::new(Planet::Earth, 0, 0), UnitType::Ranger).unwrap();
        let worker_in_range = world.create_unit(Team::Red, MapLocation::new(Planet::Earth, 5, 0), UnitType::Worker).unwrap();
        let worker_out_of_range = world.create_unit(Team::Red, MapLocation::new(Planet::Earth, 10, 0), UnitType::Worker).unwrap();

        // The ranger can attack the adjacent worker, but not the non-adjacent worker.
        assert![world.can_attack(ranger, worker_in_range)];
        assert![!world.can_attack(ranger, worker_out_of_range)];
        assert![world.attack(ranger, worker_in_range).is_ok()];

        // The worker should have taken some damage.
        assert_eq![world.get_unit(worker_in_range).unwrap().health(), 30];
        assert_eq![world.get_unit(worker_out_of_range).unwrap().health(), 100];

        // The ranger cannot attack again.
        assert![!world.is_attack_ready(ranger)];
        assert_err![world.attack(ranger, worker_in_range), GameError::Overheated];
        assert![!world.is_attack_ready(ranger)];
        assert![world.can_attack(ranger, worker_in_range)];

        // Create a healer, and use it to heal the worker.
        let healer = world.create_unit(Team::Red, MapLocation::new(Planet::Earth, 5, 1), UnitType::Healer).unwrap();
        assert![world.can_heal(healer, worker_in_range)];
        assert![world.heal(healer, worker_in_range).is_ok()];
        assert_eq![world.get_unit(worker_in_range).unwrap().health(), 40];
    }

    #[test]
    fn test_replicate() {
        let mut world = GameWorld::test_world();
        let worker = world.create_unit(Team::Red, MapLocation::new(Planet::Earth, 0, 0), UnitType::Worker).unwrap();
        let _ = world.create_unit(Team::Blue, MapLocation::new(Planet::Earth, 1, 0), UnitType::Factory).unwrap();

        // The worker cannot replicate to the west, because that space is off the map.
        assert![!world.can_replicate(worker, Direction::West)];
        assert_err![world.replicate(worker, Direction::West), GameError::LocationOffMap];

        // The worker cannot replicate to the east, because that space is obstructed.
        assert![!world.can_replicate(worker, Direction::East)];
        assert_err![world.replicate(worker, Direction::East), GameError::LocationNotEmpty];

        // The worker can replicate to the north.
        assert![world.can_replicate(worker, Direction::North)];
        assert![world.replicate(worker, Direction::North).is_ok()];
        assert_eq![world.karbonite(), 40];

        // The child cannot replicate, because there isn't enough Karbonite.
        let child = world.sense_unit_at_location(MapLocation::new(Planet::Earth, 0, 1)).unwrap().unwrap().id;
        assert![!world.can_replicate(child, Direction::North)];
        assert_err![world.replicate(child, Direction::North), GameError::InsufficientKarbonite];

        // After acquiring more Karbonite, replication is possible.
        world.my_team_mut().karbonite += 1000;
        assert![world.can_replicate(child, Direction::North)];
        assert![world.replicate(child, Direction::North).is_ok()];

        // The child cannot replicate again this round.
        assert![!world.can_replicate(child, Direction::East)];
        assert_err![world.replicate(child, Direction::East), GameError::Overheated];

        // Even after ending the round, the child cannot replicate immediately again.
        world.end_round();
        assert![!world.can_replicate(child, Direction::East)];
        assert_err![world.replicate(child, Direction::East), GameError::Overheated];
    }

    #[test]
    fn test_repair() {
        let mut world = GameWorld::test_world();
        let factory = world.create_unit(Team::Red, MapLocation::new(Planet::Earth, 0, 0), UnitType::Factory).unwrap();
        let worker = world.create_unit(Team::Red, MapLocation::new(Planet::Earth, 1, 0), UnitType::Worker).unwrap();

        // The worker cannot repair the factory, as it is not yet built.
        assert![!world.can_repair(worker, factory)];
        assert_err![world.repair(worker, factory), GameError::StructureNotYetBuilt];

        // After forcibly completing the structure, we damage it.
        world.get_unit_mut(factory).unwrap().be_built(1000);
        assert![world.get_unit(factory).unwrap().structure_is_built().unwrap()];
        world.get_unit_mut(factory).unwrap().take_damage(100);
        assert_eq![world.get_unit(factory).unwrap().health(), 900];

        // The factory can now be repaired.
        assert![world.can_repair(worker, factory)];
        assert![world.repair(worker, factory).is_ok()];
        assert_eq![world.get_unit(factory).unwrap().health(), 910];

        // The worker cannot repair again this turn.
        assert![!world.can_repair(worker, factory)];
        assert_err![world.repair(worker, factory), GameError::Overheated];

        // After force-ending the round, the worker can repair again.
        world.end_round();
        assert![world.can_repair(worker, factory)];

        // If the worker moves away, it cannot repair the factory.
        assert![world.move_robot(worker, Direction::East).is_ok()];
        assert![!world.can_repair(worker, factory)];
        assert_err![world.repair(worker, factory), GameError::OutOfRange];
    }

    #[test]
    fn test_ranger_attack_range() {
        let mut world = GameWorld::test_world();
        let ranger = world.create_unit(Team::Red, MapLocation::new(Planet::Earth, 0, 0), UnitType::Ranger).unwrap();
        let too_close = world.create_unit(Team::Blue, MapLocation::new(Planet::Earth, 1, 0), UnitType::Ranger).unwrap();
        let too_far = world.create_unit(Team::Blue, MapLocation::new(Planet::Earth, 10, 0), UnitType::Ranger).unwrap();
        let just_right = world.create_unit(Team::Blue, MapLocation::new(Planet::Earth, 5, 0), UnitType::Ranger).unwrap();

        assert_err![world.attack(ranger, too_close), GameError::OutOfRange];
        assert_err![world.attack(ranger, too_far), GameError::OutOfRange];
        assert![world.attack(ranger, just_right).is_ok()];
    }
  
    #[test]
    fn test_mage_splash() {
        let mut world = GameWorld::test_world();
        let mage = world.create_unit(Team::Red, MapLocation::new(Planet::Earth, 0, 0), UnitType::Mage).unwrap();
        let mut victims = vec![];
        for x in 1..4 {
            for y in 1..4 {
                victims.push(world.create_unit(Team::Red, MapLocation::new(Planet::Earth, x, y), UnitType::Factory).unwrap());
            }
        }

        // After attacking the middle factory, all factories should be damaged.
        for victim in victims.iter() {
            assert_eq![world.unit_info(*victim).unwrap().health, 250];
        }
        assert![world.attack(mage, victims[4]).is_ok()];
        for victim in victims.iter() {
            assert_eq![world.unit_info(*victim).unwrap().health, 100];
        }
    }

    #[test]
    fn test_apocalypse() {
        let mut world = GameWorld::test_world();
        let _ = world.create_unit(Team::Red, MapLocation::new(Planet::Earth, 0, 0), UnitType::Factory).unwrap();
        let _ = world.create_unit(Team::Red, MapLocation::new(Planet::Mars, 0, 0), UnitType::Factory).unwrap();
        
        // Both units exist until round 750.
        for _ in 1..750 {
            assert_eq![world.get_planet(Planet::Earth).units.len(), 1];
            assert_eq![world.get_planet(Planet::Mars).units.len(), 1];
            world.end_round();
        }

        // At the start of round 750, the Earth unit is destroyed.
        assert_eq![world.get_planet(Planet::Earth).units.len(), 0];
        assert_eq![world.get_planet(Planet::Mars).units.len(), 1];
    }

    #[test]
    fn test_is_game_over() {
        let mut world = GameWorld::test_world();

        // Initially, neither player has units, so the game is over, but it's a tossup who won.
        assert![world.is_game_over().is_some()];

        // If we give both red and blue units, the game is not over.
        world.create_unit(Team::Red, MapLocation::new(Planet::Earth, 0, 0), UnitType::Knight).unwrap();
        world.create_unit(Team::Blue, MapLocation::new(Planet::Earth, 0, 0), UnitType::Knight).unwrap();
        assert![world.is_game_over().is_none()];

        // If we advance 1000 rounds, the game should be over, and it's again a tossup.
        for _ in 0..1000 {
            world.end_round();
        }
        assert![world.is_game_over().is_some()];
        // The apocalypse has now destroyed the preexisting units.

        // Giving red some extra Karbonite means a victory for red.
        world.get_team_mut(Team::Red).karbonite += 10;
        assert![world.is_game_over().is_some()];
        assert_eq![world.is_game_over().unwrap(), Team::Red];

        // Giving blue even more Karbonite lets blue win.
        world.get_team_mut(Team::Blue).karbonite += 20;
        assert![world.is_game_over().is_some()];
        assert_eq![world.is_game_over().unwrap(), Team::Blue];

        // Giving red a unit lets red win.
        world.create_unit(Team::Red, MapLocation::new(Planet::Earth, 0, 0), UnitType::Knight).unwrap();
        assert![world.is_game_over().is_some()];
        assert_eq![world.is_game_over().unwrap(), Team::Red];

        // Giving blue a more expensive unit lets blue win.
        world.create_unit(Team::Blue, MapLocation::new(Planet::Mars, 0, 0), UnitType::Factory).unwrap();
        assert![world.is_game_over().is_some()];
        assert_eq![world.is_game_over().unwrap(), Team::Blue];
    }
}
