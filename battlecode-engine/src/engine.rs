//! The core battlecode engine.

use super::schema::Delta;

use engine::Direction::*;

/// Represents a direction from one MapLocation to another.
///
/// Directions for each of the cardinals (north, south, east, west), and each
/// of the diagonals (northwest, southwest, northeast, southeast). There is
/// also a "center" direction, representing no direction.
///
/// Coordinates increase in the North and East directions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Direction {
    North = 0,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,

    // No direction
    Center,
}

impl Direction {
    fn num_to_direction(num: &u8) -> Direction {
        match *num {
            0 => North,
            1 => NorthEast,
            2 => East,
            3 => SouthEast,
            4 => South,
            5 => SouthWest,
            6 => West,
            7 => NorthWest,
            _ => Center,
        }
    }

    /// Returns the (x, y) displacement of this direction.
    pub fn delta(&self) -> (i32, i32) {
        match *self {
            North => (0, -1),
            NorthEast => (1, -1),
            East => (1, 0),
            SouthEast => (1, 1),
            South => (0, 1),
            SouthWest => (-1, 1),
            West => (-1, 0),
            NorthWest => (-1, -1),
            Center => (0, 0),
        }
    }

    /// Returns the direction opposite this one, or Center if it's Center.
    pub fn opposite(&self) -> Direction {
        if *self == Center {
            return Center;
        }
        let new_dir = ((self.clone() as u8) + 4) % 8;
        Direction::num_to_direction(&new_dir)
    }

    /// Returns the direction 45 degrees to the left (counter-clockwise) of
    /// this one, or Center if it's Center.
    pub fn rotate_left(&self) -> Direction {
        if *self == Center {
            return Center;
        }
        let new_dir = ((self.clone() as u8) + 7) % 8;
        Direction::num_to_direction(&new_dir)
    }

    /// Returns the direction 45 degrees to the right (clockwise) of this one,
    /// or Center if it's Center.
    pub fn rotate_right(&self) -> Direction {
        if *self == Center {
            return Center;
        }
        let new_dir = ((self.clone() as u8) + 1) % 8;
        Direction::num_to_direction(&new_dir)
    }
}

/// The planets in the Battlecode world.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Planet {
    Earth,
    Mars,
}

/// Represents two-dimensional coordinates in the Battlecode world.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MapLocation {
    pub x: i32,
    pub y: i32,
    pub planet: Planet,
}

impl MapLocation {
    /// Returns a new MapLocation representing the location with the given
    /// coordinates on the given planet.
    pub fn new(x: i32, y: i32, planet: Planet) -> MapLocation {
        MapLocation { x: x, y: y, planet: planet }
    }

    // TODO: more methods
}

#[cfg(test)]
mod tests {
    use super::Direction::*;

    #[test]
    fn direction_opposite() {
        assert_eq!(North.opposite(), South);
        assert_eq!(NorthEast.opposite(), SouthWest);
        assert_eq!(East.opposite(), West);
        assert_eq!(SouthEast.opposite(), NorthWest);
        assert_eq!(South.opposite(), North);
        assert_eq!(SouthWest.opposite(), NorthEast);
        assert_eq!(West.opposite(), East);
        assert_eq!(NorthWest.opposite(), SouthEast);
        assert_eq!(Center.opposite(), Center);
    }

    #[test]
    fn direction_rotate_left() {
        assert_eq!(North.rotate_left(), NorthWest);
        assert_eq!(NorthEast.rotate_left(), North);
        assert_eq!(East.rotate_left(), NorthEast);
        assert_eq!(SouthEast.rotate_left(), East);
        assert_eq!(South.rotate_left(), SouthEast);
        assert_eq!(SouthWest.rotate_left(), South);
        assert_eq!(West.rotate_left(), SouthWest);
        assert_eq!(NorthWest.rotate_left(), West);
        assert_eq!(Center.rotate_left(), Center);
    }

    #[test]
    fn direction_rotate_right() {
        assert_eq!(North.rotate_right(), NorthEast);
        assert_eq!(NorthEast.rotate_right(), East);
        assert_eq!(East.rotate_right(), SouthEast);
        assert_eq!(SouthEast.rotate_right(), South);
        assert_eq!(South.rotate_right(), SouthWest);
        assert_eq!(SouthWest.rotate_right(), West);
        assert_eq!(West.rotate_right(), NorthWest);
        assert_eq!(NorthWest.rotate_right(), North);
        assert_eq!(Center.rotate_right(), Center);
    }
}