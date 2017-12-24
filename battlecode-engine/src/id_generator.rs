//! Generates a sequence of unique pseudorandom positive integer IDS for units.

use rand;
use rand::Rng;
use unit::UnitID;

/// The size of groups of IDs to reserve at a time.
const ID_BLOCK_SIZE: usize = 4096;

/// Generates a sequence of unique pseudorandom positive integer IDS for units.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDGenerator {
    /// The block of reserved IDs we walk through.
    reserved_ids: Vec<UnitID>,
    /// Where we are in the current block.
    cursor: usize,
    /// The number at the start of the next block.
    next_id_block: UnitID,
}

impl IDGenerator {
    /// Create a new generator.
    pub fn new() -> IDGenerator {
        let mut id_gen = IDGenerator {
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
        id
    }

    /// Reserve the next ID_BLOCK_SIZE ints after self.next_id_block, shuffle
    /// them, and reset the cursor.
    fn allocate_next_block(&mut self) {
        self.cursor = 0;
        for i in 0..ID_BLOCK_SIZE {
            self.reserved_ids[i] = self.next_id_block + i as UnitID;
        }
        rand::thread_rng().shuffle(&mut self.reserved_ids);
        self.next_id_block += ID_BLOCK_SIZE as UnitID;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::io::{self, Write};

    #[test]
    fn id_generator() {
        // Create an ID generator.
        let mut id_gen = super::IDGenerator::new();

        // Generate a bunch of IDs. All the IDs should be unique.
        let mut ids = HashSet::new();
        for i in 0..10000 {
            let id = id_gen.next_id();
            assert![!ids.contains(&id)];
            ids.insert(id);
        }
    }
}
