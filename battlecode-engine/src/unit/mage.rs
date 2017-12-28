use super::*;

/// Info specific to Mages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MageInfo {
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
    /// The percentage of standard attack damage dealt when exploding itself.
    pub explode_multiplier: Percent,
    /// Whether Blink is unlocked.
    pub is_blink_unlocked: bool,
    /// The radius in which a Mage is able to teleport with Blink.
    pub blink_radius: u32,
}

impl MageInfo {
    /// Default Mage stats.
    pub fn default() -> MageInfo {
        MageInfo {
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
