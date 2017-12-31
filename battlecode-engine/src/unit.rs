//! Units are player-controlled entities that can perform certain
//! game actions, depending on their type.

use failure::Error;
use std::cmp;

use super::constants::*;
use super::error::GameError;
use super::location::*;
use super::research::Level;
use super::world::Team;
use unit::UnitType::*;

/// Percentage.
pub type Percent = u32;

/// The ID of an unit is assigned when the unit is spawned.
pub type UnitID = u32;

/// The public version of the unit. Contains all the unit's stats but none of
/// the action. The other team can see everything in the unit info.
pub struct UnitInfo {
    /// The unique ID of the unit.
    pub id: UnitID,
    /// The team the unit is on.
    pub team: Team,
    /// The type of the unit.
    pub unit_type: UnitType,
    /// The current location of the unit.
    pub location: Option<MapLocation>,
    /// The current health of the unit.
    pub health: u32,
}

/// The different unit types, which include factories, rockets, and the robots.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UnitType {
    /// Workers are the foundation of the civilization.
    Worker,
    /// Knights are a melee unit that is strong in numbers.
    Knight,
    /// Rangers are a ranged unit with good all-around combat.
    Ranger,
    /// Mages are a fragile but specialized ranged unit for large areas.
    Mage,
    /// Healers are a suport unit that can heal other units.
    Healer,
    /// Factories are the hub for producing combative robots.
    Factory,
    /// Rockets are the only unit that can move between planets.
    Rocket,
}

impl UnitType {
    /// List all the unit types.
    pub fn all() -> Vec<UnitType> {
        vec![
            Worker,
            Knight,
            Ranger,
            Mage,
            Healer,
            Factory,
            Rocket,
        ]
    }

    /// Return the default stats of the given unit type.
    fn default(&self) -> Unit {
        match *self {
            Worker => Unit {
                unit_type: Worker,
                health: 100,
                max_health: 100,
                damage: 0,
                attack_range: 0,
                vision_range: 50,
                movement_cooldown: 20,
                attack_cooldown: 0,
                ..Default::default()
            },
            Knight => Unit {
                unit_type: Knight,
                health: 250,
                max_health: 250,
                damage: 100,
                attack_range: 1,
                vision_range: 50,
                movement_cooldown: 15,
                attack_cooldown: 20,
                ability_cooldown: 75,
                ability_range: 10,
                ..Default::default()
            },
            Ranger => Unit {
                unit_type: Ranger,
                health: 200,
                max_health: 200,
                damage: 70,
                attack_range: 50,
                vision_range: 70,
                movement_cooldown: 20,
                attack_cooldown: 20,
                ability_cooldown: 150,
                ability_range: 10,
                ..Default::default()
            },
            Mage => Unit {
                unit_type: Mage,
                health: 100,
                max_health: 100,
                damage: 150,
                attack_range: 30,
                vision_range: 30,
                movement_cooldown: 20,
                attack_cooldown: 20,
                ability_cooldown: 100,
                ability_range: 5,
                ..Default::default()
            },
            Healer => Unit {
                unit_type: Healer,
                health: 100,
                max_health: 100,
                damage: -10,
                attack_range: 30,
                vision_range: 50,
                movement_cooldown: 25,
                attack_cooldown: 10,
                ability_cooldown: 50,
                ability_range: 30,
                ..Default::default()
            },
            Factory => Unit {
                unit_type: Factory,
                health: 1000 / 4,
                max_health: 1000,
                ..Default::default()
            },
            Rocket => Unit {
                unit_type: Rocket,
                health: 200 / 4,
                max_health: 200,
                ..Default::default()
            },
        }
    }
}

