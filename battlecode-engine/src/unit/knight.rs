//! Knights are a melee unit that is strong in numbers.

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnightController {
    level: Level,
    max_health: u32,
    damage: i32,
    attack_range: u32,
    vision_range: u32,
    movement_cooldown: u32,
    attack_cooldown: u32,

    defense_per_robot: Percent,
    is_javelin_unlocked: bool,
    javelin_attack_range: u32,
    javelin_attack_cooldown: u32,
}

impl RobotController for KnightController {
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

impl KnightController {
    /// Default Knight stats.
    pub fn default() -> KnightController {
        KnightController {
            level: 0,
            max_health: 250,
            damage: 100,
            attack_range: 1,
            vision_range: 50,
            movement_cooldown: 15,
            attack_cooldown: 20,
            defense_per_robot: 1,
            is_javelin_unlocked: false,
            javelin_attack_range: 10,
            javelin_attack_cooldown: 15,
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

    /// The decrease in attack on the knight per adjacent robot.
    pub fn defense_per_robot(&self) -> Percent {
        self.defense_per_robot
    }

    /// Whether javelin is unlocked.
    pub fn is_javelin_unlocked(&self) -> bool {
        self.is_javelin_unlocked
    }

    /// Javelin attack range.
    pub fn javelin_attack_range(&self) -> u32 {
        self.javelin_attack_range
    }

    /// Javelin attack cooldown.
    pub fn javelin_attack_cooldown(&self) -> u32 {
        self.javelin_attack_cooldown
    }

    /// The Knight's Tree
    ///
    /// 1. Greater Strength in Numbers: Decreases the strength of an attack
    ///    on a knight by an additional 1% for each adjacent robot.
    /// 2. Even Greater Strength in Numbers: Decreases the strength of an
    ///    attack on a knight by an additional 1% (total 2%) for each adjacent
    ///    robot.
    /// 3. Javelin: Unlocks "Javelin" for Knights.
    pub fn research(&mut self) -> Result<(), Error> {
        match self.level {
            0 => { self.defense_per_robot += 1; },
            1 => { self.defense_per_robot += 1; },
            2 => { self.is_javelin_unlocked = true; },
            _ => Err(GameError::InvalidResearchLevel)?,
        }
        self.level += 1;
        Ok(())
    }
}
