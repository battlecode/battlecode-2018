//! Entities are player-controlled units with certain characteristics and
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
    /// Returns whether the unit is currently able to make a movement to a valid location.
    pub fn is_move_ready(&self) -> bool {
        match self.unit_info {
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
