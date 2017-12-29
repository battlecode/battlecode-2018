//! Mages are a fragile but specialized ranged unit for large areas.

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MageController {
    level: Level,
    max_health: u32,
    damage: i32,
    attack_range: u32,
    vision_range: u32,
    movement_cooldown: u32,
    attack_cooldown: u32,

    explode_multiplier: Percent,
    is_blink_unlocked: bool,
    blink_radius: u32,
}

impl RobotController for MageController {
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

impl MageController {
    /// Default Mage stats.
    pub fn default() -> MageController {
        MageController {
            level: 0,
            max_health: 100,
            damage: 150,
            attack_range: 30,
            vision_range: 30,
            movement_cooldown: 20,
            attack_cooldown: 20,
            explode_multiplier: 100,
            is_blink_unlocked: false,
            blink_radius: 5,
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

    /// The percentage of standard attack damage dealt when exploding itself.
    pub fn explode_multiplier(&self) -> Percent {
        self.explode_multiplier
    }

    /// Whether Blink is unlocked.
    pub fn is_blink_unlocked(&self) -> bool {
        self.is_blink_unlocked
    }

    /// The radius in which a Mage is able to teleport with Blink.
    pub fn blink_radius(&self) -> u32 {
        self.blink_radius
    }

    /// The Mage's Tree
    ///
    /// 1. Glass Cannon: Increases standard attack damage by +15 (total 10%).
    /// 2. Glass Cannon II: Increases standard attack damage by an additional
    ///    +15 (total 20%).
    /// 3. Watch the World Burn: Increase exploding damage from 100% to 150%
    ///    of its standard attack damage.
    /// 4. Blink: Unlocks "Blink" for Mages.
    pub fn research(&mut self) -> Result<(), Error> {
        match self.level {
            0 => { self.damage += 15; },
            1 => { self.damage += 15; },
            2 => { self.explode_multiplier += 50; },
            3 => { self.is_blink_unlocked = true; },
            _ => Err(GameError::InvalidResearchLevel)?,
        }
        self.level += 1;
        Ok(())
    }
}
