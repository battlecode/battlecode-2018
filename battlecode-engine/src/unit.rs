//! Units are player-controlled entities that can perform certain
//! game actions, depending on their type.

use failure::Error;
use super::error::GameError;
use super::location::*;
use super::research::Level;
use super::world::Team;
use unit::UnitInfo::*;
use super::constants::*;

/// Percentage.
pub type Percent = u32;

/// The ID of an unit is assigned when the unit is spawned.
pub type UnitID = u32;

/// The different unit types, which include factories, rockets, and the robots.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
    pub fn default(&self) -> UnitInfo {
        match *self {
            UnitType::Worker => Worker(WorkerInfo::default()),
            UnitType::Knight => Knight(KnightInfo::default()),
            UnitType::Ranger => Ranger(RangerInfo::default()),
            UnitType::Mage => Mage(MageInfo::default()),
            UnitType::Healer => Healer(HealerInfo::default()),
            UnitType::Factory => Factory(FactoryInfo::default()),
            UnitType::Rocket => Rocket(RocketInfo::default()),
        }
    }
}

/// Inherent properties of a robot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RobotStats {
    /// The maximum health of the robot.
    pub max_health: u32,
    /// The damage inflicted by the robot during a normal attack.
    pub damage: i32,
    /// The distance squared, inclusive, of which a robot may attack.
    pub attack_range: u32,
    /// The distance squared, inclusive, of which a robot may see.
    pub vision_range: u32,
    /// The movement cooldown of the robot.
    pub movement_cooldown: u32,
    /// The attack cooldown of the robot.
    pub attack_cooldown: u32,
}

/// Info specific to Workers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerInfo {
    /// The research level.
    pub level: Level,
    /// The robot stats.
    pub robot_stats: RobotStats,
    /// The health restored when building or repairing a factory or rocket.
    pub build_repair_health: u32,
    /// The maximum amount of karbonite harvested from a deposit in one turn.
    pub harvest_amount: u32,
}

impl WorkerInfo {
    /// Default worker stats.
    pub fn default() -> WorkerInfo {
        WorkerInfo {
            level: 0,
            robot_stats: RobotStats {
                max_health: 100,
                damage: 0,
                attack_range: 0,
                vision_range: 50,
                movement_cooldown: 20,
                attack_cooldown: 0,
            },
            build_repair_health: 5,
            harvest_amount: 3,
        }
    }

    /// The Worker's Tree
    ///
    /// 1. Gimme some of that Black Stuff: Workers harvest an additional +1
    ///    Karbonite from a deposit (not deducted from the deposit).
    /// 2. Time is of the Essence: Workers add 20% more health when repairing
    ///    or constructing a building.
    /// 3. Time is of the Essence II: Workers add another 20% more health when
    ///    repairing or constructing a building.
    /// 4. Time is of the Essence III: Workers add another 40% more health when
    ///    repairing or constructing a building.
    pub fn research(&mut self) -> Result<(), Error> {
        match self.level {
            0 => { self.harvest_amount += 1; },
            1 => { self.build_repair_health += 1; },
            2 => { self.build_repair_health += 1; },
            3 => { self.build_repair_health += 3; },
            _ => Err(GameError::InvalidResearchLevel)?,
        }
        self.level += 1;
        Ok(())
    }
}

/// Info specific to Knights.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnightInfo {
    /// The research level.
    pub level: Level,
    /// The robot stats.
    pub robot_stats: RobotStats,
    /// The decrease in attack on the knight per adjacent robot.
    pub defense_per_robot: Percent,
    /// Whether javelin is unlocked.
    pub is_javelin_unlocked: bool,
    /// Javelin attack range.
    pub javelin_attack_range: u32,
    /// Javelin attack cooldown.
    pub javelin_attack_cooldown: u32,
}

impl KnightInfo {
    /// Default Knight stats.
    pub fn default() -> KnightInfo {
        KnightInfo {
            level: 0,
            robot_stats: RobotStats {
                max_health: 250,
                damage: 100,
                attack_range: 1,
                vision_range: 50,
                movement_cooldown: 15,
                attack_cooldown: 20,
            },
            defense_per_robot: 1,
            is_javelin_unlocked: false,
            javelin_attack_range: 10,
            javelin_attack_cooldown: 15,
        }
    }

