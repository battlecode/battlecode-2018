//! Defines constants that affect gameplay.

use super::world::Rounds;

// *********************************
// ****** MAP CONSTANTS ************
// *********************************

/// The minimum possible map height.
pub const MAP_HEIGHT_MIN: usize = 20;

/// The maximum possible map height.
pub const MAP_HEIGHT_MAX: usize = 50;

/// The minumum possible map width.
pub const MAP_WIDTH_MIN: usize = 20;

/// The maxiumum possible map width.
pub const MAP_WIDTH_MAX: usize = 50;

/// The minimum x or y-coordinate.
pub const MAP_COORDINATE_MIN: i32 = -10000;

/// The maximum x or y-coordinate.
pub const MAP_COORDINATE_MAX: i32 = 10000;

/// The minimum starting Karbonite deposit on Earth.
pub const MAP_KARBONITE_MIN: u32 = 0;

/// The maximum starting Karbonite deposit on Earth.
pub const MAP_KARBONITE_MAX: u32 = 50;

// *********************************
// ****** WEATHER CONSTANTS ********
// *********************************

/// The minimum number of rounds since the last asteroid strike.
pub const ASTEROID_ROUND_MIN: Rounds = 2;

/// The maximum number of rounds since the last asteroid strike.
pub const ASTEROID_ROUND_MAX: Rounds = 20;

/// The minimum karbonite in an asteroid strike.
pub const ASTEROID_KARB_MIN: u32 = 20;

/// The maximum karbonite in an asteroid strike.
pub const ASTEROID_KARB_MAX: u32 = 200;

/// The minimum flight time due to the orbit.
pub const ORBIT_FLIGHT_MIN: i32 = 100;

/// The maximum flight time due to the orbit.
pub const ORBIT_FLIGHT_MAX: i32 = 400;

// *********************************
// ****** RESEARCH CONSTANTS *******
// *********************************

/// The cost of each level of research on the Worker branch.
pub const WORKER_COST: [Rounds; 5] = [0, 60, 80, 100, 200];

/// The cost of each level of research on the Knight branch.
pub const KNIGHT_COST: [Rounds; 4] = [0, 80, 100, 200];

/// The cost of each level of research on the Ranger branch.
pub const RANGER_COST: [Rounds; 4] = [0, 80, 100, 200];

/// The cost of each level of research on the Mage branch.
pub const MAGE_COST: [Rounds; 5] = [0, 60, 80, 100, 200];

/// The cost of each level of research on the Healer branch.
pub const HEALER_COST: [Rounds; 4] = [0, 80, 100, 200];

/// The cost of each level of research on the Factory branch.
pub const FACTORY_COST: [Rounds; 1] = [0];

/// The cost of each level of research on the Rocket branch.
pub const ROCKET_COST: [Rounds; 4] = [0, 300, 200, 200];

// *********************************
// ****** UNIT CONSTANTS ***********
// *********************************

/// The heat each robot dissipates per round.
pub const HEAT_LOSS_PER_ROUND: u32 = 10;

/// The robot must have less than this amount of heat to perform
/// actions corresponding to that heat.
pub const MAX_HEAT_TO_ACT: u32 = 10;

/// The damage a rocket deals to adjacent units upon landing.
pub const ROCKET_BLAST_DAMAGE: u32 = 50;

// *********************************
// ****** GAME PARAMETERS **********
// *********************************

/// The round at which the game is forced to end
pub const ROUND_LIMIT: Rounds = 1000;

/// The length of the communication array, in bytes
pub const COMMUNICATION_ARRAY_LENGTH: usize = 100;

/// The communication delay between planets, in rounds
pub const COMMUNICATION_DELAY: usize = 200;
