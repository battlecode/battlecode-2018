//! Defines constants that affect gameplay.

use super::world::Rounds;

// *********************************
// ******* GAME CONSTANTS **********
// *********************************

/// The round at which the game is forced to end
pub const ROUND_LIMIT: Rounds = 1000;

// *********************************
// *** COMMUNICATION CONSTANTS *****
// *********************************

/// The length of the communication array, in bytes
pub const COMMUNICATION_ARRAY_LENGTH: usize = 100;

/// The communication delay between planets, in rounds
pub const COMMUNICATION_DELAY: usize = 50;

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

/// The minimum starting Karbonite deposit on Earth.
pub const MAP_KARBONITE_MIN: u32 = 0;

/// The maximum starting Karbonite deposit on Earth.
pub const MAP_KARBONITE_MAX: u32 = 50;

// *********************************
// ****** WEATHER CONSTANTS ********
// *********************************

/// The minimum karbonite in an asteroid strike.
pub const ASTEROID_KARB_MIN: u32 = 20;

/// The maximum karbonite in an asteroid strike.
pub const ASTEROID_KARB_MAX: u32 = 200;

/// The minimum flight time due to the orbit.
pub const ORBIT_FLIGHT_MIN: u32 = 50;

/// The maximum flight time due to the orbit.
pub const ORBIT_FLIGHT_MAX: u32 = 200;

/// The minimum period of the orbit pattern.
pub const ORBIT_PERIOD_MIN: u32 = 100;

/// The maximum period of the orbit pattern.
pub const ORBIT_PERIOD_MAX: u32 = 500;

/// At the start of this round, all units on Earth are destroyed.
pub const APOCALYPSE_ROUND: Rounds = 750;

// *********************************
// ***** KARBONITE CONSTANTS *******
// *********************************

/// The starting amount of karbonite per team.
pub const KARBONITE_STARTING: u32 = 100;
/// The base amount of karbonite gained per turn.
pub const KARBONITE_PER_ROUND: u32 = 10;
/// The karbonite per round is decreased by 1 karbonite for every
/// KARBONITE_DECREASE_RATIO karbonite in the stockpile.
pub const KARBONITE_DECREASE_RATIO: u32 = 40;

// *********************************
// ****** RESEARCH CONSTANTS *******
// *********************************

/// The cost of each level of research on the Worker branch.
pub const RESEARCH_WORKER_COST: [Rounds; 5] = [0, 25, 75, 75, 75];

/// The cost of each level of research on the Knight branch.
pub const RESEARCH_KNIGHT_COST: [Rounds; 4] = [0, 25, 75, 100];

/// The cost of each level of research on the Ranger branch.
pub const RESEARCH_RANGER_COST: [Rounds; 4] = [0, 25, 100, 200];

/// The cost of each level of research on the Mage branch.
pub const RESEARCH_MAGE_COST: [Rounds; 5] = [0, 25, 75, 100, 75];

/// The cost of each level of research on the Healer branch.
pub const RESEARCH_HEALER_COST: [Rounds; 4] = [0, 25, 100, 100];

/// The cost of each level of research on the Factory branch.
pub const RESEARCH_FACTORY_COST: [Rounds; 1] = [0];

/// The cost of each level of research on the Rocket branch.
pub const RESEARCH_ROCKET_COST: [Rounds; 4] = [0, 50, 100, 100];

// *********************************
// ****** HEAT CONSTANTS ***********
// *********************************

/// The heat each robot dissipates per round.
pub const HEAT_LOSS_PER_ROUND: u32 = 10;

/// The robot must have less than this amount of heat to perform
/// actions corresponding to that heat.
pub const MAX_HEAT_TO_ACT: u32 = 10;

// *********************************
// ********** UNIT COSTS ***********
// *********************************

/// The cost of a worker in a factory.
pub const FACTORY_WORKER_COST: u32 = 50;
/// The cost of a knight in a factory.
pub const FACTORY_KNIGHT_COST: u32 = 40;
/// The cost of a ranger in a factory.
pub const FACTORY_RANGER_COST: u32 = 40;
/// The cost of a mage in a factory.
pub const FACTORY_MAGE_COST: u32 = 40;
/// The cost of a healer in a factory.
pub const FACTORY_HEALER_COST: u32 = 40;
/// The cost to blueprint a factory.
pub const BLUEPRINT_FACTORY_COST: u32 = 200;
/// The cost to blueprint a rocket.
pub const BLUEPRINT_ROCKET_COST: u32 = 150;
/// The cost to replicate a worker.
pub const REPLICATE_WORKER_COST: u32 = 60;
