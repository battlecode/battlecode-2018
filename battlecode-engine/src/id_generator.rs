//! Generates a sequence of unique pseudorandom positive integer IDS for units.
//!
//! Algorithm from:
//! http://preshing.com/20121224/how-to-generate-a-sequence-of-unique-random-integers/

use unit::UnitID;

/// IDs generated are less than p, which is a prime such that p = 3 mod 4. This
/// is the closest prime number less than 2^16 that satisifes this condition.
const PRIME: u16 = 65519;
const XOR_VALUE: u16 = 0xc0d3;

/// IDs less than or equal to the maximum reserved ID are reserved for
/// initial units or testing units.
const MAX_RESERVED_ID: UnitID = 10;

/// Generates a sequence of unique pseudorandom positive integer IDS for units.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IDGenerator {
    /// Map seed.
    seed: u16,
    /// An index into the random number generator. Must not be 0 or 1.
    index: u16,
}

fn permute_qpr(x: UnitID) -> UnitID {
    // The integers out of range are mapped to themselves.
    if x >= PRIME {
        return x;
    }
    let residue = ((x as u32 * x as u32) % PRIME as u32) as u16;
    if x <= PRIME / 2 {
        residue
    } else {
        PRIME - residue
    }
}

impl IDGenerator {
    /// Create a new generator.
    pub fn new(seed: u16) -> IDGenerator {
        IDGenerator {
            seed: seed,
            index: 2,
        }
    }

    /// Return a new ID. Each unit ID is unique.
    /// Does not produce IDs in the range [0, MAX_RESERVED_ID].
    pub fn next_id(&mut self) -> UnitID {
        let mut id = self.next_id_unchecked();
        while id <= MAX_RESERVED_ID {
            id = self.next_id_unchecked();
        }
        id
    }

    fn next_id_unchecked(&mut self) -> UnitID {
        let x = self.index;
        self.index += 1;
        permute_qpr(permute_qpr(x).wrapping_add(self.seed) ^ XOR_VALUE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const NUM_IDS: usize = 20000;

    #[test]
    fn test_id_generator_uniqueness() {
        let test_seeds: [u16; 3] = [1, 2, 888];
        for seed in test_seeds.iter() {
            // Create an ID generator.
            let mut id_gen = IDGenerator::new(*seed);

            // Generate a bunch of IDs. All the IDs should be unique.
            let mut ids = HashSet::new();
            for i in 0..NUM_IDS {
                let id = id_gen.next_id();
                assert!(!ids.contains(&id), "failed at the {}th ID", i);
                ids.insert(id);
            }
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
