//! The communication arrays for a single team.
//! 
//! A team has a different array on each planet. A player can read the most
//! recent team array for its planet, but reads a delayed version of the team
//! array for the other planet.

use failure::Error;
use fnv::FnvHashMap;
use std::collections::VecDeque;

use super::constants::*;
use super::location::*;
use super::error::*;

/// A team-shared communication array for a single player.
pub type TeamArray = Vec<i32>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TeamArrayInfo {
    history: FnvHashMap<Planet, VecDeque<TeamArray>>,
}

impl TeamArrayInfo {
    /// Constructs a new team array info.
    pub fn new() -> TeamArrayInfo {
        // The length of the history is COMMUNICATION_DELAY + 1 for each array
        // from 1 to COMMUNICATION_DELAY rounds ago, and the current round.
        let mut history: FnvHashMap<Planet, VecDeque<TeamArray>> = FnvHashMap::default();
        let mut deque: VecDeque<TeamArray> = VecDeque::with_capacity(COMMUNICATION_DELAY + 1);
        for _ in 0..COMMUNICATION_DELAY + 1 {
            deque.push_back(vec![0; COMMUNICATION_ARRAY_LENGTH]);
        }
        history.insert(Planet::Earth, deque.clone());
        history.insert(Planet::Mars, deque);
        TeamArrayInfo {
            history: history,
        }
    }

    fn get_arrays(&self, planet: Planet) -> &VecDeque<TeamArray> {
        if let Some(array) = self.history.get(&planet) {
            array
        } else {
            unreachable!();
        }
    }

    fn get_arrays_mut(&mut self, planet: Planet) -> &mut VecDeque<TeamArray> {
        if let Some(array) = self.history.get_mut(&planet) {
            array
        } else {
            unreachable!();
        }
    }

    /// Filters the team array history such that the new history does not store
    /// any intermediate arrays, and only the arrays that can be accessed on
    /// this planet.
    pub fn filter(&self, planet: Planet) -> TeamArrayInfo {
        let this_array = self.get_arrays(planet).front().unwrap().clone();
        let that_array = self.get_arrays(planet.other()).back().unwrap().clone();
        let mut history: FnvHashMap<Planet, VecDeque<TeamArray>> = FnvHashMap::default();

        let mut this_deque: VecDeque<TeamArray> = VecDeque::with_capacity(1);
        let mut that_deque: VecDeque<TeamArray> = VecDeque::with_capacity(1);
        this_deque.push_back(this_array);
        that_deque.push_back(that_array);

        history.insert(planet, this_deque);
        history.insert(planet.other(), that_deque);
        TeamArrayInfo {
            history: history,
        }
    }

    /// Get the most recent version of this planet's team array.
    pub fn first_array(&self, planet: Planet) -> &TeamArray {
        self.get_arrays(planet).front().unwrap()
    }

    /// Get the oldest version of this planet's team array.
    pub fn last_array(&self, planet: Planet) -> &TeamArray {
        self.get_arrays(planet).back().unwrap()
    }

    /// Writes the value at the index of this planet's team array.
    /// Errors if the array written to is accessed out of bounds.
    pub fn write(&mut self, planet: Planet, index: usize, value: i32) -> Result<(), Error> {
        let array = self.get_arrays_mut(planet).front_mut().unwrap();
        if index < array.len() {
            array[index] = value;
            Ok(())
        } else {
            Err(GameError::ArrayOutOfBounds)?
        }
    }

