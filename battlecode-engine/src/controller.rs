//! The outermost layer of the engine stack. Responsible for exposing
//! the API that the player will use, and for generating messages to
//! send to other parts of the Battlecode infrastructure.

use config::Config;
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

pub struct GameController {
    world: GameWorld,
    old_world: GameWorld,
    config: Config,
    turn: TurnMessage,
}

impl GameController {

    // ************************************************************************
    // ************************************************************************
    // ************************************************************************
    // **************************** PLAYER API ********************************
    // ************************************************************************
    // ************************************************************************
    // ************************************************************************

    /// Initializes the game world and creates a new controller
    /// for a player to interact with it.
    pub fn new_player(game: StartGameMessage) -> GameController {
        GameController {
            world: game.world.clone(),
            old_world: game.world,
            config: Config::player_config(),
            turn: TurnMessage { changes: vec![] }
        }
    }

    /// Starts the current turn, by updating the player's GameWorld with changes
    /// made since the last time the player had a turn.
    pub fn start_turn(&mut self, turn: &StartTurnMessage) -> Result<(), Error> {
        self.old_world.start_turn(turn);
        self.world = self.old_world.clone();
        self.turn = TurnMessage { changes: vec![] };
        Ok(())
    }

    /// Ends the current turn. Returns the list of changes made in this turn.
    pub fn end_turn(&mut self) -> Result<TurnMessage, Error> {
        Ok(self.turn.clone())
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

    /// The unit controller for the unit of this ID. Use this method to get
    /// detailed statistics on a unit in your team: heat, cooldowns, and
    /// properties of special abilities like units garrisoned in a rocket.
    ///
    /// Note that mutating this object does NOT have any effect on the actual
    /// game. You MUST call the mutators in world!!
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    pub fn unit_controller(&self, id: UnitID) -> Result<&Unit, Error> {
        self.world.unit_controller(id)
    }

    /// The single unit with this ID.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    pub fn unit(&self, id: UnitID) -> Result<UnitInfo, Error> {
        self.world.unit(id)
    }

    /// All the units within the vision range, in no particular order.
    /// Does not include units in space.
    pub fn units_ref(&self) -> Vec<&UnitInfo> {
        self.world.units_ref()
    }

    /// All the units within the vision range, in no particular order.
    /// Does not include units in space.
    pub fn units(&self) -> Vec<UnitInfo> {
        self.world.units()
    }

    /// All the units within the vision range, by ID.
    /// Does not include units in space.
    pub fn units_by_id(&self) -> FnvHashMap<UnitID, UnitInfo> {
        self.world.units_by_id()
    }

    /// All the units within the vision range, by location.
    /// Does not include units in garrisons or in space.
    pub fn units_by_loc(&self) -> FnvHashMap<MapLocation, UnitID> {
        self.world.units_by_loc()
    }

    /// All the units of this team that are in space.
    pub fn units_in_space(&self) -> Vec<UnitInfo> {
        self.world.units_in_space()
    }

    /// The karbonite at the given location.
    ///
    /// * GameError::InvalidLocation - the location is outside the vision range.
    pub fn karbonite_at(&self, location: MapLocation) -> Result<u32, Error> {
        Ok(self.world.karbonite_at(location)?)
    }

    /// Whether the location is within the vision range.
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
                              -> Vec<UnitInfo> {
        self.world.sense_nearby_units(location, radius)
    }

    /// Sense units near the location within the given radius, inclusive, in
    /// distance squared. The units are within the vision range. Additionally
    /// filters the units by team.
    pub fn sense_nearby_units_by_team(&self, location: MapLocation,
                                      radius: u32, team: Team) -> Vec<UnitInfo> {
        self.world.sense_nearby_units_by_team(location, radius, team)
    }

    /// Sense units near the location within the given radius, inclusive, in
    /// distance squared. The units are within the vision range. Additionally
    /// filters the units by unit type.
    pub fn sense_nearby_units_by_type(&self, location: MapLocation,
                                      radius: u32, unit_type: UnitType) -> Vec<UnitInfo> {
        self.world.sense_nearby_units_by_type(location, radius, unit_type)
    }

