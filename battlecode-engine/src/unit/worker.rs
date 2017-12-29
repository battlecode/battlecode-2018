//! Workers are the foundation of the civilization.

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerController {
    level: Level,
    max_health: u32,
    damage: i32,
    attack_range: u32,
    vision_range: u32,
    movement_cooldown: u32,
    attack_cooldown: u32,

    build_repair_health: u32,
    harvest_amount: u32,
}

impl RobotController for WorkerController {
    fn damage(&self) -> i32 {
        self.damage
    }

    fn attack_range(&self) -> u32 {
        self.attack_range
    }

    fn vision_range(&self) -> u32 {
        self.vision_range
    }

    fn movement_cooldown(&self) -> u32 {
        self.movement_cooldown
    }

    fn attack_cooldown(&self) -> u32 {
        self.attack_cooldown
    }
}

impl WorkerController {
    /// Default worker stats.
    pub fn default() -> WorkerController {
        WorkerController {
            level: 0,
            max_health: 100,
            damage: 0,
            attack_range: 0,
            vision_range: 50,
            movement_cooldown: 20,
            attack_cooldown: 0,
            build_repair_health: 5,
            harvest_amount: 3,
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

    /// The health restored when building or repairing a factory or rocket.
    pub fn build_repair_health(&self) -> u32 {
        self.build_repair_health
    }

    /// The maximum amount of karbonite harvested from a deposit in one turn.
    pub fn harvest_amount(&self) -> u32 {
        self.harvest_amount
    }

    /// The Worker's Tree
    ///
    /// 1. Gimme some of that Black Stuff: Workers harvest an additional +1
    ///    Karbonite from a deposit (not deducted from the deposit).
    /// 2. Time is of the Essence: Workers add 20% more health when repairing
    ///    or constructing a building.
    /// 3. Time is of the Essence II: Workers add another 20% more health when
    ///    repairing or constructing a building.
    /// 4. Time is of the Essence III: Workers add another 40% more health when
    ///    repairing or constructing a building.
    pub fn research(&mut self) -> Result<(), Error> {
        match self.level {
            0 => { self.harvest_amount += 1; },
            1 => { self.build_repair_health += 1; },
            2 => { self.build_repair_health += 1; },
            3 => { self.build_repair_health += 3; },
            _ => Err(GameError::InvalidResearchLevel)?,
        }
        self.level += 1;
        Ok(())
    }
}