    /// The Knight's Tree
    ///
    /// 1. Greater Strength in Numbers: Decreases the strength of an attack
    ///    on a knight by an additional 1% for each adjacent robot.
    /// 2. Even Greater Strength in Numbers: Decreases the strength of an
    ///    attack on a knight by an additional 1% (total 2%) for each adjacent
    ///    robot.
    /// 3. Javelin: Unlocks "Javelin" for Knights.
    pub fn research(&mut self) -> Result<(), Error> {
        match self.level {
            0 => { self.defense_per_robot += 1; },
            1 => { self.defense_per_robot += 1; },
            2 => { self.is_javelin_unlocked = true; },
            _ => Err(GameError::InvalidResearchLevel)?,
        }
        self.level += 1;
        Ok(())
    }
}

/// Info specific to Rangers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RangerInfo {
    /// The research level.
    pub level: Level,
    /// The robot stats.
    pub robot_stats: RobotStats,
    /// The range within which the Ranger cannot attack.
    pub cannot_attack_range: u32,
    /// Whether Snipe is unlocked.
    pub is_snipe_unlocked: bool,
}

impl RangerInfo {
    /// Default Ranger stats.
    pub fn default() -> RangerInfo {
        RangerInfo {
            level: 0,
            robot_stats: RobotStats {
                max_health: 200,
                damage: 70,
                attack_range: 50,
                vision_range: 70,
                movement_cooldown: 20,
                attack_cooldown: 20,
            },
            cannot_attack_range: 10,
            is_snipe_unlocked: false,
        }
    }

    /// The Ranger's Tree
    ///
    /// 1. Get in Fast: Decreases a Ranger’s movement cooldown by 5.
    /// 2. Scopes: Increases a Ranger’s vision range by 30.
    /// 3. Snipe: Unlocks "Snipe" ability for Rangers.
    pub fn research(&mut self) -> Result<(), Error> {
        match self.level {
            0 => { self.robot_stats.movement_cooldown -= 5; },
            1 => { self.robot_stats.vision_range += 30; },
            2 => { self.is_snipe_unlocked = true; },
            _ => Err(GameError::InvalidResearchLevel)?,
        }
        self.level += 1;
        Ok(())
    }
}

/// Info specific to Mages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MageInfo {
    /// The research level.
    pub level: Level,
    /// The robot stats.
    pub robot_stats: RobotStats,
    /// The percentage of standard attack damage dealt when exploding itself.
    pub explode_multiplier: Percent,
    /// Whether Blink is unlocked.
    pub is_blink_unlocked: bool,
    /// The radius in which a Mage is able to teleport with Blink.
    pub blink_radius: u32,
}

impl MageInfo {
    /// Default Mage stats.
    pub fn default() -> MageInfo {
        MageInfo {
            level: 0,
            robot_stats: RobotStats {
                max_health: 100,
                damage: 150,
                attack_range: 30,
                vision_range: 30,
                movement_cooldown: 20,
                attack_cooldown: 20,
            },
            explode_multiplier: 100,
            is_blink_unlocked: false,
            blink_radius: 5,
        }
    }

    /// The Mage's Tree
    ///
    /// 1. Glass Cannon: Increases standard attack damage by +15 (total 10%).
    /// 2. Glass Cannon II: Increases standard attack damage by an additional
    ///    +15 (total 20%).
    /// 3. Watch the World Burn: Increase exploding damage from 100% to 150%
    ///    of its standard attack damage.
    /// 4. Blink: Unlocks "Blink" for Mages.
    pub fn research(&mut self) -> Result<(), Error> {
        match self.level {
            0 => { self.robot_stats.damage += 15; },
            1 => { self.robot_stats.damage += 15; },
            2 => { self.explode_multiplier += 50; },
            3 => { self.is_blink_unlocked = true; },
            _ => Err(GameError::InvalidResearchLevel)?,
        }
        self.level += 1;
        Ok(())
    }
}

/// Info specific to Healers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealerInfo {
    /// The research level.
    pub level: Level,
    /// The robot stats.
    pub robot_stats: RobotStats,
    /// This amount of health is automatically restored to itself each round.
    pub self_heal_amount: u32,
    /// Whether Overcharge is unlocked.
    pub is_overcharge_unlocked: bool,
}

impl HealerInfo {
    /// Default Healer stats.
    pub fn default() -> HealerInfo {
        HealerInfo {
            level: 0,
            robot_stats: RobotStats {
                max_health: 100,
                damage: -10,
                attack_range: 30,
                vision_range: 50,
                movement_cooldown: 25,
                attack_cooldown: 10,
            },
            self_heal_amount: 1,
            is_overcharge_unlocked: false,
        }
    }

