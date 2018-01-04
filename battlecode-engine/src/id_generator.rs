//! Generates a sequence of unique pseudorandom positive integer IDS for units.

use rand::{Rng, SeedableRng, StdRng};
use unit::UnitID;
use world::Team;

/// The size of groups of IDs to reserve at a time.
const ID_BLOCK_SIZE: usize = 4096;

/// Generates a sequence of unique pseudorandom positive integer IDS for units.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct IDGenerator {
    /// IDs generated are modulo mod 2.
    modulo: u32,
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
    pub fn new(team: Team, seed: u32) -> IDGenerator {
        let mut id_gen = IDGenerator {
            modulo: team as u32,
            seed: seed,
            reserved_ids: vec![0; ID_BLOCK_SIZE as usize],
            cursor: 0,
            next_id_block: 0,
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
        id * 2 + self.modulo
    }

    /// Reserve the next ID_BLOCK_SIZE ints after self.next_id_block, shuffle
    /// them, and reset the cursor.
    fn allocate_next_block(&mut self) {
        self.cursor = 0;
        for i in 0..ID_BLOCK_SIZE {
            self.reserved_ids[i] = self.next_id_block + i as UnitID;
        }

        // The seed is a function of the original map seed, the team of the
        // ID generator, and the next block size.
        let seed: &[_] = &[(self.seed + self.next_id_block + self.modulo) as usize];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        rng.shuffle(&mut self.reserved_ids);
        self.next_id_block += ID_BLOCK_SIZE as UnitID;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use super::super::world::Team;

    #[test]
    fn uniqueness() {
        // Create an ID generator for two teams.
        let mut id_gen_a = super::IDGenerator::new(Team::Red, 1);
        let mut id_gen_b = super::IDGenerator::new(Team::Blue, 1);

        // Generate a bunch of IDs. All the IDs should be unique.
        // Also all red IDs are even, and blue IDs are odd.
        let mut ids = HashSet::new();
        for _ in 0..super::ID_BLOCK_SIZE * 2 {
            let id_a = id_gen_a.next_id();
            assert!(!ids.contains(&id_a));
            assert_eq!(id_a % 2, 0);
            ids.insert(id_a);

            let id_b = id_gen_b.next_id();
            assert!(!ids.contains(&id_b));
            assert_eq!(id_b % 2, 1);
            ids.insert(id_b);
        }
    }

    #[test]
    fn determinacy() {
        // ID generators with the same seed and team produce the same results.
        let mut id_gen_a = super::IDGenerator::new(Team::Red, 1);
        let mut id_gen_b = super::IDGenerator::new(Team::Red, 1);
        for _ in 0..super::ID_BLOCK_SIZE * 2 {
            let id_a = id_gen_a.next_id();
            let id_b = id_gen_b.next_id();
            assert_eq!(id_a, id_b);
        }

        // Different seeds produce different results.
        let mut id_gen_a = super::IDGenerator::new(Team::Red, 1);
        let mut id_gen_b = super::IDGenerator::new(Team::Red, 2);
        let mut different_results = false;
        for _ in 0..super::ID_BLOCK_SIZE * 2 {
            let id_a = id_gen_a.next_id();
            let id_b = id_gen_b.next_id();
            different_results = different_results || id_a != id_b;
        }
        assert!(different_results)
    }
}
