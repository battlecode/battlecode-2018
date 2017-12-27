//! Research upgrades and unlocks capabilities of units. The entire research
//! tree has a linear branch associated with each unit type. Each branch has
//! an associated level, where Level 0 represents no research yet. Performing
//! an upgrade takes a fixed number of rounds, and unlocks the upgrade at the
//! next level in the branch.

use failure::Error;
use fnv::FnvHashMap;
use super::constants;
use super::error::GameError;
use super::unit::UnitType as Branch;
use super::world::Rounds;

/// Research level.
pub type Level = usize;

fn get_cost_array(branch: &Branch) -> Vec<Rounds> {
    match branch {
        &Branch::Worker  => constants::WORKER_COST.to_vec(),
        &Branch::Knight  => constants::KNIGHT_COST.to_vec(),
        &Branch::Ranger  => constants::RANGER_COST.to_vec(),
        &Branch::Mage    => constants::MAGE_COST.to_vec(),
        &Branch::Healer  => constants::HEALER_COST.to_vec(),
        &Branch::Factory => constants::FACTORY_COST.to_vec(),
        &Branch::Rocket  => constants::ROCKET_COST.to_vec(),
    }
}

/// Returns the maximum level of the research branch.
pub fn get_max_level(branch: &Branch) -> Level {
    get_cost_array(branch).len() as Level - 1
}

/// Returns the cost of a level, in rounds, of a research branch. Errors if the
/// level can't be researched i.e. not in the range [0, get_max_level(branch)].
pub fn get_cost(branch: &Branch, level: Level) -> Result<Rounds, Error> {
    if let Some(cost) = get_cost_array(branch).get(level) {
        Ok(*cost)
    } else {
        Err(GameError::InvalidResearchLevel)?
    }
}

/// The status of research for a single team.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ResearchInfo {
    /// The current level of each research branch, starting at 0.
    level: FnvHashMap<Branch, Level>,

    /// The level of each branch if the queue were to be exhausted.
    maybe_level: FnvHashMap<Branch, Level>,

    /// Branches queued to be researched, including the current branch.
    queue: Vec<Branch>,

    /// The number of rounds to go until the first branch in the queue is
    /// finished, if the queue is not empty.
    rounds_left: Option<Rounds>,
}

impl ResearchInfo {
    /// Construct an initial research state.
    pub fn new() -> ResearchInfo {
        let mut level = FnvHashMap::default();
        for unit in Branch::all() {
            level.insert(unit.clone(), 0);
        }

        ResearchInfo {
            level: level.clone(),
            maybe_level: level,
            queue: vec![],
            rounds_left: None,
        }
    }

    /// Returns the current level of the research branch.
    pub fn get_level(&self, branch: &Branch) -> Level {
        if let Some(level) = self.level.get(branch) {
            level.clone()
        } else {
            unreachable!();
        }
    }

    fn get_level_mut(&mut self, branch: &Branch) -> &mut Level {
        if let Some(level) = self.level.get_mut(branch) {
            level
        } else {
            unreachable!();
        }
    }

    fn get_maybe_level(&mut self, branch: &Branch) -> Level {
        if let Some(level) = self.maybe_level.get(branch) {
            *level
        } else {
            unreachable!();
        }
    }

    fn get_maybe_level_mut(&mut self, branch: &Branch) -> &mut Level {
        if let Some(level) = self.maybe_level.get_mut(branch) {
            level
        } else {
            unreachable!();
        }
    }

    /// Returns the research queue, where the front of the queue is at the
    /// beginning of the list.
    pub fn get_queue(&self) -> Vec<Branch> {
        self.queue.clone()
    }

    /// Returns the next branch to be researched, which is the branch at the
    /// front of the research queue. Returns None if the queue is empty.
    pub fn get_next_in_queue(&self) -> Option<Branch> {
        self.queue.get(0).map(|branch| branch.clone())
    }

    /// Returns the number of rounds left until the upgrade at the front of the
    /// research queue is applied, or None if the queue is empty.
    pub fn get_rounds_left(&self) -> Option<Rounds> {
        self.rounds_left.clone()
    }

