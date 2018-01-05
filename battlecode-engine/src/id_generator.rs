//! Generates a sequence of unique pseudorandom positive integer IDS for units.

use rand::{Rng, SeedableRng, StdRng};
use unit::UnitID;

/// The size of groups of IDs to reserve at a time.
const ID_BLOCK_SIZE: usize = 4096;

/// Generates a sequence of unique pseudorandom positive integer IDS for units.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IDGenerator {
    /// Map seed.
    seed: u32,
    /// The block of reserved IDs we walk through.
    reserved_ids: Vec<UnitID>,
    /// Where we are in the current block.
    cursor: usize,
    /// The number at the start of the next block.
    next_id_block: UnitID,
}

impl IDGenerator {
    /// Create a new generator. Generates IDs that are (team as u32) mod 2.
    pub fn new(seed: u32) -> IDGenerator {
        let mut id_gen = IDGenerator {
            seed: seed,
            reserved_ids: vec![0; ID_BLOCK_SIZE],
            cursor: 0,
            next_id_block: ID_BLOCK_SIZE as UnitID,
        };

        id_gen.allocate_next_block();
        id_gen
    }

    /// Return a new ID. Each unit ID is unique.
    pub fn next_id(&mut self) -> UnitID {
        let id = self.reserved_ids[self.cursor];
        self.cursor += 1;

        if self.cursor == ID_BLOCK_SIZE {
            self.allocate_next_block();
        }
        id
    }

    /// Reserve the next ID_BLOCK_SIZE ints after self.next_id_block, shuffle
    /// them, and reset the cursor.
    fn allocate_next_block(&mut self) {
        self.cursor = 0;
        for i in 0..ID_BLOCK_SIZE {
            self.reserved_ids[i] = self.next_id_block + i as UnitID;
        }

        // The seed is a function of the original map seed
        // and the next block size.
        let seed: &[_] = &[(self.seed + self.next_id_block) as usize];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        rng.shuffle(&mut self.reserved_ids);
        self.next_id_block += ID_BLOCK_SIZE as UnitID;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const NUM_IDS: usize = ID_BLOCK_SIZE * 2;

    #[test]
    fn test_id_generator_uniqueness() {
        // Create an ID generator for two teams.
        let mut id_gen = IDGenerator::new(1);

        // Generate a bunch of IDs. All the IDs should be unique.
        let mut ids = HashSet::new();
        for _ in 0..NUM_IDS {
            let id = id_gen.next_id();
            assert!(!ids.contains(&id));
            ids.insert(id);
        }
    }

    #[test]
    fn test_id_generator_determinacy() {
        // ID generators with the same seed produce the same results.
        let mut id_gen_a = IDGenerator::new(1337);
        let mut id_gen_b = IDGenerator::new(1337);
        for _ in 0..NUM_IDS {
            let id_a = id_gen_a.next_id();
            let id_b = id_gen_b.next_id();
            assert_eq!(id_a, id_b);
        }

        // Different seeds produce different results.
        let mut id_gen_a = IDGenerator::new(6147);
        let mut id_gen_b = IDGenerator::new(6370);
        let mut different_results = false;
        for _ in 0..NUM_IDS {
            let id_a = id_gen_a.next_id();
            let id_b = id_gen_b.next_id();
            different_results = different_results || id_a != id_b;
        }
        assert!(different_results)
    }

    #[test]
    fn test_id_generator_mirror() {
        // ID generators should properly produce units with the same IDs
        // in both engine mode and player mode. Thus a cloned ID generator
        // should be in the same state as its original.
        let mut id_gen_manager = IDGenerator::new(1234);
        let mut id_gen_red = id_gen_manager.clone();

        // Red makes some units.
        let mut red_ids: Vec<UnitID> = vec![];
        for _ in 0..NUM_IDS {
            red_ids.push(id_gen_red.next_id());
        }

        // The manager mirrors creating two Red units.
        for i in 0..NUM_IDS {
            let manager_id = id_gen_manager.next_id();
            assert_eq!(manager_id, red_ids[i]);
        }

        // Blue clones the manager's ID generator, and makes some units.
        let mut id_gen_blue = id_gen_manager.clone();
        let mut blue_ids: Vec<UnitID> = vec![];
        for _ in 0..NUM_IDS {
            blue_ids.push(id_gen_blue.next_id());
        }

        // The manager mirrors creating two Blue units.
        for i in 0..NUM_IDS {
            let manager_id = id_gen_manager.next_id();
            assert_eq!(manager_id, blue_ids[i]);
        }
    }
}
