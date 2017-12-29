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
    pub fn new(id: UnitID, team: Team, unit_type: UnitType, level: Level) -> Result<Unit, Error> {
        let controller = unit_type.default();
        let health = match controller {
            Worker(ref c) => c.max_health,
            Knight(ref c) => c.max_health,
            Ranger(ref c) => c.max_health,
            Mage(ref c) => c.max_health,
            Healer(ref c) => c.max_health,
            Factory(ref c) => c.max_health / 4,
            Rocket(ref c) => c.max_health / 4,
        };

        let mut unit = Unit {
            id: id,
            team: team,
            location: None,
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
    // ******************************* ACCESSORS ******************************
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

    /// The location of the unit, if currently on the map. Units can be
    /// temporarily removed from the map in rocket-related situations.
    pub fn location(&self) -> Option<MapLocation> {
        self.location
    }

    /// The current health of the unit.
    pub fn health(&self) -> u32 {
        self.health
    }

    /// The movement heat of the unit.
    pub fn movement_heat(&self) -> u32 {
        self.movement_heat
    }

    /// The attack heat of the unit.
    pub fn attack_heat(&self) -> u32 {
        self.attack_heat
    }

    // ************************************************************************
    // ************************** MOVEMENT METHODS ****************************
    // ************************************************************************

    /// Returns whether the unit is currently able to make a movement to a
    /// valid location.
    pub fn is_move_ready(&self) -> bool {
        match self.controller {
            // TODO: check if movement delay, etc. are ready.
            Knight(ref _controller) => true,
            _ => false,
        }
    }

    /// Move the unit to this location.
    pub fn move_to(&mut self, location: Option<MapLocation>) {
        self.location = location;
    }

    // ************************************************************************
    // *************************** COMBAT METHODS *****************************
    // ************************************************************************

    /// Take the amount of damage given, returning true if the unit has died.
    /// Returns false if the unit is still alive.
    pub fn take_damage(&mut self, damage: u32) -> bool {
        // TODO: Knight damage resistance??
        self.health -= cmp::min(damage, self.health);
        self.health == 0
    }

    // ************************************************************************
    // *********************** SPECIAL ABILITY METHODS ************************
    // ************************************************************************

    /// Returns the garrisoned units in this unit. Only applicable to Rockets,
    /// and returns None otherwise.
    pub fn garrisoned_units(&self) -> Option<Vec<Unit>> {
        match self.controller {
            Rocket(ref c) => Some(c.garrisoned_units()),
            _ => None,
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
        let unit_a = Unit::new(1, Team::Red, UnitType::Worker, 0).unwrap();
        assert_eq!(unit_a.id, 1);
        assert_eq!(unit_a.team, Team::Red);
        assert_eq!(unit_a.unit_type(), UnitType::Worker);

        let mut c = match unit_a.controller {
            Worker(worker_c) => worker_c,
            _ => panic!("expected Worker"),
        };

        assert_eq!(c.level, 0);
        assert_eq!(c.harvest_amount, 3);
        assert_eq!(c.build_repair_health, 5);

        c.research().unwrap();
        assert_eq!(c.level, 1);
        assert_eq!(c.harvest_amount, 4);
        assert_eq!(c.build_repair_health, 5);

        c.research().unwrap();
        assert_eq!(c.level, 2);
        assert_eq!(c.harvest_amount, 4);
        assert_eq!(c.build_repair_health, 6);

        let unit_b = Unit::new(2, Team::Red, UnitType::Worker, 2).unwrap();
        let c = match unit_b.controller {
            Worker(worker_c) => worker_c,
            _ => panic!("expected Worker"),
        };

        assert_eq!(c.level, 2);
        assert_eq!(c.harvest_amount, 4);
        assert_eq!(c.build_repair_health, 6);
    }
}
