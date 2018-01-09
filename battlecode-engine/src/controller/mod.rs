//! The outermost layer of the engine stack. Responsible for exposing
//! the API that the player will use, and for generating messages to
//! send to other parts of the Battlecode infrastructure.

use location::*;
use map::*;
use research::*;
use rockets::*;
use schema::*;
use team_array::*;
use unit::*;
use world::*;
    
use failure::Error;
use fnv::FnvHashMap;
use std::env;
use std::mem;

mod streams;
use self::streams::Streams;

/// Configuration for the game controller.
pub struct Config {
    /// Whether to generate messages to be sent to the viewer.
    pub generate_viewer_messages: bool,

    /// Whether to generate turn messages.
    pub generate_turn_messages: bool,
}

impl Config {
    pub fn player_config() -> Config {
        Config {
            generate_viewer_messages: false,
            generate_turn_messages: true,
        }
    }

    pub fn runner_config() -> Config {
        Config {
            generate_viewer_messages: true,
            generate_turn_messages: true,
        }
    }
}

/// The outermost layer of the engine stack.
pub struct GameController {
    world: GameWorld,
    old_world: GameWorld,
    config: Config,
    turn: TurnMessage,
    stream: Option<Streams>,
    player_key: Option<String>
}

fn check_message<T>(msg: ReceivedMessage<T>, player_key: &str) -> Result<T, Error> {
    let ReceivedMessage {
        logged_in,
        client_id,
        error,
        message
    } = msg;
    if !logged_in {
        bail!("Not logged in?");
    }
    if &client_id[..] != player_key {
        bail!("Wrong client_id: should be '{}', is '{}'", player_key, client_id);
    }
    if let Some(error) = error {
        bail!("Error from manager: '{}'", error);
    }
    if let Some(message) = message {
        Ok(message)
    } else {
        bail!("No message sent?")
    }
}


impl GameController {

    // ************************************************************************
    // ************************************************************************
    // ************************************************************************
    // **************************** PLAYER API ********************************
    // ************************************************************************
    // ************************************************************************
    // ************************************************************************

    /// Connect to a manager using environment variables.
    /// You should call this method if you're running inside the player docker container.
    /// It will connect to the manager and block until it's your turn. (Don't worry, you'll be
    /// paused during the blocking anyway.)
    pub fn new_player_env() -> Result<GameController, Error> {
        let socket_file = env::var("SOCKET_FILE")?;
        let player_key = env::var("PLAYER_KEY")?;

        // send login
        let mut stream = Streams::new(socket_file)?;
        stream.write(&LoginMessage {
            client_id: player_key.clone()
        })?;

        // wait for response with empty string in message field
        let msg = stream.read::<ReceivedMessage<String>>()?;
        let s = check_message(msg, &player_key[..])?;
        if &s[..] != "" {
            bail!("Non-empty login response: {}", s);
        }

        // block, and eventually receive the start game message
        let msg = stream.read::<ReceivedMessage<StartGameMessage>>()?;
        let StartGameMessage { mut world } = check_message(msg, &player_key[..])?;


        // then the start turn message
        let msg = stream.read::<ReceivedMessage<StartTurnMessage>>()?;
        let turn = check_message(msg, &player_key[..])?;
        world.start_turn(&turn);

        println!("Successfully connected to the manager!");

        // now return and let the player do their thing on the first turn :)
        Ok(GameController {
            old_world: world.clone(),
            world,
            config: Config::player_config(),
            turn: TurnMessage { changes: vec![] },
            stream: Some(stream),
            player_key: Some(player_key)
        })
    }

    /// Submit your current turn and wait for your next turn. Blocks. Don't worry, you'll be
    /// paused during the blocking anyway.
    pub fn next_turn(&mut self) -> Result<(), Error> {
        if let None = self.stream {
            bail!("Controller is not in env mode, has no stream, can't call next_turn()");
        }
        if let None = self.player_key {
            bail!("No player key??");
        }

        // extract our previous turn, replacing it with an empty one
        let mut turn_message = TurnMessage { changes: vec![] };
        mem::swap(&mut self.turn, &mut turn_message);

        // send off our previous turn
        self.stream.as_mut().unwrap().write(&SentMessage {
            client_id: self.player_key.as_ref().unwrap().clone(),
            turn_message
        })?;

        // block and receive the state for our next turn
        let msg = self.stream.as_mut().unwrap().read::<ReceivedMessage<StartTurnMessage>>()?;
        let start_turn = check_message(msg, &self.player_key.as_ref().unwrap()[..])?;

        // setup the world state
        self.old_world.start_turn(&start_turn);
        self.world = self.old_world.clone();


        // yield control to the player
        Ok(())
    }

    /// Initializes the game world and creates a new controller
    /// for a player to interact with it.
    /// Mainly for testing purposes.
    pub fn new_player(game: StartGameMessage) -> GameController {
        GameController {
            world: game.world.clone(),
            old_world: game.world,
            config: Config::player_config(),
            turn: TurnMessage { changes: vec![] },
            stream: None,
            player_key: None
        }
    }

    /// Starts the current turn, by updating the player's GameWorld with changes
    /// made since the last time the player had a turn.
    pub fn start_turn(&mut self, turn: &StartTurnMessage) {
        self.old_world.start_turn(turn);
        self.world = self.old_world.clone();
        self.turn = TurnMessage { changes: vec![] };
    }

