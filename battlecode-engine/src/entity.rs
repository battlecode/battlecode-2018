//! Entities are player-controlled entities with certain characteristics and
//! game actions, depending on their type.

use super::location;
use super::world::Team;
use entity::Entity::*;

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
pub enum Entity {
    /// Knights are a melee unit that is strong in numbers.
    Knight(KnightInfo),
    /// Factories are the hub for producing combative robots.
    Factory(FactoryInfo),
    /// Rockets are the only unit that can move between planets.
    Rocket(RocketInfo),
}

/// Generic info for a single entity, and the associated body.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityInfo {
    pub id: EntityID,
    pub team: Team,
    pub max_health: u32,
    pub location: location::MapLocation,
    pub health: u32,

    /// The associated body (a robot, factory, or rocket).
    pub body: Entity,
}

/// Moves a robot to the given location.
pub fn move_location(entity: &EntityInfo, _location: &location::MapLocation) {
    match &entity.body {
        &Knight(ref _knight) => (),
        _ => ()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
