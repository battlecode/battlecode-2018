//! Rockets are the only unit that can initiate travel between planets.

use fnv::FnvHashMap;
use std::cmp::Ordering;

use super::unit::UnitID;
use super::world::Rounds;
use super::location::MapLocation;

/// A rocket landing.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct RocketLanding {
    pub rocket_id: UnitID,
    pub destination: MapLocation,
}

impl Ord for RocketLanding {
    fn cmp(&self, other: &RocketLanding) -> Ordering {
        self.rocket_id.cmp(&other.rocket_id)
    }
}

impl PartialOrd for RocketLanding {
    fn partial_cmp(&self, other: &RocketLanding) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl RocketLanding {
    /// Construct a new rocket landing.
    pub fn new(rocket_id: UnitID, destination: MapLocation) -> Self {
        RocketLanding {
            rocket_id: rocket_id,
            destination: destination,
        }
    }
}

/// All rocket landings.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RocketLandingInfo {
    landings: FnvHashMap<Rounds, Vec<RocketLanding>>,
}

impl RocketLandingInfo {
    /// Construct an empty rocket landing info.
    pub fn new() -> RocketLandingInfo {
        RocketLandingInfo {
            landings: FnvHashMap::default(),
        }
    }

    /// Add a rocket landing on this round.
    pub fn add_landing(&mut self, round: Rounds, landing: RocketLanding) {
        if !self.landings.contains_key(&round) {
            self.landings.insert(round, vec![]);
        }

        self.landings.get_mut(&round)
                     .expect("landing round should exist")
                     .push(landing);
    }

    /// Add many rocket landings on this round.
    pub fn add_landings(&mut self, round: Rounds, landings: Vec<RocketLanding>) {
        if !self.landings.contains_key(&round) {
            self.landings.insert(round, vec![]);
        }

        self.landings.get_mut(&round)
                     .expect("landing round should exist")
                     .extend(landings);
    }

    /// Get the rocket landings on this round.
    pub fn landings_on(&self, round: Rounds) -> Vec<RocketLanding> {
        if let Some(landings) = self.landings.get(&round) {
            landings.clone()
        } else {
            vec![]
        }
    }

    /// All rocket landings, ordered by round.
    pub fn all(&self) -> Vec<(Rounds, RocketLanding)> {
        let mut all_landings: Vec<(Rounds, RocketLanding)> = vec![];
        for (round, landings) in self.landings.clone().into_iter() {
            all_landings.extend(
                landings.into_iter().map(|landing| (round, landing))
            );
        }
        all_landings.sort();
        all_landings
    }

    /// All rocket landings, grouped by round but in no particular order.
    pub fn all_grouped(&self) -> FnvHashMap<Rounds, Vec<RocketLanding>> {
        self.landings.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::location::Planet;

    #[test]
    fn test_rocket_landings() {
        let mut landings = RocketLandingInfo::new();

        // Add a few landings.
        let loc = MapLocation::new(Planet::Mars, 0, 0);
        let landing = RocketLanding::new(1, loc);
        landings.add_landing(1, landing);
        landings.add_landing(2, landing);
        landings.add_landing(2, landing);
        assert_eq!(landings.landings_on(1).len(), 1);
        assert_eq!(landings.landings_on(2).len(), 2);
        assert_eq!(landings.landings_on(3).len(), 0);

        landings.add_landings(3, vec![landing, landing]);
        assert_eq!(landings.landings_on(3).len(), 2);

        landings.add_landing(3, landing);
        assert_eq!(landings.landings_on(3).len(), 3);

        landings.add_landings(2, vec![landing, landing, landing]);
        assert_eq!(landings.landings_on(2).len(), 5);

        // Check when all landings are returned, they are ordered by round.
        let all = landings.all();
        assert_eq!(all.len(), 9);
        for i in 0..all.len() - 1 {
            assert_lte!(all[i].0, all[i + 1].0);
        }

        // Check all grouped.
        let all_grouped = landings.all_grouped();
        assert_eq!(all_grouped.keys().len(), 3);
        assert_eq!(all_grouped.get(&1).expect("key should exist").len(), 1);
        assert_eq!(all_grouped.get(&2).expect("key should exist").len(), 5);
        assert_eq!(all_grouped.get(&3).expect("key should exist").len(), 3);
    }
}