    /// Ends the current turn. Returns the list of changes made in this turn.
    /// Mainly for testing purposes; use next_turn().
    pub fn end_turn(&mut self) -> TurnMessage {
        self.world.flush_viewer_changes();
        self.turn.clone()
    }

    // ************************************************************************
    // ************************** GENERAL METHODS *****************************
    // ************************************************************************

    /// The current round, starting at round 1 and up to `ROUND_LIMIT` rounds.
    /// A round consists of a turn from each team on each planet.
    pub fn round(&self) -> Rounds {
        self.world.round()
    }

    /// The current planet.
    pub fn planet(&self) -> Planet {
        self.world.planet()
    }

    /// The team whose turn it is.
    pub fn team(&self) -> Team {
        self.world.team()
    }

    /// The starting map of the given planet. Includes the map's planet,
    /// dimensions, impassable terrain, and initial units and karbonite.
    pub fn starting_map(&self, planet: Planet) -> &PlanetMap {
        self.world.starting_map(planet)
    }

    /// The karbonite in the team's resource pool.
    pub fn karbonite(&self) -> u32 {
        self.world.karbonite()
    }

    // ************************************************************************
    // ************************** SENSING METHODS *****************************
    // ************************************************************************

    /// The single unit with this ID. Use this method to get detailed
    /// statistics on a unit: heat, cooldowns, and properties of special
    /// abilities like units garrisoned in a rocket.
    ///
    /// * NoSuchUnit - the unit does not exist (inside the vision range).
    pub fn unit_ref(&self, id: UnitID) -> Result<&Unit, Error> {
        self.world.unit_ref(id)
    }

    /// The single unit with this ID. Use this method to get detailed
    /// statistics on a unit: heat, cooldowns, and properties of special
    /// abilities like units garrisoned in a rocket.
    ///
    /// * NoSuchUnit - the unit does not exist (inside the vision range).
    pub fn unit(&self, id: UnitID) -> Result<Unit, Error> {
        self.world.unit(id)
    }

    /// All the units within the vision range, in no particular order.
    /// Does not include units in space.
    pub fn units_ref(&self) -> Vec<&Unit> {
        self.world.units_ref()
    }

    /// All the units within the vision range, in no particular order.
    /// Does not include units in space.
    pub fn units(&self) -> Vec<Unit> {
        self.world.units()
    }

    /// All the units on your team.
    /// Does not include units in space.
    pub fn my_units(&self) -> Vec<Unit> {
        let my_team = self.world.team();
        self.world.units().iter().filter(|u| u.team() == my_team).map(|u| u.clone()).collect()
    }

    /// All the units within the vision range, by ID.
    /// Does not include units in space.
    pub fn units_by_id(&self) -> FnvHashMap<UnitID, Unit> {
        self.world.units_by_id()
    }

    /// All the units within the vision range, by location.
    /// Does not include units in garrisons or in space.
    pub fn units_by_loc(&self) -> FnvHashMap<MapLocation, UnitID> {
        self.world.units_by_loc()
    }

    /// All the units of this team that are in space. You cannot see units
    /// on the other team that are in space.
    pub fn units_in_space(&self) -> Vec<Unit> {
        self.world.units_in_space()
    }

    /// The karbonite at the given location.
    ///
    /// * LocationOffMap - the location is off the map.
    /// * LocationNotVisible - the location is outside the vision range.
    pub fn karbonite_at(&self, location: MapLocation) -> Result<u32, Error> {
        self.world.karbonite_at(location)
    }

    /// Returns an array of all locations within a certain radius squared of
    /// this location that are on the map.
    ///
    /// The locations are ordered first by the x-coordinate, then the
    /// y-coordinate. The radius squared is inclusive.
    pub fn all_locations_within(&self, location: MapLocation,
                                radius_squared: u32) -> Vec<MapLocation> {
        self.world.all_locations_within(location, radius_squared)
    }

    /// Whether the location is on the map and within the vision range.
    pub fn can_sense_location(&self, location: MapLocation) -> bool {
        self.world.can_sense_location(location)
    }

    /// Whether there is a unit with this ID within the vision range.
    pub fn can_sense_unit(&self, id: UnitID) -> bool {
        self.world.can_sense_unit(id)
    }

    /// Sense units near the location within the given radius, inclusive, in
    /// distance squared. The units are within the vision range.
    pub fn sense_nearby_units(&self, location: MapLocation, radius: u32)
                              -> Vec<Unit> {
        self.world.sense_nearby_units(location, radius)
    }

    /// Sense units near the location within the given radius, inclusive, in
    /// distance squared. The units are within the vision range. Additionally
    /// filters the units by team.
    pub fn sense_nearby_units_by_team(&self, location: MapLocation,
                                      radius: u32, team: Team) -> Vec<Unit> {
        self.world.sense_nearby_units_by_team(location, radius, team)
    }

    /// Sense units near the location within the given radius, inclusive, in
    /// distance squared. The units are within the vision range. Additionally
    /// filters the units by unit type.
    pub fn sense_nearby_units_by_type(&self, location: MapLocation,
                                      radius: u32, unit_type: UnitType) -> Vec<Unit> {
        self.world.sense_nearby_units_by_type(location, radius, unit_type)
    }

    /// The unit at the location, if it exists.
    ///
    /// * LocationOffMap - the location is off the map.
    /// * LocationNotVisible - the location is outside the vision range.
    pub fn sense_unit_at_location_opt(&self, location: MapLocation)
                                  -> Result<Option<Unit>, Error> {
        self.world.sense_unit_at_location(location)
    }

