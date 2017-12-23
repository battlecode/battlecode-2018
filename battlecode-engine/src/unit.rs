//! Units are player-controlled entities that can perform certain
//! game actions, depending on their type.

use super::location;
use super::world::Team;
use unit::UnitInfo::*;

/// The ID of an unit is assigned when the unit is spawned. Each unit ID
/// is unique and in the range [0, 65,535], inclusive.
pub type UnitID = u16;

/// Info specific to knights.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnightInfo {

}

/// Info specific to rockets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RocketInfo {

}

/// Info specific to factories.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactoryInfo {
    /// Entities queued to be produced.
    production_queue: Vec<Unit>,
}

/// Entities are player-controlled units with certain characteristics and
/// game actions, depending on their type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UnitInfo {
    /// Knights are a melee unit that is strong in numbers.
    Knight(KnightInfo),
    /// Factories are the hub for producing combative robots.
    Factory(FactoryInfo),
    /// Rockets are the only unit that can move between planets.
    Rocket(RocketInfo),
}

impl Default for UnitInfo {
    fn default() -> UnitInfo {
        Knight(KnightInfo {})
    }
}

/// Generic info for a single unit, and the associated specific info.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Unit {
    pub id: UnitID,
    pub team: Team,
    pub max_health: u32,
    pub location: location::MapLocation,
    pub health: u32,

    /// The unit-specific info (a robot, factory, or rocket).
    pub unit_info: UnitInfo,
}

impl Unit {
    pub fn new(id: UnitID,
               team: Team,
               max_health: u32,
               location: location::MapLocation,
               health: u32,
               unit_info: UnitInfo) -> Unit {
        Unit {
            id, team, max_health, location, health, unit_info
        }
    }

    /// Create a generic unit, for testing purposes.
    pub fn test_unit(id: UnitID) -> Unit {
        Unit {
            id: id,
            team: Team::Red,
            max_health: 10,
            location: location::MapLocation::new(location::Planet::Earth, -1, -1),
            health: 10,
            unit_info: Knight(KnightInfo{}),
        }
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