    /// The Healer's Tree
    ///
    /// 1. Spirit Water: Increases Healer’s healing ability by -2.
    /// 2. Spirit Water II: Increases Healer’s healing ability by an
    ///    additional +5.
    /// 3. Overcharge: Unlocks "Overcharge" for Healers.
    pub fn research(&mut self) -> Result<(), Error> {
        match self.level {
            0 => { self.robot_stats.damage -= 2; },
            1 => { self.robot_stats.damage -= 5; },
            2 => { self.is_overcharge_unlocked = true; },
            _ => Err(GameError::InvalidResearchLevel)?,
        }
        self.level += 1;
        Ok(())
    }
}

/// Info specific to factories.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactoryInfo {
    /// The research level.
    pub level: Level,
    /// The maximum health.
    pub max_health: u32,
}

impl FactoryInfo {
    /// Default Factory stats.
    pub fn default() -> FactoryInfo {
        FactoryInfo {
            level: 0,
            max_health: 1000,
        }
    }
    /// The Factory's Tree does not exist.
    pub fn research(&mut self) -> Result<(), Error> {
        Err(GameError::InvalidResearchLevel)?
    }
}

/// Info specific to rockets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RocketInfo {
    /// The research level.
    pub level: Level,
    /// The maximum health.
    pub max_health: u32,
    /// The maximum number of robots it can hold at once.
    pub used: bool,
    /// The maximum number of robots it can hold at once.
    pub max_capacity: usize,
    /// Whether Rocketry has been researched.
    pub is_rocketry_unlocked: bool,
    /// The percentage of typical travel time required by a rocket.
    pub travel_time_multiplier: Percent,
}

impl RocketInfo {
    /// Default Rocket stats.
    pub fn default() -> RocketInfo {
        RocketInfo {
            level: 0,
            max_health: 200,
            used: false,
            max_capacity: 8,
            is_rocketry_unlocked: false,
            travel_time_multiplier: 100,
        }
    }

    /// The Rocket's Tree
    ///
    /// 1. Rocketry: Unlocks rocket technology. Workers can now blueprint and
    ///    build rockets.
    /// 2. Rocket Boosters: Reduces rocket travel time by 20% compared to the
    ///    travel time determined by the orbit of the planets.
    /// 3. Increased Capacity: Allows rockets to garrison 50% more units per
    ///    rocket.
    pub fn research(&mut self) -> Result<(), Error> {
        match self.level {
            0 => { self.is_rocketry_unlocked = true; },
            1 => { self.travel_time_multiplier -= 20; },
            2 => { self.max_capacity += 4; },
            _ => Err(GameError::InvalidResearchLevel)?,
        }
        self.level += 1;
        Ok(())
    }
}

/// Units are player-controlled objects with certain characteristics and
/// game actions, depending on their type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UnitInfo {
    /// Workers are the foundation of the civilization.
    Worker(WorkerInfo),
    /// Knights are a melee unit that is strong in numbers.
    Knight(KnightInfo),
    /// Rangers are a ranged unit with good all-around combat.
    Ranger(RangerInfo),
    /// Mages are a fragile but specialized ranged unit for large areas.
    Mage(MageInfo),
    /// Healers are a suport unit that can heal other units.
    Healer(HealerInfo),
    /// Factories are the hub for producing combative robots.
    Factory(FactoryInfo),
    /// Rockets are the only unit that can move between planets.
    Rocket(RocketInfo),
}

impl UnitInfo {
    /// Research the next level.
    pub fn research(&mut self) -> Result<(), Error> {
        match self {
            &mut Worker(ref mut info)  => info.research(),
            &mut Knight(ref mut info)  => info.research(),
            &mut Ranger(ref mut info)  => info.research(),
            &mut Mage(ref mut info)    => info.research(),
            &mut Healer(ref mut info)  => info.research(),
            &mut Factory(ref mut info) => info.research(),
            &mut Rocket(ref mut info)  => info.research(),
        }
    }
}