    /// Whether there is a visible unit at a location.
    pub fn has_unit_at_location(&self, location: MapLocation) -> bool {
        self.world.sense_unit_at_location(location).is_ok()
    }

    /// The unit at the location, if it exists.
    ///
    /// * LocationOffMap - the location is off the map.
    /// * LocationNotVisible - the location is outside the vision range.
    pub fn sense_unit_at_location(&self, location: MapLocation)
                                  -> Result<Unit, Error> {
        let loc = self.world.sense_unit_at_location(location)?;
        if let Some(loc) = loc {
            Ok(loc)
        } else {
            bail!("No unit at location.")
        }
    }

    // ************************************************************************
    // ************************** WEATHER METHODS *****************************
    // ************************************************************************

    /// The asteroid strike pattern on Mars.
    pub fn asteroid_pattern(&self) -> AsteroidPattern {
        self.world.asteroid_pattern()
    }

    /// The orbit pattern that determines a rocket's flight duration.
    pub fn orbit_pattern(&self) -> OrbitPattern {
        self.world.orbit_pattern()
    }

    /// The current duration of flight if a rocket were to be launched this
    /// round. Does not take into account any research done on rockets.
    pub fn current_duration_of_flight(&self) -> Rounds {
        self.world.current_duration_of_flight()
    }

    // ************************************************************************
    // *********************** COMMUNICATION METHODS **************************
    // ************************************************************************

    /// Gets a read-only version of this planet's team array. If the given
    /// planet is different from the planet of the player, reads the version
    /// of the planet's team array from COMMUNICATION_DELAY rounds prior.
    pub fn get_team_array(&self, planet: Planet) -> &TeamArray {
        self.world.get_team_array(planet)
    }