    /// Sets the number of rounds left to the cost of the first thing in the
    /// queue. Sets the cost to None if the queue is empty.
    fn reset_rounds_left(&mut self) {
        if self.queue.len() == 0 {
            self.rounds_left = None;
            return;
        }

        let branch = &self.queue[0];
        let level = self.get_level(branch) + 1;
        if let Ok(cost) = get_cost(branch, level) {
            self.rounds_left = Some(cost);
        } else {
            unreachable!();
        }
    }

    /// Resets the research queue to be empty. Returns true if the queue was
    /// not empty before, and false otherwise.
    pub fn reset_queue(&mut self) -> bool {
        let old_queue = self.queue.clone();
        let old_queue_len = old_queue.len();
        for branch in old_queue {
            *self.get_maybe_level_mut(&branch) -= 1;
        }
        self.queue.clear();
        self.rounds_left = None;
        old_queue_len != 0
    }

    /// Adds a branch to the back of the queue, if it is a valid upgrade, and
    /// starts research if it is the first in the queue.
    ///
    /// Returns whether the branch was successfully added.
    pub fn add_to_queue(&mut self, branch: &Branch) -> bool {
        let new_level = self.get_maybe_level(branch) + 1;
        let max_level = get_max_level(branch);
        if new_level > max_level {
            return false;
        }

        self.queue.push(branch.clone());
        self.maybe_level.insert(branch.clone(), new_level);
        if self.queue.len() == 1 {
            self.reset_rounds_left();
        }
        true
    }