/// A single unit in the game.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Unit {
    /// The unique ID of a unit.
    pub id: UnitID,
    /// The team the unit belongs to.
    pub team: Team,
    /// The unit type.
    pub unit_type: UnitType,

    /// The location of the unit, if currently on the map. Units
    /// can be temporarily removed from the map in rocket-related
    /// situations.
    pub location: Option<MapLocation>,
    /// The current health of the unit.
    pub health: u32,
    /// The movement heat of the unit.
    pub movement_heat: u32,
    /// The attack heat of the unit.
    pub attack_heat: u32,

    /// The countdown (for Ranger snipe attacks).
    countdown: u32,
    /// The target location (for Ranger snipe attacks).
    target_location: Option<MapLocation>,
    /// The Factory production queue.
    production_queue: Vec<Unit>,
    /// The units inside a rocket.
    garrisoned_units: Vec<UnitID>,
    /// Whether the unit is ready to be used (Factories and Rockets).
    is_ready: bool,

    /// The unit-specific info (a robot, factory, or rocket).
    pub unit_info: UnitInfo,
}

impl Unit {
    /// Create a new unit of the given type.
    pub fn new(id: UnitID, team: Team, unit_type: UnitType, level: Level) -> Result<Unit, Error> {
        let mut unit_info = unit_type.default();
        for _ in 0..level {
            unit_info.research()?;
        }

        let health = match unit_info {
            Worker(ref info) => info.robot_stats.max_health,
            Knight(ref info) => info.robot_stats.max_health,
            Ranger(ref info) => info.robot_stats.max_health,
            Mage(ref info) => info.robot_stats.max_health,
            Healer(ref info) => info.robot_stats.max_health,
            Factory(ref info) => info.max_health / 4,
            Rocket(ref info) => info.max_health / 4,
        };

        let is_ready = unit_type != UnitType::Factory && unit_type != UnitType::Rocket;
        Ok(Unit {
            id: id,
            team: team,
            unit_type: unit_type,
            location: None,
            health: health,
            movement_heat: 0,
            attack_heat: 0,
            countdown: 0,
            target_location: None,
            production_queue: vec![],
            garrisoned_units: vec![],
            is_ready: is_ready,
            unit_info: unit_info,
        })
    }

    /// Create a generic unit, for testing purposes.
    pub fn test_unit(id: UnitID) -> Result<Unit, Error> {
        Unit::new(id, Team::Red, UnitType::Knight, 1)
    }

    /// Returns whether the unit is currently able to make a movement to a valid location.
    pub fn is_move_ready(&self) -> bool {
        match self.unit_info {
            // TODO: check if movement delay, etc. are ready.
            Worker(_) => self.movement_heat < MAX_HEAT_TO_ACT,
            Knight(_) => self.movement_heat < MAX_HEAT_TO_ACT,
            Ranger(_) => self.movement_heat < MAX_HEAT_TO_ACT,
            Mage(_) => self.movement_heat < MAX_HEAT_TO_ACT,
            Healer(_) => self.movement_heat < MAX_HEAT_TO_ACT,
            _ => false,
        }
    }

    /// Returns the garrisoned units in this unit. Only applicable to Rockets,
    /// and returns None otherwise.
    pub fn get_garrisoned_units(&self) -> Result<&Vec<UnitID>, Error> {
        match self.unit_type {
            UnitType::Rocket => Ok(&self.garrisoned_units),
            _ => Err(GameError::InappropriateUnitType)?,
        }
    }

    /// Returns the garrisoned units in this unit. Only applicable to Rockets,
    /// and returns None otherwise.
    pub fn get_garrisoned_units_mut(&mut self) -> Result<&mut Vec<UnitID>, Error> {
        match self.unit_type {
            UnitType::Rocket => Ok(&mut self.garrisoned_units),
            _ => Err(GameError::InappropriateUnitType)?,
        }
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
        assert_eq!(unit_a.unit_type, UnitType::Worker);

        let mut info = match unit_a.unit_info {
            Worker(worker_info) => worker_info,
            _ => panic!("expected Worker"),
        };

        assert_eq!(info.level, 0);
        assert_eq!(info.harvest_amount, 3);
        assert_eq!(info.build_repair_health, 5);

        info.research().unwrap();
        assert_eq!(info.level, 1);
        assert_eq!(info.harvest_amount, 4);
        assert_eq!(info.build_repair_health, 5);

        info.research().unwrap();
        assert_eq!(info.level, 2);
        assert_eq!(info.harvest_amount, 4);
        assert_eq!(info.build_repair_health, 6);

        let unit_b = Unit::new(2, Team::Red, UnitType::Worker, 2).unwrap();
        let info = match unit_b.unit_info {
            Worker(worker_info) => worker_info,
            _ => panic!("expected Worker"),
        };

        assert_eq!(info.level, 2);
        assert_eq!(info.harvest_amount, 4);
        assert_eq!(info.build_repair_health, 6);
    }
}
