//! Simple data structures to represent locations, directions, and planets.

use std::u32;
use location::Direction::*;

/// A direction from one MapLocation to another.
///
/// Directions for each of the cardinals (north, south, east, west), and each
/// of the diagonals (northwest, southwest, northeast, southeast). There is
/// also a "center" direction, representing no direction.
///
/// Coordinates increase in the north and east directions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Direction {
    North = 0,
    Northeast,
    East,
    Southeast,
    South,
    Southwest,
    West,
    Northwest,

    // No direction
    Center,
}

impl Direction {
    fn num_to_direction(num: u8) -> Direction {
        match num {
            0 => North,
            1 => Northeast,
            2 => East,
            3 => Southeast,
            4 => South,
            5 => Southwest,
            6 => West,
            7 => Northwest,
            _ => Center,
        }
    }

    /// Returns all the directions ordered clockwise, starting with north and
    /// not including the center direction.
    pub fn all() -> Vec<Direction> {
        vec![
            North,
            Northeast,
            East,
            Southeast,
            South,
            Southwest,
            West,
            Northwest
        ]
    }

    /// Returns the (x, y) displacement of this direction.
    pub fn delta(&self) -> (i32, i32) {
        match *self {
            North => (0, 1),
            Northeast => (1, 1),
            East => (1, 0),
            Southeast => (1, -1),
            South => (0, -1),
            Southwest => (-1, -1),
            West => (-1, 0),
            Northwest => (-1, 1),
            Center => (0, 0),
        }
    }

    /// Whether this direction is a diagonal one.
    pub fn is_diagonal(&self) -> bool {
        unimplemented!();
    }

    /// Returns the direction opposite this one, or Center if it's Center.
    pub fn opposite(&self) -> Direction {
        if *self == Center {
            return Center;
        }
        let new_dir = ((self.clone() as u8) + 4) % 8;
        Direction::num_to_direction(new_dir)
    }

    /// Returns the direction 45 degrees to the left (counter-clockwise) of
    /// this one, or Center if it's Center.
    pub fn rotate_left(&self) -> Direction {
        if *self == Center {
            return Center;
        }
        let new_dir = ((self.clone() as u8) + 7) % 8;
        Direction::num_to_direction(new_dir)
    }

    /// Returns the direction 45 degrees to the right (clockwise) of this one,
    /// or Center if it's Center.
    pub fn rotate_right(&self) -> Direction {
        if *self == Center {
            return Center;
        }
        let new_dir = ((self.clone() as u8) + 1) % 8;
        Direction::num_to_direction(new_dir)
    }
}

/// The planets in the Battlecode world.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Planet {
    Earth,
    Mars,
}

/// Two-dimensional coordinates in the Battlecode world.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MapLocation {
    pub planet: Planet,
    pub x: i32,
    pub y: i32,
}

impl MapLocation {
    /// Returns a new MapLocation representing the location with the given
    /// coordinates on a planet.
    pub fn new(planet: Planet, x: i32, y: i32) -> MapLocation {
        MapLocation { planet: planet, x: x, y: y }
    }

    /// Returns the location one square from this one in the given direction.
    pub fn add(&self, direction: Direction) -> MapLocation {
        MapLocation { 
            planet: self.planet,
            x: self.x + direction.delta().0, 
            y: self.y + direction.delta().1,
        }
    }

    /// Returns the location one square from this one in the opposite direction.
    pub fn subtract(&self, _direction: Direction) -> MapLocation {
        unimplemented!();
    }

    /// Returns the location `multiple` squares from this one in the given
    /// direction.
    pub fn add_multiple(&self, _direction: Direction,
                        _multiple: i32) -> MapLocation {
        unimplemented!();
    }

    /// Returns the location translated from this location by `dx` in the x
    /// direction and `dy` in the y direction.
    pub fn translate(&self, _dx: i32, _dy: i32) -> MapLocation {
        unimplemented!();
    }

    /// Computes the square of the distance from this location to the specified
    /// location. If on different planets, returns the maximum integer.
    pub fn distance_squared_to(&self, o: MapLocation) -> u32 {
        if self.planet == o.planet {
            ((self.x - o.x) * (self.x - o.x) + (self.y - o.y) * (self.y - o.y)) as u32
        } else {
            u32::max_value()
        }
    }

    /// Returns the Direction from this location to the specified location.
    /// If the locations are equal this method returns Center.
    pub fn direction_to(&self, _o: MapLocation) -> Direction {
        unimplemented!();
    }

    /// Determines whether this location is adjacent to the specified location,
    /// including diagonally. Note that squares are not adjacent to themselves.
    pub fn is_adjacent_to(&self, o: MapLocation) -> bool {
        self.distance_squared_to(o) <= 2
    }

