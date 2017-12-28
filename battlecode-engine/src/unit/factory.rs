use super::*;

/// Info specific to factories.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactoryInfo {
    /// The research level.
    pub level: Level,
    /// The maximum health.
    pub max_health: u32,
    /// The unit production queue.
    production_queue: Vec<Unit>,
    /// Whether the unit is ready to be used.
    is_ready: bool,
}

impl FactoryInfo {
    /// Default Factory stats.
    pub fn default() -> FactoryInfo {
        FactoryInfo {
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