    /// Updates the internal state of research as if another round has passed.
    /// If an upgrade has completed, returns the branch that was just upgraded,
    /// and continues work on the next upgrade in the queue.
    ///
    /// Otherwise returns None.
    pub fn update(&mut self) -> Option<Branch> {
        if let Some(rounds_left) = self.rounds_left {
            if rounds_left > 1 {
                self.rounds_left = Some(rounds_left - 1);
                return None;
            }

            let branch = self.queue.remove(0);
            *self.get_level_mut(&branch) += 1;
            self.reset_rounds_left();
            Some(branch)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ResearchInfo;
    use super::super::unit::UnitType as Branch;

    #[test]
    fn static_cost_level_getters() {
        for branch in Branch::all() {
            let max_level = super::get_max_level(&branch);
            for level in 1..max_level + 1 {
                assert!(super::get_cost(&branch, level).unwrap() > 0);
            }
            assert_eq!(super::get_cost(&branch, 0).unwrap(), 0);
            assert!(super::get_cost(&branch, max_level + 1).is_err());
        }
    }

    #[test]
    fn research_info_constructor() {
        let r = ResearchInfo::new();
        assert_eq!(r.get_queue(), vec![]);
        assert_eq!(r.get_next_in_queue(), None);
        assert_eq!(r.get_rounds_left(), None);

        for branch in Branch::all() {
            assert_eq!(r.get_level(&branch), 0);
        }
    }

    #[test]
    fn simple_research_queue_mutators() {
        let mut r = ResearchInfo::new();
        let knight_cost = super::get_cost(&Branch::Knight, 1).unwrap();

        // Add a Knight.
        assert!(r.add_to_queue(&Branch::Knight));
        assert_eq!(r.get_queue(), vec![Branch::Knight]);
        assert_eq!(r.get_next_in_queue(), Some(Branch::Knight));
        assert_eq!(r.get_rounds_left(), Some(knight_cost));

        // Add a Mage.
        assert!(r.add_to_queue(&Branch::Mage));
        assert_eq!(r.get_queue(), vec![Branch::Knight, Branch::Mage]);
        assert_eq!(r.get_next_in_queue(), Some(Branch::Knight));
        assert_eq!(r.get_rounds_left(), Some(knight_cost));

        // Reset a queue with items in it.
        assert!(r.reset_queue());
        assert_eq!(r.get_queue(), vec![]);
        assert_eq!(r.get_next_in_queue(), None);
        assert_eq!(r.get_rounds_left(), None);

        // Reset an empty queue.
        assert!(!r.reset_queue());

        // Try to add the Knight too many times.
        let max_level = super::get_max_level(&Branch::Knight);
        for _ in 0..max_level {
            assert!(r.add_to_queue(&Branch::Knight));
        }
        assert!(!r.add_to_queue(&Branch::Knight));
    }

    #[test]
    fn update_trivial() {
        let mut r = ResearchInfo::new();
        assert_eq!(r.update(), None);
        assert_eq!(r.get_queue(), vec![]);
        assert_eq!(r.get_next_in_queue(), None);
        assert_eq!(r.get_rounds_left(), None);
    }

    #[test]
    fn update_no_reset() {
        let mut r = ResearchInfo::new();
        let knight_cost_l1 = super::get_cost(&Branch::Knight, 1).unwrap();
        let knight_cost_l2 = super::get_cost(&Branch::Knight, 2).unwrap();
        let mage_cost = super::get_cost(&Branch::Mage, 1).unwrap();

        // Research the Knight twice and Mage once.
        assert!(r.add_to_queue(&Branch::Knight));
        assert!(r.add_to_queue(&Branch::Knight));
        assert!(r.add_to_queue(&Branch::Mage));

        // Assume the Knight costs more than one round at each level.
        assert!(knight_cost_l1 > 1);
        assert!(knight_cost_l2 > 1);

        // Proceed one round.
        assert_eq!(r.update(), None);
        assert_eq!(r.get_rounds_left(), Some(knight_cost_l1 - 1));
        assert_eq!(r.get_level(&Branch::Knight), 0);
        assert_eq!(r.get_level(&Branch::Mage), 0);

        // Research the first Knight.
        for _ in 1..knight_cost_l1 - 1 {
            assert_eq!(r.update(), None);
        }
        assert_eq!(r.update(), Some(Branch::Knight));
        assert_eq!(r.get_queue(), vec![Branch::Knight, Branch::Mage]);
        assert_eq!(r.get_rounds_left(), Some(knight_cost_l2));
        assert_eq!(r.get_level(&Branch::Knight), 1);
        assert_eq!(r.get_level(&Branch::Mage), 0);

        // Research the second Knight.
        for _ in 1..knight_cost_l2 {
            assert_eq!(r.update(), None);
        }
        assert_eq!(r.update(), Some(Branch::Knight));
        assert_eq!(r.get_queue(), vec![Branch::Mage]);
        assert_eq!(r.get_rounds_left(), Some(mage_cost));
        assert_eq!(r.get_level(&Branch::Knight), 2);
        assert_eq!(r.get_level(&Branch::Mage), 0);

        // Finish researching the Mage. The queue is now empty.
        for _ in 1..mage_cost {
            assert_eq!(r.update(), None);
        }
        assert_eq!(r.update(), Some(Branch::Mage));
        assert_eq!(r.get_rounds_left(), None);
        assert_eq!(r.get_level(&Branch::Knight), 2);
        assert_eq!(r.get_level(&Branch::Mage), 1);
    }

    #[test]
    fn update_with_reset() {
        let mut r = ResearchInfo::new();
        let knight_cost = super::get_cost(&Branch::Knight, 1).unwrap();
        let mage_cost = super::get_cost(&Branch::Mage, 1).unwrap();

        // Start researching the knight and the mage.
        assert!(r.add_to_queue(&Branch::Knight));
        assert!(r.add_to_queue(&Branch::Mage));

        // Assume the Knight costs more than one round.
        assert!(knight_cost > 1);

        // Proceed one round.
        assert_eq!(r.update(), None);
        assert_eq!(r.get_rounds_left(), Some(knight_cost - 1));
        assert_eq!(r.get_level(&Branch::Knight), 0);
        assert_eq!(r.get_level(&Branch::Mage), 0);

        // Reset the queue and proceed a round.
        assert!(r.reset_queue());
        assert_eq!(r.update(), None);
        assert_eq!(r.get_rounds_left(), None);
        assert_eq!(r.get_level(&Branch::Knight), 0);
        assert_eq!(r.get_level(&Branch::Mage), 0);

        // Start researching again and proceed to completion.
        assert!(r.add_to_queue(&Branch::Knight));
        assert!(r.add_to_queue(&Branch::Mage));
        for _ in 0..knight_cost + mage_cost {
            r.update();
        }
        assert_eq!(r.get_rounds_left(), None);
        assert_eq!(r.get_level(&Branch::Knight), 1);
        assert_eq!(r.get_level(&Branch::Mage), 1);
    }
}
