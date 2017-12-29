//! Rangers are a ranged unit with good all-around combat.

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RangerController {
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
    /// The range within which the Ranger cannot attack.
    pub cannot_attack_range: u32,
    /// Whether Snipe is unlocked.
    pub is_snipe_unlocked: bool,
    /// The countdown (for Ranger snipe attacks).
    countdown: u32,
    /// The target location (for Ranger snipe attacks).
    target_location: Option<MapLocation>,
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
