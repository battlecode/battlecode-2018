//! Entities are player-controlled entities with certain characteristics and
//! game actions, depending on their type.

use super::location;
use super::world::Team;
use super::error::GameError;
use entity::EntityInfo::*;

/// The ID of an entity is assigned when the entity is spawned. Each entity ID
/// is unique and in the range [0, 65,535], inclusive.
pub type EntityID = u16;

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
    production_queue: Vec<Entity>,
}

/// Entities are player-controlled entities with certain characteristics and
/// game actions, depending on their type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntityInfo {
    /// Knights are a melee unit that is strong in numbers.
    Knight(KnightInfo),
    /// Factories are the hub for producing combative robots.
    Factory(FactoryInfo),
    /// Rockets are the only unit that can move between planets.
    Rocket(RocketInfo),
}

/// Generic info for a single entity, and the associated specific info.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entity {
    pub id: EntityID,
    pub team: Team,
    pub max_health: u32,
    pub location: location::MapLocation,
    pub health: u32,

    /// The unit-specific info (a robot, factory, or rocket).
    pub spec: EntityInfo,
}

/// Moves a robot in the given direction.
pub fn entity_move(entity: &mut Entity, _direction: location::Direction) -> Result<(), GameError> {
    match entity.spec {
        Knight(ref _knight_info) => Ok(()),
        _ => Err(GameError::InternalEngineError)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