    /// The unit at the location, if it exists.
    ///
    /// * GameError::InvalidLocation - the location is outside the vision range.
    pub fn sense_unit_at_location(&self, location: MapLocation)
                                  -> Result<Option<UnitInfo>, Error> {
        Ok(self.world.sense_unit_at_location(location)?)
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
    /// * GameError::ArrayOutOfBounds - the index of the array is out of
    ///   bounds. It must be within [0, COMMUNICATION_ARRAY_LENGTH).
    pub fn write_team_array(&mut self, index: usize, value: i32) -> Result<(), Error> {
        let delta = Delta::WriteTeamArray { index, value };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
    }

    // ************************************************************************
    // ********************** UNIT DESTRUCTION METHODS ************************
    // ************************************************************************

    /// Disintegrates the unit and removes it from the map. If the unit is a
    /// factory or a rocket, also disintegrates any units garrisoned inside it.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    pub fn disintegrate_unit(&mut self, unit_id: UnitID) -> Result<(), Error> {
        let delta = Delta::Disintegrate { unit_id };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
    }

    // ************************************************************************
    // ************************* LOCATION METHODS *****************************
    // ************************************************************************

    /// Whether the location is clear for a unit to occupy, either by movement
    /// or by construction.
    ///
    /// * GameError::InvalidLocation - the location is outside the vision range.
    pub fn is_occupiable(&self, location: MapLocation) -> Result<bool, Error> {
        Ok(self.world.is_occupiable(location)?)
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
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a robot.
    /// * GameError::InvalidAction - the robot cannot move in that direction.
    pub fn move_robot(&mut self, robot_id: UnitID, direction: Direction) -> Result<(), Error> {
        let delta = Delta::Move { robot_id, direction };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
    }

    // ************************************************************************
    // *************************** ATTACK METHODS *****************************
    // ************************************************************************
   

    /// Whether the robot can attack the given unit, without taking into
    /// account the unit's attack heat. Takes into account only the unit's
    /// attack range, and the location of the unit.
    pub fn can_attack(&self, robot_id: UnitID, target_unit_id: UnitID) -> bool {
        self.world.can_attack(robot_id, target_unit_id)
    }

    /// Whether the robot is ready to attack. Tests whether the robot's attack
    /// heat is sufficiently low.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is a healer, or not a robot.
    pub fn is_attack_ready(&self, robot_id: UnitID) -> bool {
        self.world.is_attack_ready(robot_id)
    }

    /// Attacks the robot, dealing the unit's standard amount of damage.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is a healer, or not a robot.
    /// * GameError::InvalidAction - the robot cannot attack that location.
    pub fn attack(&mut self, robot_id: UnitID, target_unit_id: UnitID) -> Result<(), Error> {
        let delta = Delta::Attack { robot_id, target_unit_id };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
    }

    // ************************************************************************
    // ************************* RESEARCH METHODS *****************************
    // ************************************************************************

    /// The research info of the current team, including what branch is
    /// currently being researched, the number of rounds left.
    ///
    /// Note that mutating this object by resetting or queueing research
    /// does not have any effect. You must call the mutators on world.
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
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a worker.
    /// * GameError::InvalidLocation - the location is off the map.
    /// * GameError::InvalidAction - the worker is not ready to harvest, or there is no karbonite.
    pub fn harvest(&mut self, worker_id: UnitID, direction: Direction)
                   -> Result<(), Error> {
        let delta = Delta::Harvest { worker_id, direction };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
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
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a worker, or the
    ///   unit type is not a factory or rocket.
    /// * GameError::InvalidLocation - the location is off the map.
    /// * GameError::InvalidAction - the worker is not ready to blueprint.
    pub fn blueprint(&mut self, worker_id: UnitID, structure_type: UnitType,
                     direction: Direction) -> Result<(), Error> {
        let delta = Delta::Blueprint { worker_id, structure_type, direction };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
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
    /// * GameError::NoSuchUnit - a unit does not exist.
    /// * GameError::TeamNotAllowed - a unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit or blueprint is the wrong type.
    /// * GameError::InvalidAction - the worker cannot build the blueprint.
    pub fn build(&mut self, worker_id: UnitID, blueprint_id: UnitID)
                 -> Result<(), Error> {
        let delta = Delta::Build { worker_id, blueprint_id };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
    }

    /// Whether the given worker can repair the given strucutre. Tests that the worker
    /// is able to execute a worker action, that the structure is built, and that the
    /// structure is within range.
    pub fn can_repair(&self, worker_id: UnitID, structure_id: UnitID) -> bool {
        self.world.can_repair(worker_id, structure_id)
    }

    /// Commands the worker to repair a structure, repleneshing health to it. This
    /// can only be done to structures which have been fully built.
    pub fn repair(&mut self, worker_id: UnitID, structure_id: UnitID) -> Result<(), Error> {
        let delta = Delta::Repair { worker_id, structure_id };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
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
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a worker.
    /// * GameError::InvalidLocation - the location is off the map.
    /// * GameError::InvalidAction - the worker is not ready to replicate.
    pub fn replicate(&mut self, worker_id: UnitID, direction: Direction)
                     -> Result<(), Error> {
        let delta = Delta::Replicate { worker_id, direction };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
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

    /// Javelins the robot, dealing the amount of ability damage.
    ///
    /// * GameError::InvalidResearchLevel - the ability has not been researched.
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a knight.
    /// * GameError::InvalidAction - the knight cannot javelin that unit.
    pub fn javelin(&mut self, knight_id: UnitID, target_unit_id: UnitID) -> Result<(), Error> {
        let delta = Delta::Javelin { knight_id, target_unit_id };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
    }

    // ************************************************************************
    // *************************** RANGER METHODS *****************************
    // ************************************************************************

    /// Begins the countdown to snipe a given location. Maximizes the units
    /// attack and movement heats until the ranger has sniped. The ranger may
    /// begin the countdown at any time, including resetting the countdown
    /// to snipe a different location.
    ///
    /// * GameError::InvalidResearchLevel - the ability has not been researched.
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a ranger.
    /// * GameError::InvalidLocation - the location is off the map or on a different planet.
    pub fn begin_snipe(&mut self, ranger_id: UnitID, location: MapLocation)
                       -> Result<(), Error> {
        let delta = Delta::BeginSnipe { ranger_id, location };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
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
    ///
    /// * GameError::InvalidResearchLevel - the ability has not been researched.
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a mage.
    pub fn is_blink_ready(&self, mage_id: UnitID) -> bool {
        self.world.is_blink_ready(mage_id)
    }

    /// Blinks the mage to the given location.
    ///
    /// * GameError::InvalidResearchLevel - the ability has not been researched.
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a mage.
    /// * GameError::InvalidAction - the mage cannot blink to that location.
    pub fn blink(&mut self, mage_id: UnitID, location: MapLocation) -> Result<(), Error> {
        let delta = Delta::Blink { mage_id, location };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
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

    /// Heals the robot, dealing the healer's standard amount of "damage".
    ///
    /// * GameError::NoSuchUnit - a unit does not exist.
    /// * GameError::TeamNotAllowed - the first unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the healer or robot is not the right type.
    /// * GameError::InvalidAction - the healer cannot heal that unit.
    pub fn heal(&mut self, healer_id: UnitID, target_robot_id: UnitID) -> Result<(), Error> {
        let delta = Delta::Heal { healer_id, target_robot_id };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
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

    /// Overcharges the robot, resetting the robot's cooldowns.
    ///
    /// * GameError::InvalidResearchLevel - the ability has not been researched.
    /// * GameError::NoSuchUnit - a unit does not exist.
    /// * GameError::TeamNotAllowed - the first unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the healer or robot is not the right type.
    /// * GameError::InvalidAction - the healer cannot overcharge that unit.
    pub fn overcharge(&mut self, healer_id: UnitID, target_robot_id: UnitID)
                      -> Result<(), Error> {
        let delta = Delta::Overcharge { healer_id, target_robot_id };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
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
    /// * GameError::NoSuchUnit - a unit does not exist.
    /// * GameError::TeamNotAllowed - either unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the robot or structure are the wrong type.
    /// * GameError::InvalidAction - the robot cannot be loaded inside the structure.
    pub fn load(&mut self, structure_id: UnitID, robot_id: UnitID)
                    -> Result<(), Error> {
        let delta = Delta::Load { structure_id, robot_id };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
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
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a structure.
    /// * GameError::InvalidLocation - the location is off the map.
    /// * GameError::InvalidAction - the rocket cannot degarrison a unit.
    pub fn unload(&mut self, structure_id: UnitID, direction: Direction)
                      -> Result<(), Error> {
        let delta = Delta::Unload { structure_id, direction };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
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
    /// * GameError::NoSuchUnit - the unit does not exist.
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a factory, or the
    ///   queued unit type is not a robot.
    /// * GameError::InvalidAction - the factory cannot produce the robot.
    pub fn produce_robot(&mut self, factory_id: UnitID, robot_type: UnitType)
                       -> Result<(), Error> {
        let delta = Delta::ProduceRobot { factory_id, robot_type };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
    }

    // ************************************************************************
    // *************************** ROCKET METHODS *****************************
    // ************************************************************************

    /// The landing rounds and locations of rockets in space that belong to the
    /// current team.
    ///
    /// Note that mutating this object does NOT have any effect on the actual
    /// game. You MUST call the mutators in world!!
    pub fn rocket_landings(&self) -> RocketLandingInfo {
        self.world.rocket_landings()
    }

    /// Whether the rocket can launch into space. The rocket can launch if the
    /// it has never been used before.
    pub fn can_launch_rocket(&self, rocket_id: UnitID, destination: MapLocation) -> bool {
        self.world.can_launch_rocket(rocket_id, destination)
    }

    /// Launches the rocket into space. If the destination is not on the map of
    /// the other planet, the rocket flies off, never to be seen again.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a rocket.
    /// * GameError::InvalidAction - the rocket cannot launch.
    pub fn launch_rocket(&mut self, rocket_id: UnitID, location: MapLocation)
                         -> Result<(), Error> {
        let delta = Delta::LaunchRocket { rocket_id, location };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
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
            turn: TurnMessage { changes: vec![] }
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
    pub fn apply_turn(&mut self, turn: &TurnMessage) -> Result<TurnApplication, Error> {
        // Serialize the filtered game state to send to the player
        let start_turn = self.world.apply_turn(&turn)?;
        // Serialize the game state to send to the viewer
        let viewer = ViewerMessage { world: self.world.clone() };
        Ok(TurnApplication {
            start_turn, viewer
        })
    }    
    
    /// Determines if the game has ended, returning the winning team if so.
    pub fn is_game_over(&self) -> Option<Team> {
        self.world.is_game_over()
    }
}

/// Returned from apply_turn.
/// This struct only exists because the bindings don't do tuples yet.
#[derive(Debug, Clone)]
pub struct TurnApplication {
    pub start_turn: StartTurnMessage,
    pub viewer: ViewerMessage
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

        // Test that red can move as expected.
        assert![!player_controller_red.can_move(red_robot, Direction::East)];
        assert![player_controller_red.can_move(red_robot, Direction::Northeast)];
        assert![player_controller_red.move_robot(red_robot, Direction::Northeast).is_ok()];

        // End red's turn, and pass the message to the manager, which
        // generates blue's start turn message and starts blue's turn.
        let red_turn_msg = player_controller_red.end_turn().unwrap();
        let application = manager_controller.apply_turn(&red_turn_msg).unwrap();
        let blue_start_turn_msg = application.start_turn;
        assert![player_controller_blue.start_turn(&blue_start_turn_msg).is_ok()];

        // Test that blue can move as expected. This demonstrates
        // it has received red's actions in its own state.
        assert![!player_controller_blue.can_move(blue_robot, Direction::North)];
        assert![player_controller_blue.can_move(blue_robot, Direction::West)];
        assert![player_controller_blue.move_robot(blue_robot, Direction::West).is_ok()];
    }
}