    /// Writes the value at the index of this planet's team array.
    ///
    /// * ArrayOutOfBounds - the index of the array is out of
    ///   bounds. It must be within [0, COMMUNICATION_ARRAY_LENGTH).
    pub fn write_team_array(&mut self, index: usize, value: i32) -> Result<(), Error> {
        let delta = Delta::WriteTeamArray { index, value };
        self.world.apply(&delta)?;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta);
        }
        Ok(())
    }

    // ************************************************************************
    // ********************** UNIT DESTRUCTION METHODS ************************
    // ************************************************************************

    /// Disintegrates the unit and removes it from the map. If the unit is a
    /// factory or a rocket, also disintegrates any units garrisoned inside it.
    ///
    /// * NoSuchUnit - the unit does not exist (inside the vision range).
    /// * TeamNotAllowed - the unit is not on the current player's team.
    pub fn disintegrate_unit(&mut self, unit_id: UnitID) -> Result<(), Error> {
        let delta = Delta::Disintegrate { unit_id };
        self.world.apply(&delta)?;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta);
        }
        Ok(())
    }

    // ************************************************************************
    // ************************* LOCATION METHODS *****************************
    // ************************************************************************

    /// Whether the location is clear for a unit to occupy, either by movement
    /// or by construction.
    ///
    /// * LocationOffMap - the location is off the map.
    /// * LocationNotVisible - the location is outside the vision range.
    pub fn is_occupiable(&self, location: MapLocation) -> Result<bool, Error> {
        self.world.is_occupiable(location)
    }

    /// Whether the robot can move in the given direction, without taking into
    /// account the unit's movement heat. Takes into account only the map
    /// terrain, positions of other robots, and the edge of the game map.
    pub fn can_move(&self, robot_id: UnitID, direction: Direction) -> bool {
        self.world.can_move(robot_id, direction)
    }

    /// Whether the robot is ready to move. Tests whether the robot's attack
    /// heat is sufficiently low.
    pub fn is_move_ready(&self, robot_id: UnitID) -> bool {
        self.world.is_move_ready(robot_id)
    }

    /// Moves the robot in the given direction.
    ///
    /// * NoSuchUnit - the robot does not exist (within the vision range).
    /// * TeamNotAllowed - the robot is not on the current player's team.
    /// * UnitNotOnMap - the robot is not on the map.
    /// * LocationNotVisible - the location is outside the vision range.
    /// * LocationOffMap - the location is off the map.
    /// * LocationNotEmpty - the location is occupied by a unit or terrain.
    /// * Overheated - the robot is not ready to move again.
    pub fn move_robot(&mut self, robot_id: UnitID, direction: Direction) -> Result<(), Error> {
        let delta = Delta::Move { robot_id, direction };
        self.world.apply(&delta)?;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta);
        }
        Ok(())
    }

    // ************************************************************************
    // *************************** ATTACK METHODS *****************************
    // ************************************************************************
   
    /// Whether the robot can attack the given unit, without taking into
    /// account the robot's attack heat. Takes into account only the robot's
    /// attack range, and the location of the robot and target.
    ///
    /// Healers cannot attack, and should use `can_heal()` instead.
    pub fn can_attack(&self, robot_id: UnitID, target_unit_id: UnitID) -> bool {
        self.world.can_attack(robot_id, target_unit_id)
    }

    /// Whether the robot is ready to attack. Tests whether the robot's attack
    /// heat is sufficiently low.
    ///
    /// Healers cannot attack, and should use `is_heal_ready()` instead.
    pub fn is_attack_ready(&self, robot_id: UnitID) -> bool {
        self.world.is_attack_ready(robot_id)
    }

    /// Commands a robot to attack a unit, dealing the 
    /// robot's standard amount of damage.
    ///
    /// Healers cannot attack, and should use `heal()` instead.
    ///
    /// * NoSuchUnit - the unit does not exist (inside the vision range).
    /// * TeamNotAllowed - the unit is not on the current player's team.
    /// * InappropriateUnitType - the unit is not a robot, or is a healer.
    /// * UnitNotOnMap - the unit or target is not on the map.
    /// * OutOfRange - the target location is not in range.
    /// * Overheated - the unit is not ready to attack.
    pub fn attack(&mut self, robot_id: UnitID, target_unit_id: UnitID) -> Result<(), Error> {
        let delta = Delta::Attack { robot_id, target_unit_id };
        self.world.apply(&delta)?;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta);
        }
        Ok(())
    }

    // ************************************************************************
    // ************************* RESEARCH METHODS *****************************
    // ************************************************************************

    /// The research info of the current team, including what branch is
    /// currently being researched, the number of rounds left.
    pub fn research_info(&self) -> Result<ResearchInfo, Error> {
        Ok(self.world.research_info())
    }

    /// Resets the research queue to be empty. Returns true if the queue was
    /// not empty before, and false otherwise.
    pub fn reset_research(&mut self) -> Result<bool, Error> {
        let delta = Delta::ResetResearchQueue;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.reset_research())
    }

    /// Adds a branch to the back of the queue, if it is a valid upgrade, and
    /// starts research if it is the first in the queue.
    ///
    /// Returns whether the branch was successfully added.
    pub fn queue_research(&mut self, branch: UnitType) -> Result<bool, Error> {
        let delta = Delta::QueueResearch { branch };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.queue_research(branch))
    }

    // ************************************************************************
    // *************************** WORKER METHODS *****************************
    // ************************************************************************

    /// Whether the worker is ready to harvest, and the given direction contains
    /// karbonite to harvest. The worker cannot already have performed an action 
    /// this round.
    pub fn can_harvest(&self, worker_id: UnitID, direction: Direction) -> bool {
        self.world.can_harvest(worker_id, direction)
    }

    /// Harvests up to the worker's harvest amount of karbonite from the given
    /// location, adding it to the team's resource pool.
    ///
    /// * NoSuchUnit - the worker does not exist (within the vision range).
    /// * TeamNotAllowed - the worker is not on the current player's team.
    /// * InappropriateUnitType - the unit is not a worker.
    /// * Overheated - the worker has already performed an action this turn.
    /// * UnitNotOnMap - the worker is not on the map.
    /// * LocationOffMap - the location in the target direction is off the map.
    /// * LocationNotVisible - the location is not in the vision range.
    /// * KarboniteDepositEmpty - the location described contains no Karbonite.
    pub fn harvest(&mut self, worker_id: UnitID, direction: Direction)
                   -> Result<(), Error> {
        let delta = Delta::Harvest { worker_id, direction };
        self.world.apply(&delta)?;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta);
        }
        Ok(())
    }

    /// Whether the worker can blueprint a unit of the given type. The worker
    /// can only blueprint factories, and rockets if Rocketry has been
    /// researched. The team must have sufficient karbonite in its resource
    /// pool. The worker cannot already have performed an action this round.
    pub fn can_blueprint(&self, worker_id: UnitID, unit_type: UnitType,
                         direction: Direction) -> bool {
        self.world.can_blueprint(worker_id, unit_type, direction)
    }

    /// Blueprints a unit of the given type in the given direction. Subtract
    /// cost of that unit from the team's resource pool.
    ///
    /// * NoSuchUnit - the worker does not exist (within the vision range).
    /// * TeamNotAllowed - the worker is not on the current player's team.
    /// * InappropriateUnitType - the unit is not a worker, or the unit type
    ///   is not a structure.
    /// * Overheated - the worker has already performed an action this turn.
    /// * UnitNotOnMap - the unit is not on the map.
    /// * LocationOffMap - the location in the target direction is off the map.
    /// * LocationNotVisible - the location is outside the vision range.
    /// * LocationNotEmpty - the location in the target direction is already
    ///   occupied.
    /// * CannotBuildOnMars - you cannot blueprint a structure on Mars.
    /// * ResearchNotUnlocked - you do not have the needed research to blueprint rockets.
    /// * InsufficientKarbonite - your team does not have enough Karbonite to
    ///   build the requested structure.
    pub fn blueprint(&mut self, worker_id: UnitID, structure_type: UnitType,
                     direction: Direction) -> Result<(), Error> {
        let delta = Delta::Blueprint { worker_id, structure_type, direction };
        self.world.apply(&delta)?;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta);
        }
        Ok(())
    }

    /// Whether the worker can build a blueprint with the given ID. The worker
    /// and the blueprint must be adjacent to each other. The worker cannot
    /// already have performed an action this round.
    pub fn can_build(&self, worker_id: UnitID, blueprint_id: UnitID) -> bool {
        self.world.can_build(worker_id, blueprint_id)
    }

    /// Builds a given blueprint, increasing its health by the worker's build
    /// amount. If raised to maximum health, the blueprint becomes a completed
    /// structure.
    ///
    /// * NoSuchUnit - either unit does not exist (within the vision range).
    /// * TeamNotAllowed - either unit is not on the current player's team.
    /// * UnitNotOnMap - the worker is not on the map.
    /// * InappropriateUnitType - the unit is not a worker, or the blueprint
    ///   is not a structure.
    /// * Overheated - the worker has already performed an action this turn.
    /// * OutOfRange - the worker is not adjacent to the blueprint.
    /// * StructureAlreadyBuilt - the blueprint has already been completed.
    pub fn build(&mut self, worker_id: UnitID, blueprint_id: UnitID)
                 -> Result<(), Error> {
        let delta = Delta::Build { worker_id, blueprint_id };
        self.world.apply(&delta)?;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta);
        }
        Ok(())
    }

    /// Whether the given worker can repair the given strucutre. Tests that the worker
    /// is able to execute a worker action, that the structure is built, and that the
    /// structure is within range.
    pub fn can_repair(&self, worker_id: UnitID, structure_id: UnitID) -> bool {
        self.world.can_repair(worker_id, structure_id)
    }

    /// Commands the worker to repair a structure, repleneshing health to it. This
    /// can only be done to structures which have been fully built.
    ///
    /// * NoSuchUnit - either unit does not exist (within the vision range).
    /// * TeamNotAllowed - either unit is not on the current player's team.
    /// * UnitNotOnMap - the worker is not on the map.
    /// * InappropriateUnitType - the unit is not a worker, or the target
    ///   is not a structure.
    /// * Overheated - the worker has already performed an action this turn.
    /// * OutOfRange - the worker is not adjacent to the structure.
    /// * StructureNotYetBuilt - the structure has not been completed.
    pub fn repair(&mut self, worker_id: UnitID, structure_id: UnitID) -> Result<(), Error> {
        let delta = Delta::Repair { worker_id, structure_id };
        self.world.apply(&delta)?;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta);
        }
        Ok(())
    }

    /// Whether the worker is ready to replicate. Tests that the worker's
    /// ability heat is sufficiently low, that the team has sufficient
    /// karbonite in its resource pool, and that the square in the given
    /// direction is empty.
    pub fn can_replicate(&self, worker_id: UnitID, direction: Direction) -> bool {
        self.world.can_replicate(worker_id, direction)
    }

    /// Replicates a worker in the given direction. Subtracts the cost of the
    /// worker from the team's resource pool.
    ///
    /// * NoSuchUnit - the worker does not exist (within the vision range).
    /// * TeamNotAllowed - the worker is not on the current player's team.
    /// * InappropriateUnitType - the unit is not a worker.
    /// * Overheated - the worker is not ready to replicate again.
    /// * InsufficientKarbonite - your team does not have enough Karbonite for
    ///   the worker to replicate.
    /// * UnitNotOnMap - the worker is not on the map.
    /// * LocationOffMap - the location in the target direction is off the map.
    /// * LocationNotVisible - the location is outside the vision range.
    /// * LocationNotEmpty - the location in the target direction is already
    ///   occupied.
    pub fn replicate(&mut self, worker_id: UnitID, direction: Direction)
                     -> Result<(), Error> {
        let delta = Delta::Replicate { worker_id, direction };
        self.world.apply(&delta)?;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta);
        }
        Ok(())
    }

    // ************************************************************************
    // *************************** KNIGHT METHODS *****************************
    // ************************************************************************

    /// Whether the knight can javelin the given robot, without taking into
    /// account the knight's ability heat. Takes into account only the knight's
    /// ability range, and the location of the robot.
    pub fn can_javelin(&self, knight_id: UnitID, target_unit_id: UnitID) -> bool {
        self.world.can_javelin(knight_id, target_unit_id)
    }

    /// Whether the knight is ready to javelin. Tests whether the knight's
    /// ability heat is sufficiently low.
    pub fn is_javelin_ready(&self, knight_id: UnitID) -> bool {
        self.world.is_javelin_ready(knight_id)
    }

    /// Javelins the robot, dealing the knight's standard damage.
    ///
    /// * NoSuchUnit - either unit does not exist (inside the vision range).
    /// * TeamNotAllowed - the knight is not on the current player's team.
    /// * UnitNotOnMap - the knight is not on the map.
    /// * InappropriateUnitType - the unit is not a knight.
    /// * ResearchNotUnlocked - you do not have the needed research to use javelin.
    /// * OutOfRange - the target does not lie within ability range of the knight.
    /// * Overheated - the knight is not ready to use javelin again.
    pub fn javelin(&mut self, knight_id: UnitID, target_unit_id: UnitID) -> Result<(), Error> {
        let delta = Delta::Javelin { knight_id, target_unit_id };
        self.world.apply(&delta)?;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta);
        }
        Ok(())
    }

    // ************************************************************************
    // *************************** RANGER METHODS *****************************
    // ************************************************************************

    /// Whether the ranger can begin to snipe the given location, without
    /// taking into account the ranger's ability heat. Takes into account only
    /// the target location and the unit's type and unlocked abilities.
    pub fn can_begin_snipe(&self, ranger_id: UnitID, location: MapLocation) -> bool {
        self.world.can_begin_snipe(ranger_id, location)
    }

    /// Whether the ranger is ready to begin snipe. Tests whether the ranger's
    /// ability heat is sufficiently low.
    pub fn is_begin_snipe_ready(&self, ranger_id: UnitID) -> bool {
        self.world.is_begin_snipe_ready(ranger_id)
    }

    /// Begins the countdown to snipe a given location. Maximizes the units
    /// attack and movement heats until the ranger has sniped. The ranger may
    /// begin the countdown at any time, including resetting the countdown
    /// to snipe a different location.
    ///
    /// * NoSuchUnit - either unit does not exist (inside the vision range).
    /// * TeamNotAllowed - the ranger is not on the current player's team.
    /// * UnitNotOnMap - the ranger is not on the map.
    /// * InappropriateUnitType - the unit is not a ranger.
    /// * ResearchNotUnlocked - you do not have the needed research to use snipe.
    /// * Overheated - the ranger is not ready to use snipe again.
    pub fn begin_snipe(&mut self, ranger_id: UnitID, location: MapLocation)
                       -> Result<(), Error> {
        let delta = Delta::BeginSnipe { ranger_id, location };
        self.world.apply(&delta)?;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta);
        }
        Ok(())
    }

    // ************************************************************************
    // **************************** MAGE METHODS ******************************
    // ************************************************************************
    
    /// Whether the mage can blink to the given location, without taking into
    /// account the mage's ability heat. Takes into account only the mage's
    /// ability range, the map terrain, positions of other units, and the edge
    /// of the game map.
    pub fn can_blink(&self, mage_id: UnitID, location: MapLocation) -> bool {
        self.world.can_blink(mage_id, location)
    }

    /// Whether the mage is ready to blink. Tests whether the mage's ability
    /// heat is sufficiently low.
    pub fn is_blink_ready(&self, mage_id: UnitID) -> bool {
        self.world.is_blink_ready(mage_id)
    }

    /// Blinks the mage to the given location.
    ///
    /// * NoSuchUnit - the mage does not exist (inside the vision range).
    /// * TeamNotAllowed - the mage is not on the current player's team.
    /// * UnitNotOnMap - the mage is not on the map.
    /// * InappropriateUnitType - the unit is not a mage.
    /// * ResearchNotUnlocked - you do not have the needed research to use blink.
    /// * OutOfRange - the target does not lie within ability range of the mage.
    /// * LocationOffMap - the target location is not on this planet's map.
    /// * LocationNotVisible - the target location is outside the vision range.
    /// * LocationNotEmpty - the target location is already occupied.
    /// * Overheated - the mage is not ready to use blink again.
    pub fn blink(&mut self, mage_id: UnitID, location: MapLocation) -> Result<(), Error> {
        let delta = Delta::Blink { mage_id, location };
        self.world.apply(&delta)?;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta);
        }
        Ok(())
    }

    // ************************************************************************
    // *************************** HEALER METHODS *****************************
    // ************************************************************************

    /// Whether the healer can heal the given robot, without taking into
    /// account the healer's attack heat. Takes into account only the healer's
    /// attack range, and the location of the robot.
    pub fn can_heal(&self, healer_id: UnitID, target_robot_id: UnitID) -> bool {
        self.world.can_heal(healer_id, target_robot_id)
    }

    /// Whether the healer is ready to heal. Tests whether the healer's attack
    /// heat is sufficiently low.
    pub fn is_heal_ready(&self, healer_id: UnitID) -> bool {
        self.world.is_heal_ready(healer_id)
    }

    /// Commands the healer to heal the target robot.
    ///
    /// * NoSuchUnit - either unit does not exist (inside the vision range).
    /// * InappropriateUnitType - the unit is not a healer, or the target is not
    ///   a robot.
    /// * TeamNotAllowed - either robot is not on the current player's team.
    /// * UnitNotOnMap - the healer is not on the map.
    /// * OutOfRange - the target does not lie within "attack" range of the healer.
    /// * Overheated - the healer is not ready to heal again.
    pub fn heal(&mut self, healer_id: UnitID, target_robot_id: UnitID) -> Result<(), Error> {
        let delta = Delta::Heal { healer_id, target_robot_id };
        self.world.apply(&delta)?;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta);
        }
        Ok(())
    }

    /// Whether the healer can overcharge the given robot, without taking into
    /// account the healer's ability heat. Takes into account only the healer's
    /// ability range, and the location of the robot.
    pub fn can_overcharge(&self, healer_id: UnitID, target_robot_id: UnitID) -> bool {
        self.world.can_overcharge(healer_id, target_robot_id)
    }

    /// Whether the healer is ready to overcharge. Tests whether the healer's
    /// ability heat is sufficiently low.
    pub fn is_overcharge_ready(&self, healer_id: UnitID) -> bool {
        self.world.is_overcharge_ready(healer_id)
    }

    /// Overcharges the robot, resetting the robot's cooldowns. The robot must
    /// be on the same team as you.
    ///
    /// * NoSuchUnit - either unit does not exist (inside the vision range).
    /// * TeamNotAllowed - either robot is not on the current player's team.
    /// * UnitNotOnMap - the healer is not on the map.
    /// * InappropriateUnitType - the unit is not a healer, or the target is not
    ///   a robot.
    /// * ResearchNotUnlocked - you do not have the needed research to use overcharge.
    /// * OutOfRange - the target does not lie within ability range of the healer.
    /// * Overheated - the healer is not ready to use overcharge again.
    pub fn overcharge(&mut self, healer_id: UnitID, target_robot_id: UnitID)
                      -> Result<(), Error> {
        let delta = Delta::Overcharge { healer_id, target_robot_id };
        self.world.apply(&delta)?;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta);
        }
        Ok(())
    }

    // ************************************************************************
    // ************************* STRUCTURE METHODS ****************************
    // ************************************************************************

    /// Whether the robot can be loaded into the given structure's garrison. The robot
    /// must be ready to move and must be adjacent to the structure. The structure
    /// and the robot must be on the same team, and the structure must have space.
    pub fn can_load(&self, structure_id: UnitID, robot_id: UnitID) -> bool {
        self.world.can_load(structure_id, robot_id)
    }

    /// Loads the robot into the garrison of the structure.
    ///
    /// * NoSuchUnit - either unit does not exist (inside the vision range).
    /// * TeamNotAllowed - either unit is not on the current player's team.
    /// * UnitNotOnMap - either unit is not on the map.
    /// * Overheated - the robot is not ready to move again.
    /// * InappropriateUnitType - the first unit is not a structure, or the
    ///   second unit is not a robot.
    /// * StructureNotYetBuilt - the structure has not yet been completed.
    /// * GarrisonFull - the structure's garrison is already full.
    /// * OutOfRange - the robot is not adjacent to the structure.
    pub fn load(&mut self, structure_id: UnitID, robot_id: UnitID)
                    -> Result<(), Error> {
        let delta = Delta::Load { structure_id, robot_id };
        self.world.apply(&delta)?;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta);
        }
        Ok(())
    }

    /// Tests whether the given structure is able to unload a unit in the
    /// given direction. There must be space in that direction, and the unit
    /// must be ready to move.
    pub fn can_unload(&self, structure_id: UnitID, direction: Direction) -> bool {
        self.world.can_unload(structure_id, direction)
    }

    /// Unloads a robot from the garrison of the specified structure into an 
    /// adjacent space. Robots are unloaded in the order they were loaded.
    ///
    /// * NoSuchUnit - the unit does not exist (inside the vision range).
    /// * TeamNotAllowed - either unit is not on the current player's team.
    /// * UnitNotOnMap - the structure is not on the map.
    /// * InappropriateUnitType - the unit is not a structure.
    /// * StructureNotYetBuilt - the structure has not yet been completed.
    /// * GarrisonEmpty - the structure's garrison is already empty.
    /// * LocationOffMap - the location in the target direction is off the map.
    /// * LocationNotEmpty - the location in the target direction is already
    ///   occupied.
    /// * Overheated - the robot inside the structure is not ready to move again.
    pub fn unload(&mut self, structure_id: UnitID, direction: Direction)
                      -> Result<(), Error> {
        let delta = Delta::Unload { structure_id, direction };
        self.world.apply(&delta)?;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta);
        }
        Ok(())
    }

    // ************************************************************************
    // ************************** FACTORY METHODS *****************************
    // ************************************************************************

    /// Whether the factory can produce a robot of the given type. The factory
    /// must not currently be producing a robot, and the team must have
    /// sufficient resources in its resource pool.
    pub fn can_produce_robot(&mut self, factory_id: UnitID, robot_type: UnitType) -> bool {
        self.world.can_produce_robot(factory_id, robot_type)
    }

    /// Starts producing the robot of the given type.
    ///
    /// * NoSuchUnit - the factory does not exist (inside the vision range).
    /// * TeamNotAllowed - the factory is not on the current player's team.
    /// * InappropriateUnitType - the unit is not a factory, or the unit type
    ///   is not a robot.
    /// * StructureNotYetBuilt - the factory has not yet been completed.
    /// * FactoryBusy - the factory is already producing a unit.
    /// * InsufficientKarbonite - your team does not have enough Karbonite to
    ///   produce the given robot.
    pub fn produce_robot(&mut self, factory_id: UnitID, robot_type: UnitType)
                       -> Result<(), Error> {
        let delta = Delta::ProduceRobot { factory_id, robot_type };
        self.world.apply(&delta)?;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta);
        }
        Ok(())
    }

    // ************************************************************************
    // *************************** ROCKET METHODS *****************************
    // ************************************************************************

    /// The landing rounds and locations of rockets in space that belong to the
    /// current team.
    pub fn rocket_landings(&self) -> RocketLandingInfo {
        self.world.rocket_landings()
    }

    /// Whether the rocket can launch into space to the given destination. The
    /// rocket can launch if the it has never been used before. The destination
    /// is valid if it contains passable terrain on the other planet.
    pub fn can_launch_rocket(&self, rocket_id: UnitID, destination: MapLocation) -> bool {
        self.world.can_launch_rocket(rocket_id, destination)
    }

    /// Launches the rocket into space, damaging the units adjacent to the
    /// takeoff location.
    ///
    /// * NoSuchUnit - the rocket does not exist (inside the vision range).
    /// * TeamNotAllowed - the rocket is not on the current player's team.
    /// * SamePlanet - the rocket cannot fly to a location on the same planet.
    /// * InappropriateUnitType - the unit is not a rocket.
    /// * StructureNotYetBuilt - the rocket has not yet been completed.
    /// * RocketUsed - the rocket has already been used.
    /// * LocationOffMap - the given location is off the map.
    /// * LocationNotEmpty - the given location contains impassable terrain.
    pub fn launch_rocket(&mut self, rocket_id: UnitID, location: MapLocation)
                         -> Result<(), Error> {
        let delta = Delta::LaunchRocket { rocket_id, location };
        self.world.apply(&delta)?;
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta);
        }
        Ok(())
    }

    // ************************************************************************
    // ************************************************************************
    // ************************************************************************
    // *************************** MANAGER API ********************************
    // ************************************************************************
    // ************************************************************************
    // ************************************************************************

    /// Initializes the game world and creates a new controller
    /// for the manager to interact with it.
    pub fn new_manager(map: GameMap) -> GameController {
        let world = GameWorld::new(map);
        GameController {
            world: world.clone(),
            old_world: world,
            config: Config::runner_config(),
            turn: TurnMessage { changes: vec![] },
            stream: None,
            player_key: None
        }
    }

    /// The start turn message to send to the first player to move. Should
    /// only be called on Round 1, and should only be sent to Red Earth.
    ///
    /// Panics if we're past Round 1...
    pub fn initial_start_turn_message(&self) -> InitialTurnApplication {
        let mut world = self.world.clone();
        world.cached_world.clear();
        InitialTurnApplication {
            start_turn: self.world.initial_start_turn_message(), 
            viewer: ViewerKeyframe { world }
        }
    }

    /// Get the first message to send to each player and initialize the world.
    pub fn start_game(&self, player: Player) -> StartGameMessage {
        StartGameMessage {
            world: self.world.cached_world(player).clone(),
        }
    }

    /// Given a TurnMessage from a player, apply those changes.
    /// Receives the StartTurnMessage for the next player.
   pub fn apply_turn(&mut self, turn: &TurnMessage) -> TurnApplication {
        // Serialize the filtered game state to send to the player
        let start_turn = self.world.apply_turn(turn);
        // Serialize the game state to send to the viewer
        let viewer = ViewerMessage { 
            changes: turn.changes.clone(),
            units: self.world.get_viewer_units(),
            additional_changes: self.world.flush_viewer_changes(),
            karbonite: self.world.karbonite(),
        };
        TurnApplication {
            start_turn, viewer
        }
    }    
    
    /// Determines if the game has ended, returning the winning team if so.
    pub fn is_game_over(&self) -> Option<Team> {
        self.world.is_game_over()
    }

    pub fn is_over(&self) -> bool {
        self.is_game_over().is_some()
    }

    pub fn winning_team(&self) -> Result<Team, Error> {
        if let Some(team) = self.is_game_over() {
            Ok(team)
        } else {
            bail!("Game is not finished");
        }
    }
}

