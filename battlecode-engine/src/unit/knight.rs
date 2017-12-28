use super::*;

/// Info specific to Knights.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnightInfo {
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
    /// The decrease in attack on the knight per adjacent robot.
    pub defense_per_robot: Percent,
    /// Whether javelin is unlocked.
    pub is_javelin_unlocked: bool,
    /// Javelin attack range.
    pub javelin_attack_range: u32,
    /// Javelin attack cooldown.
    pub javelin_attack_cooldown: u32,
}

impl KnightInfo {
    /// Default Knight stats.
    pub fn default() -> KnightInfo {
        KnightInfo {
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
