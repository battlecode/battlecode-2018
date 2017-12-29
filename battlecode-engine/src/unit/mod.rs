//! Units are player-controlled entities that can perform certain
//! game actions, depending on their type.

use failure::Error;
use std::cmp;

use super::constants::*;
use super::error::GameError;
use super::location::*;
use super::research::Level;
use super::world::Team;
use unit::UnitController::*;

// Import each unit file into this namespace.
use self::worker::*;
mod worker;
use self::knight::*;
mod knight;
use self::ranger::*;
mod ranger;
use self::mage::*;
mod mage;
use self::healer::*;
mod healer;
use self::factory::*;
mod factory;
use self::rocket::*;
mod rocket;

/// Percentage.
pub type Percent = u32;

/// The ID of an unit is assigned when the unit is spawned.
pub type UnitID = u32;

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
            UnitType::Worker,
            UnitType::Knight,
            UnitType::Ranger,
            UnitType::Mage,
            UnitType::Healer,
            UnitType::Factory,
            UnitType::Rocket,
        ]
    }

    /// Return the default stats of the given unit type.
    fn default(&self) -> UnitController {
        match *self {
            UnitType::Worker => Worker(WorkerController::default()),
            UnitType::Knight => Knight(KnightController::default()),
            UnitType::Ranger => Ranger(RangerController::default()),
            UnitType::Mage => Mage(MageController::default()),
            UnitType::Healer => Healer(HealerController::default()),
            UnitType::Factory => Factory(FactoryController::default()),
            UnitType::Rocket => Rocket(RocketController::default()),
        }
    }
}

/// Implemented by all robot controllers. Robots include: Worker, Knight,
/// Ranger, Mage, Healer.
trait RobotController {
    /// The damage inflicted by the robot during a normal attack.
    fn damage(&self) -> i32;
    /// The distance squared, inclusive, of which a robot may attack.
    fn attack_range(&self) -> u32;
    /// The distance squared, inclusive, of which a robot may see.
    fn vision_range(&self) -> u32;
    /// The movement cooldown of the robot.
    fn movement_cooldown(&self) -> u32;
    /// The attack cooldown of the robot.
    fn attack_cooldown(&self) -> u32;
}

/// Units are player-controlled objects with certain characteristics and
/// game actions, depending on their type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
enum UnitController {
    /// Workers are the foundation of the civilization.
    Worker(WorkerController),
    /// Knights are a melee unit that is strong in numbers.
    Knight(KnightController),
    /// Rangers are a ranged unit with good all-around combat.
    Ranger(RangerController),
    /// Mages are a fragile but specialized ranged unit for large areas.
    Mage(MageController),
    /// Healers are a suport unit that can heal other units.
    Healer(HealerController),
    /// Factories are the hub for producing combative robots.
    Factory(FactoryController),
    /// Rockets are the only unit that can move between planets.
    Rocket(RocketController),
}

/// A single unit in the game.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Unit {
    id: UnitID,
    team: Team,
    location: Option<MapLocation>,
    health: u32,
    movement_heat: u32,
    attack_heat: u32,
    controller: UnitController,
}