/// Returned from apply_turn.
/// This struct only exists because the bindings don't do tuples yet.
#[derive(Debug, Clone)]
pub struct TurnApplication {
    pub start_turn: StartTurnMessage,
    pub viewer: ViewerMessage
}

/// Returned from initial_start_turn_message.
#[derive(Debug, Clone)]
pub struct InitialTurnApplication {
    pub start_turn: StartTurnMessage,
    pub viewer: ViewerKeyframe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn() {
        // NOTE: This test depends on movement working properly.
        let red_player = Player::new(Team::Red, Planet::Earth);
        let blue_player = Player::new(Team::Blue, Planet::Earth);

        // Place some robots manually on the Earth map, for sake of testing.
        let mut map = GameMap::test_map();
        map.earth_map.initial_units = vec![
            Unit::new(1, Team::Red, UnitType::Knight, 0,
                Location::OnMap(MapLocation::new(Planet::Earth, 0, 0))).unwrap(),
            Unit::new(2, Team::Blue, UnitType::Knight, 0,
                Location::OnMap(MapLocation::new(Planet::Earth, 1, 0))).unwrap(),
        ];

        // Create a controller for the manager.
        let mut manager_controller = GameController::new_manager(map);
        let red_robot = 1;
        let blue_robot = 2;
        
        // Create controllers for each Earth player, so the first
        // player can see the new robots, then start red's turn.
        let red_start_game_msg = manager_controller.start_game(red_player);
        let blue_start_game_msg = manager_controller.start_game(blue_player);
        let mut player_controller_red = GameController::new_player(red_start_game_msg);
        let mut player_controller_blue = GameController::new_player(blue_start_game_msg);

        // Send the first STM to red and test that red can move as expected.
        let initial_start_turn_msg = manager_controller.initial_start_turn_message();
        player_controller_red.start_turn(&initial_start_turn_msg.start_turn);
        assert![!player_controller_red.can_move(red_robot, Direction::East)];
        assert![player_controller_red.can_move(red_robot, Direction::Northeast)];
        assert![player_controller_red.move_robot(red_robot, Direction::Northeast).is_ok()];

        // End red's turn, and pass the message to the manager, which
        // generates blue's start turn message and starts blue's turn.
        let red_turn_msg = player_controller_red.end_turn();
        let application = manager_controller.apply_turn(&red_turn_msg);
        let blue_start_turn_msg = application.start_turn;
        player_controller_blue.start_turn(&blue_start_turn_msg);

        // Test that blue can move as expected. This demonstrates
        // it has received red's actions in its own state.
        assert![!player_controller_blue.can_move(blue_robot, Direction::North)];
        assert![player_controller_blue.can_move(blue_robot, Direction::West)];
        assert![player_controller_blue.move_robot(blue_robot, Direction::West).is_ok()];
    }

    #[test]
    fn test_serialization() {
        use serde_json::to_string;
        let mut c = GameController::new_manager(GameMap::test_map());
        println!("----start");
        println!("{}", to_string(&c.start_game(Player::new(Team::Red, Planet::Earth))).unwrap());
        println!("----initial");
        let initial = c.initial_start_turn_message();
        println!("----initial planet_maps");
        println!("{}", to_string(&initial.viewer.world.planet_maps).unwrap());
        println!("----initial planet_states");
        println!("{}", to_string(&initial.viewer.world.planet_states).unwrap());
        println!("----initial team_states");
        println!("{}", to_string(&initial.viewer.world.team_states).unwrap());
        println!("----initial cached_world {}", initial.viewer.world.cached_world.len());
        println!("{}", to_string(&initial.viewer.world.cached_world).unwrap());
        println!("----apply");
        let t = TurnMessage { changes: vec![] };
        let a = c.apply_turn(&t);
        println!("{}", to_string(&a.viewer).unwrap());

    }
}