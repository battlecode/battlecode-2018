//! The outermost layer of the engine stack. Responsible for exposing
//! the API that the player will use, and for generating messages to
//! send to other parts of the Battlecode infrastructure.

use config::Config;
use location::*;
use map::*;
use schema::*;
use unit::*;
use world::*;

use failure::Error;
use fnv::FnvHashMap;

struct GameController {
    world: GameWorld,
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
    pub fn new() -> GameController {
        GameController {
            // TODO: load an actual map.
            world: GameWorld::test_world(),
            config: Config::player_config(),
            turn: TurnMessage { changes: vec![] }
        }
    }

    /// Starts the current turn, by updating the player's GameWorld with changes
    /// made since the last time the player had a turn.
    pub fn start_turn(&mut self, turn: StartTurnMessage) -> Result<(), Error> {
        self.world = turn.world;
        self.turn = TurnMessage { changes: vec![] };
        Ok(())
    }

    /// Ends the current turn. Returns the list of changes made in this turn.
    pub fn end_turn(&mut self) -> Result<TurnMessage, Error> {
        self.world.end_turn()?;
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

    /// The rockets in space that belong to the current team, including
    /// their landing rounds and locations, by landing round.
    pub fn rockets_in_space(&self) -> FnvHashMap<Rounds, Vec<Unit>> {
        self.world.rockets_in_space()
    }

    /// The starting map of the given planet. Includes the map's planet,
    /// dimensions, impassable terrain, and initial units and karbonite.
    pub fn starting_map(&self, planet: Planet) -> PlanetMap {
        self.world.starting_map(planet)
    }

    /// The karbonite in the team's resource pool.
    pub fn karbonite(&self) -> u32 {
        self.world.karbonite()
    }

    // ************************************************************************
    // ************************** SENSING METHODS *****************************
    // ************************************************************************

    /// All the units within the vision range.
    pub fn units(&self) -> Vec<Unit> {
        self.world.units()
    }

    /// All the units within the vision range, by ID.
    pub fn units_by_id(&self) -> FnvHashMap<UnitID, UnitInfo> {
        self.world.units_by_id()
    }

    /// All the units within the vision range, by location.
    pub fn units_by_loc(&self) -> FnvHashMap<MapLocation, UnitID> {
        self.world.units_by_loc()
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
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a robot.
    pub fn can_move(&self, robot_id: UnitID, direction: Direction) -> Result<bool, Error> {
        Ok(self.world.can_move(robot_id, direction)?)
    }

    /// Whether the robot is ready to move. Tests whether the robot's attack
    /// heat is sufficiently low.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a robot.
    pub fn is_move_ready(&self, robot_id: UnitID) -> Result<bool, Error> {
        Ok(self.world.is_move_ready(robot_id)?)
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
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is a healer, or not a robot.
    pub fn can_attack(&self, robot_id: UnitID, target_unit_id: UnitID) -> Result<bool, Error> {
        Ok(self.world.can_attack(robot_id, target_unit_id)?)
    }

    /// Whether the robot is ready to attack. Tests whether the robot's attack
    /// heat is sufficiently low.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is a healer, or not a robot.
    pub fn is_attack_ready(&self, robot_id: UnitID) -> Result<bool, Error> {
        Ok(self.world.is_attack_ready(robot_id)?)
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

    /*
    /// Returns research info of the current player.
    fn get_research(&self) -> ResearchInfo {
        let team = self.team();
        self.get_team_info(team).research.clone()
    }

    /// Returns mutable research info of the current player.
    fn get_research_mut(&mut self) -> &mut ResearchInfo {
        let team = self.team();
        &mut self.get_team_info_mut(team).research
    }

    /// The research info of the current team, including what branch is
    /// currently being researched, the number of rounds left.
    ///
    /// Note that mutating this object by resetting or queueing research
    /// does not have any effect. You must call the mutators on world.
    pub fn research_info(&self) -> ResearchInfo {
        self.get_research()
    }

    /// Resets the research queue to be empty. Returns true if the queue was
    /// not empty before, and false otherwise.
    pub fn reset_research(&mut self) -> bool {
        self.get_research_mut().reset_queue()
    }

    /// Adds a branch to the back of the queue, if it is a valid upgrade, and
    /// starts research if it is the first in the queue.
    ///
    /// Returns whether the branch was successfully added.
    pub fn queue_research(&mut self, branch: &Branch) -> bool {
        self.get_research_mut().add_to_queue(branch)
    }

    /// Update the current research and process any completed upgrades.
    fn process_research(&mut self, team: Team) -> Result<(), Error> {
        if let Some(branch) = self.get_team_info_mut(team).research.next_round()? {
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
    */

    // ************************************************************************
    // *************************** WORKER METHODS *****************************
    // ************************************************************************

    /// Whether the worker is ready to harvest. The worker cannot already have
    /// performed an action this round.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a worker.
    pub fn can_harvest(&self, worker_id: UnitID) -> Result<bool, Error> {
        Ok(self.world.can_harvest(worker_id)?)
    }

    /// Harvests up to the worker's harvest amount of karbonite from the given
    /// location, adding it to the team's resource pool.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a worker.
    /// * GameError::InvalidLocation - the location is off the map.
    /// * GameError::InvalidAction - the worker is not ready to harvest.
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
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a worker, or the
    ///   unit type is not a factory or rocket.
    pub fn can_blueprint(&self, worker_id: UnitID, unit_type: UnitType)
                         -> Result<bool, Error> {
        Ok(self.world.can_blueprint(worker_id, unit_type)?)
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
    ///
    /// * GameError::NoSuchUnit - a unit does not exist.
    /// * GameError::TeamNotAllowed - a unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the worker or blueprint is the wrong
    ///   type. A unit that has already been built is no longer a blueprint.
    pub fn can_build(&self, worker_id: UnitID, blueprint_id: UnitID)
                     -> Result<bool, Error> {
        Ok(self.world.can_build(worker_id, blueprint_id)?)
    }

    /// Blueprints a unit of the given type in the given direction. Subtract
    /// cost of that unit from the team's resource pool.
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

    /// Whether the worker is ready to replicate. Tests that the worker's
    /// ability heat is sufficiently low, and that the team has sufficient
    /// karbonite in its resource pool.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a worker.
    pub fn can_replicate(&self, worker_id: UnitID) -> Result<bool, Error> {
        Ok(self.world.can_replicate(worker_id)?)
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
    ///
    /// * GameError::InvalidResearchLevel - the ability has not been researched.
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a knight.
    pub fn can_javelin(&self, knight_id: UnitID, target_unit_id: UnitID) -> Result<bool, Error> {
        Ok(self.world.can_javelin(knight_id, target_unit_id)?)
    }

    /// Whether the knight is ready to javelin. Tests whether the knight's
    /// ability heat is sufficiently low.
    ///
    /// * GameError::InvalidResearchLevel - the ability has not been researched.
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a knight.
    pub fn is_javelin_ready(&self, knight_id: UnitID) -> Result<bool, Error> {
        Ok(self.world.is_javelin_ready(knight_id)?)
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
    ///
    /// * GameError::InvalidResearchLevel - the ability has not been researched.
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a mage.
    /// * GameError::InvalidLocation - the location is outside the vision range.
    pub fn can_blink(&self, mage_id: UnitID, location: MapLocation) -> Result<bool, Error> {
        Ok(self.world.can_blink(mage_id, location)?)
    }

    /// Whether the mage is ready to blink. Tests whether the mage's ability
    /// heat is sufficiently low.
    ///
    /// * GameError::InvalidResearchLevel - the ability has not been researched.
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a mage.
    pub fn is_blink_ready(&self, mage_id: UnitID) -> Result<bool, Error> {
        Ok(self.world.is_blink_ready(mage_id)?)
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
    ///
    /// * GameError::NoSuchUnit - a unit does not exist.
    /// * GameError::TeamNotAllowed - the first unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the healer or robot is not the right type.
    pub fn can_heal(&self, healer_id: UnitID, target_robot_id: UnitID) -> Result<bool, Error> {
        Ok(self.world.can_heal(healer_id, target_robot_id)?)
    }

    /// Whether the healer is ready to heal. Tests whether the healer's attack
    /// heat is sufficiently low.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a healer.
    pub fn is_heal_ready(&self, healer_id: UnitID) -> Result<bool, Error> {
        Ok(self.world.is_heal_ready(healer_id)?)
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
    ///
    /// * GameError::InvalidResearchLevel - the ability has not been researched.
    /// * GameError::NoSuchUnit - a unit does not exist.
    /// * GameError::TeamNotAllowed - the first unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the healer or robot is not the right type.
    pub fn can_overcharge(&self, healer_id: UnitID, target_robot_id: UnitID)
                          -> Result<bool, Error> {
        Ok(self.world.can_overcharge(healer_id, target_robot_id)?)
    }

    /// Whether the healer is ready to overcharge. Tests whether the healer's
    /// ability heat is sufficiently low.
    ///
    /// * GameError::InvalidResearchLevel - the ability has not been researched.
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a healer.
    pub fn is_overcharge_ready(&self, healer_id: UnitID) -> Result<bool, Error> {
        Ok(self.world.is_overcharge_ready(healer_id)?)
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
    ///
    /// * GameError::NoSuchUnit - a unit does not exist.
    /// * GameError::TeamNotAllowed - either unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the robot or structure are the wrong type.
    pub fn can_load(&self, structure_id: UnitID, robot_id: UnitID)
                        -> Result<bool, Error> {
        Ok(self.world.can_load(structure_id, robot_id)?)
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
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a structure.
    /// * GameError::InvalidLocation - the location is off the map.
    pub fn can_unload(&self, structure_id: UnitID, direction: Direction)
                                 -> Result<bool, Error> {
        Ok(self.world.can_unload(structure_id, direction)?)
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

    /*
    /// Adds a unit to the factory's production queue. Does nothing if the
    /// production queue is full. Returns whether the unit was added.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist.
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a factory, or the
    ///   queued unit type is not a robot.
    pub fn queue_robot(&mut self, _factory_id: UnitID, _unit_type: UnitType)
                       -> Result<bool, Error> {
        unimplemented!();
    }

    /// Process the end of the turn for factories. If a factory added a unit
    /// to its garrison, also mark that unit down in the game world.
    fn _process_factory(&self) {
        unimplemented!()
    }
    */

    // ************************************************************************
    // *************************** ROCKET METHODS *****************************
    // ************************************************************************

    /// Whether the rocket can launch into space. The rocket can launch if the
    /// it has never been used before.
    ///
    /// * GameError::NoSuchUnit - the unit does not exist (inside the vision range).
    /// * GameError::TeamNotAllowed - the unit is not on the current player's team.
    /// * GameError::InappropriateUnitType - the unit is not a rocket.
    pub fn can_launch_rocket(&self, rocket_id: UnitID, destination: MapLocation)
                             -> Result<bool, Error> {
        Ok(self.world.can_launch_rocket(rocket_id, destination)?)
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
    pub fn new_manager() -> GameController {
        GameController {
            // TODO: load an actual map.
            world: GameWorld::test_world(),
            config: Config::runner_config(),
            turn: TurnMessage { changes: vec![] }
        }
    }

    /// Given a TurnMessage from a player, apply those changes.
    pub fn apply_turn(&mut self, turn: TurnMessage) -> Result<(StartTurnMessage, ViewerMessage), Error> {
        self.world.apply_turn(&turn)?;
        // Serialize the filtered game state to send to the player
        // TODO: filter the world somehow
        let start_turn_message = StartTurnMessage { world: self.world.clone() };
        // Serialize the game state to send to the viewer
        let viewer_message = ViewerMessage { world: self.world.clone() };
        Ok((start_turn_message, viewer_message))
    }
}

mod tests {
    use super::GameController;
    use location::*;
    use schema::*;
    use unit::*;
    use world::*;

    #[test]
    fn test_turn() {
        // NOTE: This test depends on movement working properly.
        
        // Create controllers for the manager and two Earth players.
        let mut manager_controller = GameController::new_manager();
        let mut player_controller_red = GameController::new();
        let mut player_controller_blue = GameController::new();
        
        // Place some robots manually, for sake of testing.
        let red_robot = manager_controller.world.create_unit(
                                                        Team::Red, 
                                                        MapLocation::new(Planet::Earth, 0, 0), 
                                                        UnitType::Knight).unwrap();
        let blue_robot = manager_controller.world.create_unit(
                                                        Team::Blue, 
                                                        MapLocation::new(Planet::Earth, 1, 0), 
                                                        UnitType::Knight).unwrap();
        
        // Manually create the first start turn message, so the first
        // player can see the new robots, then start red's turn.
        let red_start_turn_msg = StartTurnMessage { world: manager_controller.world.clone() };
        assert![player_controller_red.start_turn(red_start_turn_msg).is_ok()];

        // Test that red can move as expected.
        assert![!player_controller_red.can_move(red_robot, Direction::East).unwrap()];
        assert![player_controller_red.can_move(red_robot, Direction::Northeast).unwrap()];
        assert![player_controller_red.move_robot(red_robot, Direction::Northeast).is_ok()];

        // End red's turn, and pass the message to the manager, which
        // generates blue's start turn message and starts blue's turn.
        let red_turn_msg = player_controller_red.end_turn().unwrap();
        let (blue_start_turn_msg, _) = manager_controller.apply_turn(red_turn_msg).unwrap();
        assert![player_controller_blue.start_turn(blue_start_turn_msg).is_ok()];

        // Test that blue can move as expected. This demonstrates
        // it has received red's actions in its own state.
        assert![!player_controller_blue.can_move(blue_robot, Direction::North).unwrap()];
        assert![player_controller_blue.can_move(blue_robot, Direction::West).unwrap()];
        assert![player_controller_blue.move_robot(blue_robot, Direction::West).is_ok()];
    }
}