impl Unit {
    /// Create a new unit of the given type.
    pub fn new(id: UnitID,
               team: Team,
               unit_type: UnitType,
               level: Level,
               location: MapLocation) -> Result<Unit, Error> {
        let controller = unit_type.default();
        let health = match controller {
            Worker(ref c) => c.max_health(),
            Knight(ref c) => c.max_health(),
            Ranger(ref c) => c.max_health(),
            Mage(ref c) => c.max_health(),
            Healer(ref c) => c.max_health(),
            Factory(ref c) => c.max_health() / 4,
            Rocket(ref c) => c.max_health() / 4,
        };

        let mut unit = Unit {
            id: id,
            team: team,
            location: Some(location),
            health: health,
            movement_heat: 0,
            attack_heat: 0,
            controller: controller,
        };

        for _ in 0..level {
            unit.research()?;
        }
        Ok(unit)
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
        match self.controller {
            Worker(_) => UnitType::Worker,
            Knight(_) => UnitType::Knight,
            Ranger(_) => UnitType::Ranger,
            Mage(_) => UnitType::Mage,
            Healer(_) => UnitType::Healer,
            Factory(_) => UnitType::Factory,
            Rocket(_) => UnitType::Rocket,
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
        match self.controller {
            Worker(_) => Ok(self.movement_heat),
            Knight(_) => Ok(self.movement_heat),
            Ranger(_) => Ok(self.movement_heat),
            Mage(_)   => Ok(self.movement_heat),
            Healer(_) => Ok(self.movement_heat),
            _ => Err(GameError::InappropriateUnitType)?,
        }
    }

    /// The movement cooldown.
    ///
    /// Errors if the unit is not a robot.
    pub fn movement_cooldown(&self) -> Result<u32, Error> {
        match self.controller {
            Worker(ref c) => Ok(c.movement_cooldown()),
            Knight(ref c) => Ok(c.movement_cooldown()),
            Ranger(ref c) => Ok(c.movement_cooldown()),
            Mage(ref c)   => Ok(c.movement_cooldown()),
            Healer(ref c) => Ok(c.movement_cooldown()),
            _ => Err(GameError::InappropriateUnitType)?,
        }
    }

    /// Whether the unit is ready to move.
    ///
    /// Errors if the unit is not a robot.
    pub fn is_move_ready(&self) -> Result<bool, Error> {
        Ok(self.movement_heat()? < MAX_HEAT_TO_ACT)
    }

    /// Moves the unit to this location.
    /// 
    /// Errors if the unit is not a robot, or not ready to move.
    pub fn move_to(&mut self, location: Option<MapLocation>)
                   -> Result<(), Error> {
        if self.is_move_ready()? {
            self.movement_heat += self.movement_cooldown()?;
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
        loc_a.adjacent_to(loc_b)
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
        match self.controller {
            Worker(ref c)  => c.max_health(),
            Knight(ref c)  => c.max_health(),
            Ranger(ref c)  => c.max_health(),
            Mage(ref c)    => c.max_health(),
            Healer(ref c)  => c.max_health(),
            Factory(ref c) => c.max_health(),
            Rocket(ref c)  => c.max_health(),
        }
    }

    /// The attack heat.
    ///
    /// Errors if the unit is not a robot.
    pub fn attack_heat(&self) -> Result<u32, Error> {
        match self.controller {
            Worker(_) => Ok(self.attack_heat),
            Knight(_) => Ok(self.attack_heat),
            Ranger(_) => Ok(self.attack_heat),
            Mage(_)   => Ok(self.attack_heat),
            Healer(_) => Ok(self.attack_heat),
            _ => Err(GameError::InappropriateUnitType)?,
        }
    }

    /// The attack cooldown.
    ///
    /// Errors if the unit is not a robot.
    pub fn attack_cooldown(&self) -> Result<u32, Error> {
        match self.controller {
            Worker(ref c) => Ok(c.attack_cooldown()),
            Knight(ref c) => Ok(c.attack_cooldown()),
            Ranger(ref c) => Ok(c.attack_cooldown()),
            Mage(ref c)   => Ok(c.attack_cooldown()),
            Healer(ref c) => Ok(c.attack_cooldown()),
            _ => Err(GameError::InappropriateUnitType)?,
        }
    }

    /// Whether the unit is ready to attack.
    ///
    /// Errors if the unit is not a robot.
    pub fn is_attack_ready(&self) -> Result<bool, Error> {
        Ok(self.attack_heat()? < MAX_HEAT_TO_ACT)
    }

    /// The damage inflicted by the robot during a normal attack.
    ///
    /// Errors if the unit is not a robot.
    pub fn damage(&self) -> Result<i32, Error> {
        match self.controller {
            Worker(ref c) => Ok(c.damage()),
            Knight(ref c) => Ok(c.damage()),
            Ranger(ref c) => Ok(c.damage()),
            Mage(ref c)   => Ok(c.damage()),
            Healer(ref c) => Ok(c.damage()),
            _ => Err(GameError::InappropriateUnitType)?,
        }
    }

    /// Updates as if the unit has attacked, and returns the damage done.
    ///
    /// Errors if the unit is not a robot, or not ready to attack.
    pub fn attack(&mut self) -> Result<i32, Error> {
        if self.is_move_ready()? {
            self.attack_heat += self.attack_cooldown()?;
            Ok(self.damage()?)
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
    // *************************** ROCKET METHODS *****************************
    // ************************************************************************

    /// The max capacity of a rocket.
    ///
    /// Errors if the unit is not a rocket.
    pub fn max_capacity(&self) -> Result<usize, Error> {
        match self.controller {
            Rocket(ref c) => Ok(c.max_capacity()),
            _ => Err(GameError::InappropriateUnitType)?,
        }
    }

    /// Whether the rocket has already been used.
    ///
    /// Errors if the unit is not a rocket.
    pub fn is_rocket_used(&self) -> Result<bool, Error> {
        match self.controller {
            Rocket(ref c) => Ok(c.used()),
            _ => Err(GameError::InappropriateUnitType)?,
        }
    }

    /// Marks the rocket as used.
    ///
    /// Errors if the unit is not a rocket.
    pub fn use_rocket(&mut self) -> Result<(), Error> {
        match self.controller {
            Rocket(ref mut c) => {
                c.mark_used();
                Ok(())
            },
            _ => Err(GameError::InappropriateUnitType)?,
        }
    }

    /// Returns the garrisoned units in a rocket.
    ///
    /// Errors if the unit is not a rocket.
    pub fn garrisoned_units(&self) -> Result<Vec<UnitID>, Error> {
        match self.controller {
            Rocket(ref c) => Ok(c.garrisoned_units()),
            _ => Err(GameError::InappropriateUnitType)?,
        }
    }

    /// Whether the unit can garrison inside the rocket. The unit must be ready
    /// to move and adjacent to the rocket. The rocket must have enough space.
    ///
    /// Errors if the rocket is the incorrect type.
    pub fn can_garrison(&self, rocket: &Unit) -> Result<bool, Error> {
        Ok(self.is_move_ready()?
            && rocket.garrisoned_units()?.len() < rocket.max_capacity()?
            && self.team == rocket.team
            && self.is_adjacent_to(rocket.location()))
    }

    /// Moves the unit inside the rocket.
    ///
    /// Errors if the unit is not a rocket.
    pub fn garrison(&mut self, id: UnitID) -> Result<(), Error> {
        match self.controller {
            Rocket(ref mut c) => {
                c.push_unit(id);
            },
            _ => Err(GameError::InappropriateUnitType)?,
        }
        Ok(())
    }

    /// Launches the rocket.
    ///
    /// Errors if the unit is not a rocket.
    pub fn launch_rocket(&mut self) -> Result<(), Error> {
        if self.unit_type() == UnitType::Rocket {
            self.location = None;
            Ok(())
        } else {
            Err(GameError::InappropriateUnitType)?
        }
    }

    /// Lands the rocket.
    ///
    /// Errors if the unit is not a rocket.
    pub fn land_rocket(&mut self, location: MapLocation) -> Result<(), Error> {
        if self.unit_type() == UnitType::Rocket {
            self.location = Some(location);
            Ok(())
        } else {
            Err(GameError::InappropriateUnitType)?
        }
    }

    /// Whether the rocket can degarrison a unit. The rocket must be on a
    /// planet and it must have at least one unit to degarrison.
    ///
    /// Errors if the unit is not a rocket.
    pub fn can_degarrison_unit(&self) -> Result<bool, Error> {
        match self.controller {
            Rocket(ref c) => {
                Ok(self.location().is_some() && c.garrisoned_units().len() > 0)
            },
            _ => Err(GameError::InappropriateUnitType)?
        }
    }

    /// Degarrisons a single unit from the rocket, and returns the unit ID.
    ///
    /// Errors if the unit is not a rocket or there are no units.
    pub fn degarrison_unit(&mut self) -> Result<UnitID, Error> {
        match self.controller {
            Rocket(ref mut c) => Ok(c.remove_first_unit()?),
            _ => Err(GameError::InappropriateUnitType)?,
        }
    }

    // ************************************************************************
    // **************************** OTHER METHODS *****************************
    // ************************************************************************

    /// Research the next level.
    pub fn research(&mut self) -> Result<(), Error> {
        match self.controller {
            Worker(ref mut c)  => c.research(),
            Knight(ref mut c)  => c.research(),
            Ranger(ref mut c)  => c.research(),
            Mage(ref mut c)    => c.research(),
            Healer(ref mut c)  => c.research(),
            Factory(ref mut c) => c.research(),
            Rocket(ref mut c)  => c.research(),
        }
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
    fn worker_constructor_and_research() {
        let loc = MapLocation::new(Planet::Earth, 0, 0);
        let unit_a = Unit::new(1, Team::Red, UnitType::Worker, 0, loc).unwrap();
        assert_eq!(unit_a.id, 1);
        assert_eq!(unit_a.team, Team::Red);
        assert_eq!(unit_a.unit_type(), UnitType::Worker);

        let mut c = match unit_a.controller {
            Worker(worker_c) => worker_c,
            _ => panic!("expected Worker"),
        };

        assert_eq!(c.level(), 0);
        assert_eq!(c.harvest_amount(), 3);
        assert_eq!(c.build_repair_health(), 5);

        c.research().unwrap();
        assert_eq!(c.level(), 1);
        assert_eq!(c.harvest_amount(), 4);
        assert_eq!(c.build_repair_health(), 5);

        c.research().unwrap();
        assert_eq!(c.level(), 2);
        assert_eq!(c.harvest_amount(), 4);
        assert_eq!(c.build_repair_health(), 6);

        let unit_b = Unit::new(2, Team::Red, UnitType::Worker, 2, loc).unwrap();
        let c = match unit_b.controller {
            Worker(worker_c) => worker_c,
            _ => panic!("expected Worker"),
        };

        assert_eq!(c.level(), 2);
        assert_eq!(c.harvest_amount(), 4);
        assert_eq!(c.build_repair_health(), 6);
    }
}