    /// Ends the round by discarding the oldest version of each planet's team
    /// array in favor of another version that is a round more recent.
    pub fn end_round(&mut self) {
        let array = self.get_arrays(Planet::Earth).front().unwrap().clone();
        self.get_arrays_mut(Planet::Earth).pop_back();
        self.get_arrays_mut(Planet::Earth).push_front(array);

        let array = self.get_arrays(Planet::Mars).front().unwrap().clone();
        self.get_arrays_mut(Planet::Mars).pop_back();
        self.get_arrays_mut(Planet::Mars).push_front(array);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::location::Planet;

    #[test]
    fn test_array_read() {
        let team_arrays = TeamArrayInfo::new();
        let earth_arrays = team_arrays.filter(Planet::Earth);
        let mars_arrays = team_arrays.filter(Planet::Mars);
        let arrays = vec![team_arrays, earth_arrays, mars_arrays];

        for arr in arrays {
            let earth_first = arr.first_array(Planet::Earth);
            let earth_last = arr.last_array(Planet::Earth);
            let mars_first = arr.first_array(Planet::Mars);
            let mars_last = arr.last_array(Planet::Mars);

            // All arrays are the right length.
            assert_eq!(earth_first.len(), COMMUNICATION_ARRAY_LENGTH);
            assert_eq!(earth_last.len(), COMMUNICATION_ARRAY_LENGTH);
            assert_eq!(mars_first.len(), COMMUNICATION_ARRAY_LENGTH);
            assert_eq!(mars_last.len(), COMMUNICATION_ARRAY_LENGTH);

            // Both planet arrays are initialized with all 0's.
            for index in 0..COMMUNICATION_ARRAY_LENGTH {
                assert_eq!(earth_first[index], 0);
                assert_eq!(earth_last[index], 0);
                assert_eq!(mars_first[index], 0);
                assert_eq!(mars_last[index], 0);
            }
        }
    }

    #[test]
    fn test_array_write() {
        let mut arrays = TeamArrayInfo::new();

        // Writing is OK.
        for index in 0..COMMUNICATION_ARRAY_LENGTH {
            assert!(arrays.write(Planet::Earth, index, index as i32).is_ok());
            assert!(arrays.write(Planet::Mars, index, index as i32).is_ok());
        }

        // Read the values you wrote..
        let earth_first = arrays.first_array(Planet::Earth).clone();
        let earth_last = arrays.last_array(Planet::Earth).clone();
        let mars_first = arrays.first_array(Planet::Mars).clone();
        let mars_last = arrays.last_array(Planet::Mars).clone();
        for index in 0..COMMUNICATION_ARRAY_LENGTH {
            assert_eq!(earth_first[index], index as i32);
            assert_eq!(earth_last[index], 0);
            assert_eq!(mars_first[index], index as i32);
            assert_eq!(mars_last[index], 0);
        }

        // Error when writing out of bounds.
        let oob_index = COMMUNICATION_ARRAY_LENGTH;
        assert_err!(arrays.write(Planet::Earth, oob_index, 0), GameError::ArrayOutOfBounds);
        assert_err!(arrays.write(Planet::Mars, oob_index, 0), GameError::ArrayOutOfBounds);
    }

    #[test]
    fn test_array_end_round_filter() {
        // On the i-th round, Earth writes 1 and Mars writes 2 to index i.
        // Write for just enough rounds that you can't see what the other
        // planet has written yet.
        let mut arrays = TeamArrayInfo::new();
        for round in 0..COMMUNICATION_DELAY - 1 {
            assert!(arrays.write(Planet::Earth, round, 1).is_ok());
            assert!(arrays.write(Planet::Mars, round, 2).is_ok());
            arrays.end_round()
        }

        // We can read the values we wrote, but the other planet is all 0's.
        let earth = arrays.filter(Planet::Earth);
        let mars = arrays.filter(Planet::Mars);
        for index in 0..COMMUNICATION_DELAY - 1 {
            assert_eq!(earth.first_array(Planet::Earth)[index], 1);
            assert_eq!(earth.last_array(Planet::Mars)[index], 0);
            assert_eq!(mars.last_array(Planet::Earth)[index], 0);
            assert_eq!(mars.first_array(Planet::Mars)[index], 2);
        }
        for index in COMMUNICATION_DELAY - 1..COMMUNICATION_ARRAY_LENGTH {
            assert_eq!(earth.first_array(Planet::Earth)[index], 0);
            assert_eq!(earth.last_array(Planet::Mars)[index], 0);
            assert_eq!(mars.last_array(Planet::Earth)[index], 0);
            assert_eq!(mars.first_array(Planet::Mars)[index], 0);
        }

        // Write one more time.
        let round = COMMUNICATION_DELAY - 1;
        assert!(arrays.write(Planet::Earth, round, 1).is_ok());
        assert!(arrays.write(Planet::Mars, round, 2).is_ok());
        arrays.end_round();

        // Read the new integer.
        let earth = arrays.filter(Planet::Earth);
        let mars = arrays.filter(Planet::Mars);
        assert_eq!(earth.first_array(Planet::Earth)[round], 1);
        assert_eq!(mars.first_array(Planet::Mars)[round], 2);

        // And read the integers from the other planet.
        assert_eq!(earth.first_array(Planet::Mars)[0], 2);
        assert_eq!(mars.first_array(Planet::Earth)[0], 1);
        for index in 1..COMMUNICATION_ARRAY_LENGTH {
            assert_eq!(earth.last_array(Planet::Mars)[index], 0);
            assert_eq!(mars.last_array(Planet::Earth)[index], 0);
        }
    }
}
