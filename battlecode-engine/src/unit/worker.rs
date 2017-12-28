//! Workers are the foundation of the civilization.

use super::*;

/// Workers are the foundation of the civilization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerInfo {
    /// The research level.
    pub level: Level,
    /// The maximum health of the robot.
    pub max_health: u32,
    /// The damage inflicted by the robot during a normal attack.
    pub damage: i32,
    /// The distance squared, inclusive, of which a robot may attack.
    pub attack_range: u32,
    /// The distance squared, inclusive, of which a robot may see.
    pub vision_range: u32,
    /// The movement cooldown of the robot.
    pub movement_cooldown: u32,
    /// The attack cooldown of the robot.
    pub attack_cooldown: u32,
    /// The health restored when building or repairing a factory or rocket.
    pub build_repair_health: u32,
    /// The maximum amount of karbonite harvested from a deposit in one turn.
    pub harvest_amount: u32,
}

impl WorkerInfo {
    /// Default worker stats.
    pub fn default() -> WorkerInfo {
        WorkerInfo {
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
