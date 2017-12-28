use super::*;

/// Info specific to rockets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RocketInfo {
    /// The research level.
    pub level: Level,
    /// The maximum health.
    pub max_health: u32,
    /// The maximum number of robots it can hold at once.
    pub max_capacity: usize,
    /// Whether Rocketry has been researched.
    pub is_rocketry_unlocked: bool,
    /// The percentage of typical travel time required by a rocket.
    pub travel_time_multiplier: Percent,
    /// The units garrisoned inside a rocket.
    garrisoned_units: Vec<Unit>,
    /// Whether the unit is ready to be used.
    is_ready: bool,
}

impl RocketInfo {
    /// Default Rocket stats.
    pub fn default() -> RocketInfo {
        RocketInfo {
            level: 0,
            max_health: 200,
            max_capacity: 8,
            is_rocketry_unlocked: false,
            travel_time_multiplier: 100,
            garrisoned_units: vec![],
            is_ready: false,
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

    pub fn garrisoned_units(&self) -> Vec<Unit> {
        self.garrisoned_units.clone()
    }
}