/// A single unit in the game and its controller. Actions can be performed on
/// this unit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Unit {
    // All units.
    id: UnitID,
    team: Team,
    level: Level,
    unit_type: UnitType,
    location: Option<MapLocation>,
    health: u32,
    movement_heat: u32,
    attack_heat: u32,
    max_health: u32,

    // All robots.
    damage: i32,
    attack_range: u32,
    vision_range: u32,
    movement_cooldown: u32,
    attack_cooldown: u32,

    /// The special ability for each non-worker robot:
    /// Javelin, Snipe, Blink, Overcharge.
    is_ability_unlocked: bool,
    ability_heat: u32,
    ability_cooldown: u32,
    ability_range: u32,

    // Factories and rockets.
    is_built: bool,

    // Worker special ability.
    build_health: u32,
    harvest_amount: u32,
    has_harvested: bool,

    // Knight special ability.
    defense_per_robot: Percent,

    // Ranger special ability.
    cannot_attack_range: u32,
    countdown: u32,
    target_location: Option<MapLocation>,

    // Mage special ability.
    explode_multiplier: Percent,

    // Healer special ability.
    self_heal_amount: u32,

    // Factory special ability.
    production_queue: Vec<UnitType>,

    // Rocket special ability.
    is_used: bool,
    max_capacity: usize,
    travel_time_multiplier: Percent,
    garrisoned_units: Vec<UnitID>,
}

impl Default for Unit {
    fn default() -> Unit {
        Unit {
            id: 0,
            health: 0,
            location: None,
            max_health: 0,
            team: Team::Red,
            unit_type: Worker,

            level: 0,
            movement_heat: 0,
            attack_heat: 0,
            damage: 0,
            attack_range: 0,
            vision_range: 0,
            movement_cooldown: 0,
            attack_cooldown: 0,

            is_ability_unlocked: false,
            ability_heat: 0,
            ability_cooldown: 0,
            ability_range: 0,

            is_built: false,
            build_health: 5,
            harvest_amount: 3,
            has_harvested: false,
            defense_per_robot: 1,
            cannot_attack_range: 10,
            countdown: 0,
            target_location: None,
            explode_multiplier: 100,
            self_heal_amount: 1,
            production_queue: vec![],
            is_used: false,
            max_capacity: 8,
            travel_time_multiplier: 100,
            garrisoned_units: vec![],
        }
    }
}

impl Unit {
    /// Create a new unit of the given type.
    pub fn new(id: UnitID,
               team: Team,
               unit_type: UnitType,
               level: Level,
               location: MapLocation) -> Result<Unit, Error> {
        let mut unit = unit_type.default();
        unit.id = id;
        unit.team = team;
        unit.location = Some(location);

        for _ in 0..level {
            unit.research()?;
        }
        Ok(unit)
    }

    /// The public version of the unit.
    pub fn info(&self) -> UnitInfo {
        UnitInfo {
            id: self.id,
            team: self.team,
            unit_type: self.unit_type,
            location: self.location,
            health: self.health,
        }
    }

    // ************************************************************************
    // ***************************** GENERAL METHODS **************************
    // ************************************************************************

    /// The unique ID of a unit.
    pub fn id(&self) -> UnitID {
        self.id
    }

    /// The team the unit belongs to.
    pub fn team(&self) -> Team {
        self.team
    }

    /// The unit type.
    pub fn unit_type(&self) -> UnitType {
        self.unit_type
    }

    // ************************************************************************
    // *************************** HELPER METHODS *****************************
    // ************************************************************************

    /// Ok if the unit is a robot. Errors otherwise.
    fn ok_if_robot(&self) -> Result<(), Error> {
        match self.unit_type {
            Worker => Ok(()),
            Knight => Ok(()),
            Ranger => Ok(()),
            Mage   => Ok(()),
            Healer => Ok(()),
            _ => Err(GameError::InappropriateUnitType)?,
        }
    }

    /// Ok if the unit is the given type. Errors otherwise.
    fn ok_if_unit_type(&self, unit_type: UnitType) -> Result<(), Error> {
        if self.unit_type == unit_type {
            Ok(())
        } else {
            Err(GameError::InappropriateUnitType)?
        }
    }

