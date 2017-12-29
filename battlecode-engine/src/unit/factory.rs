//! Factories are the hub for producing combative robots.

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactoryController {
    /// The research level.
    pub level: Level,
    /// The maximum health.
    pub max_health: u32,
    /// The unit production queue.
    production_queue: Vec<Unit>,
    /// Whether the unit is ready to be used.
    is_ready: bool,
}

impl FactoryController {
    /// Default Factory stats.
    pub fn default() -> FactoryController {
        FactoryController {
            level: 0,
            max_health: 1000,
            production_queue: vec![],
            is_ready: false,
        }
    }
    /// The Factory's Tree does not exist.
    pub fn research(&mut self) -> Result<(), Error> {
        Err(GameError::InvalidResearchLevel)?
    }
}
