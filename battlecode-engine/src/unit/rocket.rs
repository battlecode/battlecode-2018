//! Rockets are the only unit that can move between planets.

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RocketController {
    level: Level,
    max_health: u32,
    used: bool,
    max_capacity: usize,
    is_rocketry_unlocked: bool,
    travel_time_multiplier: Percent,
    garrisoned_units: Vec<UnitID>,
    is_ready: bool,
}

impl RocketController {
    /// Default Rocket stats.
    pub fn default() -> RocketController {
        RocketController {
            level: 0,
            max_health: 200,
            used: false,
            max_capacity: 8,
            is_rocketry_unlocked: false,
            travel_time_multiplier: 100,
            garrisoned_units: vec![],
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

    /// Whether the rocket has been used to travel to another planet.
    pub fn used(&self) -> bool {
        self.used
    }

    /// The maximum number of robots it can hold at once.
    pub fn max_capacity(&self) -> usize {
        self.max_capacity
    }

    /// Whether Rocketry has been researched.
    pub fn is_rocketry_unlocked(&self) -> bool {
        self.is_rocketry_unlocked
    }

    /// The percentage of typical travel time required by a rocket.
    pub fn travel_time_multiplier(&self) -> Percent {
        self.travel_time_multiplier
    }

    /// The units garrisoned inside a rocket.
    pub fn garrisoned_units(&self) -> Vec<UnitID> {
        self.garrisoned_units.clone()
    }

    /// Whether the unit is ready to be used.
    pub fn is_ready(&self) -> bool {
        self.is_ready
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

    /// Boards the unit by ID. Assumes the unit can board.
    pub fn push_unit(&mut self, id: UnitID) {
        self.garrisoned_units.push(id);
    }

    /// Remove and returns the first unit to board by ID.
    ///
    /// Errors if there are no units.
    pub fn remove_first_unit(&mut self) -> Result<UnitID, Error> {
        if self.garrisoned_units.len() == 0 {
            Err(GameError::InvalidAction)?
        }
        Ok(self.garrisoned_units.remove(0))
    }

    /// Marks the rocket as used.
    pub fn mark_used(&mut self) {
        self.used = true;
    }
}
