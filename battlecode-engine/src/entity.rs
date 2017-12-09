//! Entities are player-controlled entities with certain characteristics and
//! game actions, depending on their type.

use super::location::MapLocation;
use super::world::Team;
use entity::RobotType::*;

/// The ID of an entity is assigned when the entity is spawned. Each entity ID
/// is unique and in the range [0, 65,535], inclusive.
pub type EntityID = u16;

/// RobotType names the different robot types.
#[derive(Debug, Serialize, Deserialize)]
pub enum RobotType {
    Knight,
}

/// RobotTypeInfo contains details on various attributes of the different
/// robots. All of this information is in the specs in a more organized form.
#[derive(Debug, Serialize, Deserialize)]
pub struct RobotTypeInfo {
    pub robot_type: RobotType,
    pub max_health: u32,
    pub damage: u32,
}

impl RobotTypeInfo {
    pub fn new(robot_type: RobotType) -> RobotTypeInfo {
        match robot_type {
            Knight => RobotTypeInfo {
                robot_type: Knight,
                max_health: 250,
                damage: 100,
            }
        }
    }
}

/// RobotInfo stores basic information of a robot that is public to both teams.
/// This info is ephemeral and there is no guarantee any of it will remain the
/// same between rounds.
#[derive(Debug, Serialize, Deserialize)]
pub struct RobotInfo {
    pub id: EntityID,
    pub team: Team,
    pub robot_type: RobotType,
    pub location: MapLocation,
    pub health: u32,
}

/// RobotInfo stores basic information of a rocket that is public to both teams.
/// This info is ephemeral and there is no guarantee any of it will remain the
/// same between rounds.
#[derive(Debug, Serialize, Deserialize)]
pub struct RocketInfo {
    pub id: EntityID,
    pub team: Team,
    pub location: MapLocation,
    pub health: u32,
}

/// FactoryInfo stores basic information of a factory that is public to both
/// teams. This info is ephemeral and there is no guarantee any of it will
/// remain the same between rounds.
#[derive(Debug, Serialize, Deserialize)]
pub struct FactoryInfo {
    pub id: EntityID,
    pub team: Team,
    pub location: MapLocation,
    pub health: u32,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