    /// Returns an array of all locations within a certain radius squared of
    /// this location. (Cannot be called with a radius of greater than 100.)
    pub fn all_locations_within(&self, _radius_squared: u32) -> Vec<MapLocation> {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {
    use std::u32;
    use super::Direction::*;
    use super::MapLocation;
    use super::Planet;

    #[test]
    fn direction_opposite() {
        assert_eq!(North.opposite(), South);
        assert_eq!(Northeast.opposite(), Southwest);
        assert_eq!(East.opposite(), West);
        assert_eq!(Southeast.opposite(), Northwest);
        assert_eq!(South.opposite(), North);
        assert_eq!(Southwest.opposite(), Northeast);
        assert_eq!(West.opposite(), East);
        assert_eq!(Northwest.opposite(), Southeast);
        assert_eq!(Center.opposite(), Center);
    }

    #[test]
    fn direction_rotate_left() {
        assert_eq!(North.rotate_left(), Northwest);
        assert_eq!(Northeast.rotate_left(), North);
        assert_eq!(East.rotate_left(), Northeast);
        assert_eq!(Southeast.rotate_left(), East);
        assert_eq!(South.rotate_left(), Southeast);
        assert_eq!(Southwest.rotate_left(), South);
        assert_eq!(West.rotate_left(), Southwest);
        assert_eq!(Northwest.rotate_left(), West);
        assert_eq!(Center.rotate_left(), Center);
    }

    #[test]
    fn direction_rotate_right() {
        assert_eq!(North.rotate_right(), Northeast);
        assert_eq!(Northeast.rotate_right(), East);
        assert_eq!(East.rotate_right(), Southeast);
        assert_eq!(Southeast.rotate_right(), South);
        assert_eq!(South.rotate_right(), Southwest);
        assert_eq!(Southwest.rotate_right(), West);
        assert_eq!(West.rotate_right(), Northwest);
        assert_eq!(Northwest.rotate_right(), North);
        assert_eq!(Center.rotate_right(), Center);
    }

    #[test]
    fn map_location_add() {
        let loc = MapLocation { planet: Planet::Earth, x: 0, y: 0 };
        assert_eq!(loc.add(North),      MapLocation { planet: Planet::Earth, x: 0, y: 1 });
        assert_eq!(loc.add(Northeast),  MapLocation { planet: Planet::Earth, x: 1, y: 1 });
        assert_eq!(loc.add(East),       MapLocation { planet: Planet::Earth, x: 1, y: 0 });
        assert_eq!(loc.add(Southeast),  MapLocation { planet: Planet::Earth, x: 1, y: -1 });
        assert_eq!(loc.add(South),      MapLocation { planet: Planet::Earth, x: 0, y: -1 });
        assert_eq!(loc.add(Southwest),  MapLocation { planet: Planet::Earth, x: -1, y: -1 });
        assert_eq!(loc.add(West),       MapLocation { planet: Planet::Earth, x: -1, y: 0 });
        assert_eq!(loc.add(Northwest),  MapLocation { planet: Planet::Earth, x: -1, y: 1 });
        assert_eq!(loc.add(Center),     MapLocation { planet: Planet::Earth, x: 0, y: 0 });
    }

    #[test]
    fn map_location_distance_squared_to() {
        let a = MapLocation::new(Planet::Earth, 4, 4);
        let b = MapLocation::new(Planet::Earth, 4, 6);
        let c = MapLocation::new(Planet::Earth, 7, 4);
        let d = MapLocation::new(Planet::Mars, 4, 4);
        assert_eq!(a.distance_squared_to(a), 0);
        assert_eq!(a.distance_squared_to(b), 4);
        assert_eq!(b.distance_squared_to(a), 4);
        assert_eq!(a.distance_squared_to(c), 9);
        assert_eq!(b.distance_squared_to(c), 13);
        assert!(a.distance_squared_to(d) == u32::max_value());
    }

    #[test]
    fn map_location_is_adjacent_to() {
        let a = MapLocation::new(Planet::Earth, 4, 4);
        let b = MapLocation::new(Planet::Earth, 4, 5);
        let c = MapLocation::new(Planet::Earth, 5, 5);
        let d = MapLocation::new(Planet::Earth, 6, 5);
        let e = MapLocation::new(Planet::Mars, 4, 5);
        assert!(a.is_adjacent_to(b));
        assert!(a.is_adjacent_to(c));
        assert!(b.is_adjacent_to(c));
        assert!(d.is_adjacent_to(c));
        assert!(!a.is_adjacent_to(d));
        assert!(!a.is_adjacent_to(e));
    }
}