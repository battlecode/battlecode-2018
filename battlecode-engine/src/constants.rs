//! Defines constants that affect gameplay.

// *********************************
// ****** MAP CONSTANTS ************
// *********************************

/// The minimum possible map height.
pub const MAP_MIN_HEIGHT: usize = 20;

/// The maximum possible map height.
pub const MAP_MAX_HEIGHT: usize = 50;

/// The minumum possible map width.
pub const MAP_MIN_WIDTH: usize = 20;

/// The maxiumum possible map width.
pub const MAP_MAX_WIDTH: usize = 50;

/// The minimum x or y-coordinate.
pub const MAP_MIN_COORDINATE: i32 = -10000;

/// The maximum x or y-coordinate.
pub const MAP_MAX_COORDINATE: i32 = 10000;

/// The minimum starting Karbonite deposit on Earth.
pub const MAP_KARBONITE_MIN: i32 = 0;

/// The maximum starting Karbonite deposit on Earth.
pub const MAP_KARBONITE_MAX: i32 = 50;

/// The minimum number of rounds since the last asteroid strike.
pub const MAP_ASTEROID_ROUND_MIN: i32 = 2;

/// The maximum number of rounds since the last asteroid strike.
pub const MAP_ASTEROID_ROUND_MAX: i32 = 20;

/// The minimum karbonite in an asteroid strike.
pub const MAP_ASTEROID_KARB_MIN: i32 = 20;

/// The maximum karbonite in an asteroid strike.
pub const MAP_ASTEROID_KARB_MAX: i32 = 200;

// *********************************
// ****** GAME PARAMETERS **********
// *********************************

/// The round at which the game is forced to end
pub const ROUND_LIMIT: usize = 3000;

/// The length of the communication array, in bytes
pub const COMMUNICATION_ARRAY_LENGTH: usize = 100;

/// The communication delay between planets, in rounds
pub const COMMUNICATION_DELAY: usize = 200;
