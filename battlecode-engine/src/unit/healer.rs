//! Healers are a suport unit that can heal other units.

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealerController {
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
    /// This amount of health is automatically restored to itself each round.
    pub self_heal_amount: u32,
    /// Whether Overcharge is unlocked.
    pub is_overcharge_unlocked: bool,
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