    // ************************************************************************
    // ************************** MOVEMENT METHODS ****************************
    // ************************************************************************

    /// The location of the unit. None if the unit is traveling or in a rocket.
    pub fn location(&self) -> Option<MapLocation> {
        self.location
    }

    /// The movement heat.
    ///
    /// Errors if the unit is not a robot.
    pub fn movement_heat(&self) -> Result<u32, Error> {
        self.ok_if_robot()?;
        Ok(self.movement_heat)
    }

    /// The movement cooldown.
    ///
    /// Errors if the unit is not a robot.
    pub fn movement_cooldown(&self) -> Result<u32, Error> {
        self.ok_if_robot()?;
        Ok(self.movement_cooldown)
    }

    /// Whether the unit is ready to move. The movement heat must be lower than
    /// the maximum heat to attack.
    ///
    /// Errors if the unit is not a robot.
    pub fn is_move_ready(&self) -> Result<bool, Error> {
        Ok(self.movement_heat()? < MAX_HEAT_TO_ACT)
    }

    /// Updates the unit's location as it if has moved, and increases the
    /// movement heat.
    /// 
    /// Errors if the unit is not a robot, or not ready to move.
    pub fn move_to(&mut self, location: Option<MapLocation>)
                   -> Result<(), Error> {
        if self.is_move_ready()? {
            self.movement_heat += self.movement_cooldown;
            self.location = location;
            Ok(())
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    /// Whether the unit is adjacent to the location.
    pub fn is_adjacent_to(&self, location: Option<MapLocation>) -> bool {
        let loc_a = match self.location() {
            Some(loc) => loc,
            None => { return false; },
        };
        let loc_b = match location {
            Some(loc) => loc,
            None => { return false; },
        };
        loc_a.is_adjacent_to(loc_b)
    }

    // ************************************************************************
    // *************************** COMBAT METHODS *****************************
    // ************************************************************************

    /// The current health.
    pub fn health(&self) -> u32 {
        self.health
    }

    /// The maximum health.
    pub fn max_health(&self) -> u32 {
        self.max_health
    }

    /// The attack heat.
    ///
    /// Errors if the unit is not a robot.
    pub fn attack_heat(&self) -> Result<u32, Error> {
        self.ok_if_robot()?;
        Ok(self.attack_heat)
    }

    /// The attack cooldown.
    ///
    /// Errors if the unit is not a robot.
    pub fn attack_cooldown(&self) -> Result<u32, Error> {
        self.ok_if_robot()?;
        Ok(self.attack_cooldown)
    }

    /// Whether the unit is ready to attack. The attack heat must be lower than
    /// the maximum heat to act.
    ///
    /// Errors if the unit is not a robot.
    pub fn is_attack_ready(&self) -> Result<bool, Error> {
        Ok(self.attack_heat()? < MAX_HEAT_TO_ACT)
    }

    /// The damage inflicted by the robot during a normal attack.
    ///
    /// Errors if the unit is not a robot.
    pub fn damage(&self) -> Result<i32, Error> {
        self.ok_if_robot()?;
        Ok(self.damage)
    }

    /// Updates the unit as if it has attacked, and increases the attack heat.
    /// Returns the damage done.
    ///
    /// Errors if the unit is not a robot, or not ready to attack.
    pub fn attack(&mut self) -> Result<i32, Error> {
        if self.is_attack_ready()? {
            self.attack_heat += self.attack_cooldown;
            Ok(self.damage)
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    /// Take the amount of damage given, returning true if the unit has died.
    /// Returns false if the unit is still alive.
    pub fn take_damage(&mut self, damage: i32) -> bool {
        // TODO: Knight damage resistance??
        self.health -= cmp::min(damage, self.health as i32) as u32;
        self.health == 0
    }

    /// Destroys the unit. Equivalent to removing it from the game.
    pub fn destroy(&mut self) {
        self.location = None;
    }

    // ************************************************************************
    // *************************** WORKER METHODS *****************************
    // ************************************************************************

    /// The health restored when building or repairing a factory or rocket.
    ///
    /// Errors if the unit is not a worker.
    pub fn build_health(&self) -> Result<u32, Error> {
        self.ok_if_unit_type(Worker)?;
        Ok(self.build_health)
    }

    /// The maximum amount of karbonite harvested from a deposit in one turn.
    ///
    /// Errors if the unit is not a worker.
    pub fn harvest_amount(&self) -> Result<u32, Error> {
        self.ok_if_unit_type(Worker)?;
        Ok(self.harvest_amount)
    }

    /// Whether the unit can harvest.
    ///
    /// Errors if the unit is not a worker.
    pub fn can_harvest(&self) -> Result<bool, Error> {
        self.ok_if_unit_type(Worker)?;
        Ok(!self.has_harvested)
    }

    /// Updates the unit as if it has harvested from a location.
    ///
    /// Errors if the unit is not a worker, or not ready to harvest.
    pub fn harvest(&mut self) -> Result<(), Error> {
        if self.can_harvest()? {
            self.has_harvested = true;
            Ok(())
        } else {
            Err(GameError::InvalidAction)?
        }
    }

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

    /// The max capacity of a rocket.
    ///
    /// Errors if the unit is not a rocket.
    pub fn max_capacity(&self) -> Result<usize, Error> {
        self.ok_if_unit_type(Rocket)?;
        Ok(self.max_capacity)
    }

    /// Whether the rocket has already been used.
    ///
    /// Errors if the unit is not a rocket.
    pub fn is_rocket_used(&self) -> Result<bool, Error> {
        self.ok_if_unit_type(Rocket)?;
        Ok(self.is_used)
    }

    /// Returns the garrisoned units in a rocket.
    ///
    /// Errors if the unit is not a rocket.
    pub fn garrisoned_units(&self) -> Result<Vec<UnitID>, Error> {
        self.ok_if_unit_type(Rocket)?;
        Ok(self.garrisoned_units.clone())
    }

    /// Whether the rocket can garrison a unit. The unit must be ready to move
    /// and adjacent to the rocket. The rocket must have enough space.
    ///
    /// Errors if the unit is not a rocket.
    pub fn can_garrison(&self, robot: &Unit) -> Result<bool, Error> {
        Ok(robot.is_move_ready()?
            && self.garrisoned_units()?.len() < self.max_capacity()?
            && self.team == robot.team
            && self.is_adjacent_to(robot.location()))
    }

    /// Updates the rocket as if it has garrisoned a unit inside the rocket.
    /// Adds the unit ID to the garrison.
    ///
    /// Errors if the unit is not a rocket, or it cannot garrison.
    pub fn garrison(&mut self, id: UnitID) -> Result<(), Error> {
        if self.garrisoned_units()?.len() < self.max_capacity()? {
            self.ok_if_unit_type(Rocket)?;
            self.garrisoned_units.push(id);
            Ok(())
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    /// Whether the rocket can launch. It must not be used and it must
    /// currently be on a planet.
    ///
    /// Errors if the unit is not a rocket.
    pub fn can_launch_rocket(&mut self) -> Result<bool, Error> {
        Ok(!self.is_rocket_used()? && self.location != None)
    }

    /// Updates the rocket as if it has launched by changing its location and
    /// marking it as used.
    ///
    /// Errors if the unit is not a rocket.
    pub fn launch_rocket(&mut self) -> Result<(), Error> {
        if self.can_launch_rocket()? {
            self.location = None;
            self.is_used = true;
            Ok(())
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    /// Updates the rocket's location as if it has landed.
    ///
    /// Errors if the unit is not a rocket, or if it cannot be landed.
    pub fn land_rocket(&mut self, location: MapLocation) -> Result<(), Error> {
        if self.location == None {
            self.ok_if_unit_type(Rocket)?;
            self.location = Some(location);
            Ok(())
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    /// Whether the rocket can degarrison a unit. The rocket must be on a
    /// planet and it must have at least one unit to degarrison. Does not check
    /// whether the unit is ready to move.
    ///
    /// Errors if the unit is not a rocket.
    pub fn can_degarrison_unit(&self) -> Result<bool, Error> {
        Ok(self.location().is_some() && self.garrisoned_units()?.len() > 0)
    }

    /// Updates the rocket as if it has degarrisoned a single unit from the
    /// rocket, returning the unit ID.
    ///
    /// Errors if the unit is not a rocket, or it cannot degarrison.
    pub fn degarrison_unit(&mut self) -> Result<UnitID, Error> {
        if self.can_degarrison_unit()? {
            Ok(self.garrisoned_units.remove(0))
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    // ************************************************************************
    // **************************** OTHER METHODS *****************************
    // ************************************************************************

    /// The current research level.
    pub fn research_level(&self) -> Level {
        self.level
    }

    /// Research the next level.
    pub fn research(&mut self) -> Result<(), Error> {
        match self.unit_type {
            Worker => match self.level {
                0 => { self.harvest_amount += 1; },
                1 => { self.build_health += 1; },
                2 => { self.build_health += 1; },
                3 => { self.build_health += 3; },
                _ => Err(GameError::InvalidResearchLevel)?,
            },
            Knight => match self.level {
                0 => { self.defense_per_robot += 1; },
                1 => { self.defense_per_robot += 1; },
                2 => { self.is_ability_unlocked = true; },
                _ => Err(GameError::InvalidResearchLevel)?,
            },
            Ranger => match self.level {
                0 => { self.movement_cooldown -= 5; },
                1 => { self.vision_range += 30; },
                2 => { self.is_ability_unlocked = true; },
                _ => Err(GameError::InvalidResearchLevel)?,
            },
            Mage => match self.level {
                0 => { self.damage += 15; },
                1 => { self.damage += 15; },
                2 => { self.explode_multiplier += 50; },
                3 => { self.is_ability_unlocked = true; },
                _ => Err(GameError::InvalidResearchLevel)?,
            },
            Healer => match self.level {
                0 => { self.damage -= 2; },
                1 => { self.damage -= 5; },
                2 => { self.is_ability_unlocked = true; },
                _ => Err(GameError::InvalidResearchLevel)?,
            },
            Rocket => match self.level {
                // TODO: rocket unlocking
                0 => { self.is_ability_unlocked = true; },
                1 => { self.travel_time_multiplier -= 20; },
                2 => { self.max_capacity += 4; },
                _ => Err(GameError::InvalidResearchLevel)?,
            },
            Factory => Err(GameError::InvalidResearchLevel)?,
        }
        self.level += 1;
        Ok(())
    }

    /// Process the end of the round.
    pub fn next_round(&mut self) {
        self.movement_heat -= cmp::min(HEAT_LOSS_PER_ROUND, self.movement_heat);
        self.attack_heat -= cmp::min(HEAT_LOSS_PER_ROUND, self.attack_heat);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movement() {
        let loc_a = MapLocation::new(Planet::Earth, 0, 0);
        let loc_b = MapLocation::new(Planet::Earth, 1, 1);
        let mut unit = Unit::new(1, Team::Red, Healer, 0, loc_a).unwrap();
        assert!(unit.location().is_some());
        assert!(unit.movement_cooldown().unwrap() > 0);
        assert!(unit.is_move_ready().unwrap());
        assert_eq!(unit.location(), Some(loc_a));
        assert_eq!(unit.movement_heat().unwrap(), 0);

        // Move to a location, and fail to move immediately after.
        assert!(unit.move_to(Some(loc_b)).is_ok());
        assert!(!unit.is_move_ready().unwrap());
        assert!(unit.move_to(Some(loc_a)).is_err());
        assert_eq!(unit.location(), Some(loc_b));

        // Wait one round, and fail to move again.
        unit.next_round();
        assert!(unit.movement_heat().unwrap() > MAX_HEAT_TO_ACT);
        assert!(!unit.is_move_ready().unwrap());
        assert!(unit.move_to(Some(loc_a)).is_err());
        assert_eq!(unit.location(), Some(loc_b));

        // Wait one more round, and succesfully move.
        unit.next_round();
        assert!(unit.movement_heat().unwrap() < MAX_HEAT_TO_ACT);
        assert!(unit.is_move_ready().unwrap());
        assert!(unit.move_to(Some(loc_a)).is_ok());
        assert_eq!(unit.location(), Some(loc_a));
    }

    #[test]
    fn test_is_adjacent_to() {
        let loc_a = MapLocation::new(Planet::Earth, 0, 0);
        let loc_b = MapLocation::new(Planet::Earth, 1, 1);
        let loc_c = MapLocation::new(Planet::Earth, 1, 2);

        let unit_a = Unit::new(1, Team::Red, Ranger, 0, loc_a).unwrap();
        let mut unit_b = Unit::new(2, Team::Red, Worker, 0, loc_b).unwrap();
        let mut unit_c = Unit::new(3, Team::Red, Mage, 0, loc_c).unwrap();

        // B is adjacent to both A and C, but A is not adjacent to C.
        assert!(unit_a.is_adjacent_to(unit_b.location()));
        assert!(unit_b.is_adjacent_to(unit_a.location()));
        assert!(unit_c.is_adjacent_to(unit_b.location()));
        assert!(unit_b.is_adjacent_to(unit_c.location()));
        assert!(!unit_a.is_adjacent_to(unit_c.location()));
        assert!(!unit_c.is_adjacent_to(unit_a.location()));

        // Nothing is adjacent to None.
        unit_b.move_to(None).unwrap();
        unit_c.move_to(None).unwrap();
        assert!(!unit_a.is_adjacent_to(unit_b.location()));
        assert!(!unit_b.is_adjacent_to(unit_a.location()));
        assert!(!unit_b.is_adjacent_to(unit_c.location()));
    }

    #[test]
    fn test_movement_error() {
        let loc = MapLocation::new(Planet::Earth, 0, 0);
        let adjacent_loc = MapLocation::new(Planet::Earth, 1, 0);

        let mut factory = Unit::new(1, Team::Red, Factory, 0, loc).unwrap();
        assert!(factory.movement_heat().is_err());
        assert!(factory.movement_cooldown().is_err());
        assert!(factory.is_move_ready().is_err());
        assert!(factory.move_to(Some(adjacent_loc)).is_err());

        let mut rocket = Unit::new(1, Team::Red, Rocket, 0, loc).unwrap();
        assert!(rocket.movement_heat().is_err());
        assert!(rocket.movement_cooldown().is_err());
        assert!(rocket.is_move_ready().is_err());
        assert!(rocket.move_to(Some(adjacent_loc)).is_err());
    }

    #[test]
    fn test_combat() {

    }

    #[test]
    fn test_rockets() {
        let loc = MapLocation::new(Planet::Earth, 0, 0);
        let adjacent_loc = loc.add(Direction::North);
        let mars_loc = MapLocation::new(Planet::Mars, 0, 0);
        let adjacent_mars_loc = mars_loc.add(Direction::North);

        let mut rocket = Unit::new(1, Team::Red, Rocket, 0, loc).unwrap();
        let mut robot = Unit::new(2, Team::Red, Mage, 0, adjacent_loc).unwrap();

        // Rocket accessor methods should fail on a robot.
        assert!(robot.max_capacity().is_err());
        assert!(robot.is_rocket_used().is_err());
        assert!(robot.garrisoned_units().is_err());
        assert!(robot.garrison(0).is_err());
        assert!(robot.can_launch_rocket().is_err());
        assert!(robot.launch_rocket().is_err());
        assert!(robot.land_rocket(loc).is_err());
        assert!(robot.can_degarrison_unit().is_err());
        assert!(robot.degarrison_unit().is_err());

        // Check accessor methods on the rocket.
        assert!(rocket.max_capacity().unwrap() > 0);
        assert!(!rocket.is_rocket_used().unwrap());
        assert_eq!(rocket.garrisoned_units().unwrap().len(), 0);
        assert!(rocket.can_garrison(&robot).unwrap());
        assert!(!rocket.can_degarrison_unit().unwrap());
        assert!(rocket.can_launch_rocket().unwrap());

        // The rocket cannot land.
        assert!(rocket.land_rocket(mars_loc).is_err());

        // Garrison a unit and launch into space.
        assert!(rocket.garrison(robot.id()).is_ok());
        robot.move_to(None).unwrap();
        assert_eq!(rocket.garrisoned_units().unwrap(), vec![robot.id()]);
        assert!(rocket.can_degarrison_unit().unwrap());
        assert_eq!(rocket.launch_rocket().unwrap(), ());
        assert_eq!(rocket.location(), None);
        assert!(rocket.is_rocket_used().unwrap());

        // Proceed a round, then land the rocket.
        robot.next_round();
        rocket.next_round();
        assert_eq!(rocket.land_rocket(mars_loc).unwrap(), ());
        assert_eq!(rocket.location(), Some(mars_loc));

        // Degarrison the unit.
        assert!(rocket.can_degarrison_unit().unwrap());
        assert_eq!(rocket.degarrison_unit().unwrap(), robot.id());
        assert!(!rocket.can_degarrison_unit().unwrap());

        // Garrison too many units
        let robot = Unit::new(0, Team::Red, Mage, 0, adjacent_mars_loc).unwrap();
        for i in 0..rocket.max_capacity().unwrap() {
            assert!(rocket.can_garrison(&robot).unwrap(), "failed to garrison unit {}", i);
            assert!(rocket.garrison(0).is_ok());
        }
        assert!(!rocket.can_garrison(&robot).unwrap());
        assert!(rocket.garrison(0).is_err());
    }

    #[test]
    fn test_research() {
        // Create a unit and check that its basic fields are correct.
        let loc = MapLocation::new(Planet::Earth, 0, 0);
        let mut unit_a = Unit::new(1, Team::Red, Worker, 0, loc).unwrap();
        assert_eq!(unit_a.id(), 1);
        assert_eq!(unit_a.team(), Team::Red);
        assert_eq!(unit_a.unit_type(), Worker);

        // Upgrade it twice and check its stats have been updated.
        assert_eq!(unit_a.research_level(), 0);
        assert_eq!(unit_a.harvest_amount().unwrap(), 3);
        assert_eq!(unit_a.build_health().unwrap(), 5);

        unit_a.research().unwrap();
        assert_eq!(unit_a.research_level(), 1);
        assert_eq!(unit_a.harvest_amount().unwrap(), 4);
        assert_eq!(unit_a.build_health().unwrap(), 5);

        unit_a.research().unwrap();
        assert_eq!(unit_a.research_level(), 2);
        assert_eq!(unit_a.harvest_amount().unwrap(), 4);
        assert_eq!(unit_a.build_health().unwrap(), 6);

        // Create a unit with a default level above 0, and check its stats.
        let unit_b = Unit::new(2, Team::Red, Worker, 2, loc).unwrap();
        assert_eq!(unit_b.research_level(), 2);
        assert_eq!(unit_b.harvest_amount().unwrap(), 4);
        assert_eq!(unit_b.build_health().unwrap(), 6);
    }
}
