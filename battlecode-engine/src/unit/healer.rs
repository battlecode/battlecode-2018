//! Healers are a suport unit that can heal other units.

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealerController {
    level: Level,
    max_health: u32,
    damage: i32,
    attack_range: u32,
    vision_range: u32,
    movement_cooldown: u32,
    attack_cooldown: u32,

    self_heal_amount: u32,
    is_overcharge_unlocked: bool,
}

impl RobotController for HealerController {
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

impl HealerController {
    /// Default Healer stats.
    pub fn default() -> HealerController {
        HealerController {
            level: 0,
            max_health: 100,
            damage: -10,
            attack_range: 30,
            vision_range: 50,
            movement_cooldown: 25,
            attack_cooldown: 10,
            self_heal_amount: 1,
            is_overcharge_unlocked: false,
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

    /// This amount of health is automatically restored to itself each round.
    pub fn self_heal_amount(&self) -> u32 {
        self.self_heal_amount
    }

    /// Whether Overcharge is unlocked.
    pub fn is_overcharge_unlocked(&self) -> bool {
        self.is_overcharge_unlocked
    }

    /// The Healer's Tree
    ///
    /// 1. Spirit Water: Increases Healer’s healing ability by -2.
    /// 2. Spirit Water II: Increases Healer’s healing ability by an
    ///    additional +5.
    /// 3. Overcharge: Unlocks "Overcharge" for Healers.
    pub fn research(&mut self) -> Result<(), Error> {
        match self.level {
            0 => { self.damage -= 2; },
            1 => { self.damage -= 5; },
            2 => { self.is_overcharge_unlocked = true; },
            _ => Err(GameError::InvalidResearchLevel)?,
        }
        self.level += 1;
        Ok(())
    }
}
