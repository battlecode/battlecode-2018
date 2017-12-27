//! Units are player-controlled entities that can perform certain
//! game actions, depending on their type.

use super::location::*;
use super::research::Level;
use super::world::Team;
use unit::UnitInfo::*;

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
            UnitType::Worker => Worker(WorkerInfo {
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
            }),
            UnitType::Knight => Knight(KnightInfo {
                level: 0,
                robot_stats: RobotStats {
                    max_health: 250,
                    damage: 100,
                    attack_range: 1,
                    vision_range: 50,
                    movement_cooldown: 15,
                    attack_cooldown: 20,
                }
            }),
            UnitType::Ranger => Ranger(RangerInfo {
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
            }),
            UnitType::Mage => Mage(MageInfo {
                level: 0,
                robot_stats: RobotStats {
                    max_health: 100,
                    damage: 150,
                    attack_range: 30,
                    vision_range: 30,
                    movement_cooldown: 20,
                    attack_cooldown: 20,
                }
            }),
            UnitType::Healer => Healer(HealerInfo {
                level: 0,
                robot_stats: RobotStats {
                    max_health: 100,
                    damage: -10,
                    attack_range: 30,
                    vision_range: 50,
                    movement_cooldown: 25,
                    attack_cooldown: 10,
                }
            }),
            UnitType::Factory => Factory(FactoryInfo {
                level: 0,
                max_health: 1000,
                production_queue: vec![],
                built: false,
            }),
            UnitType::Rocket => Rocket(RocketInfo {
                level: 0,
                max_health: 200,
                max_capacity: 8,
                built: false,
                garrisoned_units: vec![],
            }),
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
    /// The Worker's Tree
    ///
    /// 1) Gimme some of that Black Stuff: Workers harvest an additional +1
    ///    Karbonite from a deposit (not deducted from the deposit).
    /// 2) Time is of the Essence: Workers add 20% more health when repairing
    ///    or constructing a building.
    /// 3) Time is of the Essence II: Workers add 50% more health when
    ///    repairing or constructing a building.
    /// 4) Time is of the Essence III: Workers add 100% more health when
    ///    repairing or constructing a building.
    pub fn research(&mut self) {

    }
}

/// Info specific to Knights.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnightInfo {
    /// The research level.
    pub level: Level,
    /// The robot stats.
    pub robot_stats: RobotStats,
}

/// Info specific to Rangers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RangerInfo {
    /// The research level.
    pub level: Level,
    /// The robot stats.
    pub robot_stats: RobotStats,
    /// The range within the ranger cannot attack.
    pub cannot_attack_range: u32,
}

/// Info specific to Mages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MageInfo {
    /// The research level.
    pub level: Level,
    /// The robot stats.
    pub robot_stats: RobotStats,
}

/// Info specific to Healers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealerInfo {
    /// The research level.
    pub level: Level,
    /// The robot stats.
    pub robot_stats: RobotStats,
}

/// Info specific to factories.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactoryInfo {
    /// The research level.
    pub level: Level,
    /// Whether the factory has been built.
    pub built: bool,
    /// The maximum health.
    pub max_health: u32,
    /// Units queued to be produced.
    pub production_queue: Vec<Unit>,
}

/// Info specific to rockets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RocketInfo {
    /// The research level.
    pub level: Level,
    /// Whether the rocket has been built.
    pub built: bool,
    /// The maximum health.
    pub max_health: u32,
    /// The maximum number of robots it can hold at once.
    pub max_capacity: usize,
    /// The units contained within this rocket.
    pub garrisoned_units: Vec<UnitID>,
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

/// A single unit in the game.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Unit {
    /// The unique ID of a unit.
    pub id: UnitID,
    /// The team the unit belongs to.
    pub team: Team,

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

    /// The unit-specific info (a robot, factory, or rocket).
    pub unit_info: UnitInfo,
}

impl Unit {
    /// Create a new unit of the given type.
    pub fn new(id: UnitID, team: Team, unit_info: UnitInfo) -> Unit {
        let health = match unit_info {
            Worker(ref info) => info.robot_stats.max_health,
            Knight(ref info) => info.robot_stats.max_health,
            Ranger(ref info) => info.robot_stats.max_health,
            Mage(ref info) => info.robot_stats.max_health,
            Healer(ref info) => info.robot_stats.max_health,
            Factory(ref info) => info.max_health,
            Rocket(ref info) => info.max_health,
        };

        Unit {
            id: id,
            team: team,
            location: None,
            health: health,
            movement_heat: 0,
            attack_heat: 0,
            unit_info: unit_info,
        }
    }

    /// Create a generic unit, for testing purposes.
    pub fn test_unit(id: UnitID) -> Unit {
        let unit_info = Knight(KnightInfo {
            level: 0,
            robot_stats: RobotStats {
                max_health: 100,
                damage: 0,
                attack_range: 0,
                vision_range: 0,
                movement_cooldown: 0,
                attack_cooldown: 0,
            }
        });
        Unit::new(id, Team::Red, unit_info)
    }

    /// Returns whether the unit is currently able to make a movement to a valid location.
    pub fn is_move_ready(&self) -> bool {
        match self.unit_info {
            // TODO: check if movement delay, etc. are ready.
            Knight(ref _knight_info) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
