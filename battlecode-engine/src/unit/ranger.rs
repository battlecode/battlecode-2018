//! Rangers are a ranged unit with good all-around combat.

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RangerController {
    level: Level,
    max_health: u32,
    damage: i32,
    attack_range: u32,
    vision_range: u32,
    movement_cooldown: u32,
    attack_cooldown: u32,

    cannot_attack_range: u32,
    is_snipe_unlocked: bool,
    countdown: u32,
    target_location: Option<MapLocation>,
}

impl RobotController for RangerController {
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

impl RangerController {
    /// Default Ranger stats.
    pub fn default() -> RangerController {
        RangerController {
            level: 0,
            max_health: 200,
            damage: 70,
            attack_range: 50,
            vision_range: 70,
            movement_cooldown: 20,
            attack_cooldown: 20,
            cannot_attack_range: 10,
            is_snipe_unlocked: false,
            countdown: 0,
            target_location: None,
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

    /// The range within which the Ranger cannot attack.
    pub fn cannot_attack_range(&self) -> u32 {
        self.cannot_attack_range
    }

    /// Whether Snipe is unlocked.
    pub fn is_snipe_unlocked(&self) -> bool {
        self.is_snipe_unlocked
    }

    /// The countdown (for Ranger snipe attacks).
    pub fn countdown(&self) -> u32 {
        self.countdown
    }

    /// The target location (for Ranger snipe attacks).
    pub fn target_location(&self) -> Option<MapLocation> {
        self.target_location
    }

    /// The Ranger's Tree
    ///
    /// 1. Get in Fast: Decreases a Ranger’s movement cooldown by 5.
    /// 2. Scopes: Increases a Ranger’s vision range by 30.
    /// 3. Snipe: Unlocks "Snipe" ability for Rangers.
    pub fn research(&mut self) -> Result<(), Error> {
        match self.level {
            0 => { self.movement_cooldown -= 5; },
            1 => { self.vision_range += 30; },
            2 => { self.is_snipe_unlocked = true; },
            _ => Err(GameError::InvalidResearchLevel)?,
        }
        self.level += 1;
        Ok(())
    }
}
