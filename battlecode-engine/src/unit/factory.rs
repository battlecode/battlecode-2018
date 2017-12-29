//! Factories are the hub for producing combative robots.

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactoryController {
    level: Level,
    max_health: u32,
    production_queue: Vec<UnitType>,
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

    /// The research level.
    pub fn level(&self) -> Level {
        self.level
    }

    /// The maximum health.
    pub fn max_health(&self) -> u32 {
        self.max_health
    }

    /// The unit production queue.
    pub fn production_queue(&self) -> Vec<UnitType> {
        self.production_queue.clone()
    }

    /// Whether the unit is ready to be used.
    pub fn is_ready(&self) -> bool {
        self.is_ready
    }

    /// The Factory's Tree does not exist.
    pub fn research(&mut self) -> Result<(), Error> {
        Err(GameError::InvalidResearchLevel)?
    }
}
