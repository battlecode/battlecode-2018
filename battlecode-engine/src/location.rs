//! Simple data structures to represent locations, directions, and planets.

use failure::Error;
use std::u32;
use location::Direction::*;
use super::error::GameError;
use super::unit::UnitID;

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
        if *self == Center {
            return false;
        }
        (self.clone() as u8 % 2) == 1
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
    pub fn subtract(&self, direction: Direction) -> MapLocation {
        MapLocation {
            planet: self.planet,
            x: self.x - direction.delta().0,
            y: self.y - direction.delta().1,
        }
    }

    /// Returns the location `multiple` squares from this one in the given
    /// direction.
    pub fn add_multiple(&self, direction: Direction,
                        multiple: i32) -> MapLocation {
        MapLocation {
            planet: self.planet,
            x: self.x + multiple * direction.delta().0,
            y: self.y + multiple * direction.delta().1,
        }
    }

    /// Returns the location translated from this location by `dx` in the x
    /// direction and `dy` in the y direction.
    pub fn translate(&self, dx: i32, dy: i32) -> MapLocation {
        MapLocation {
            planet: self.planet,
            x: self.x + dx,
            y: self.y + dy,
        }
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
    /// If the locations are equal this method returns Center. No direction is
    /// returned if the locations are on different planets.
    pub fn direction_to(&self, o: MapLocation) -> Option<Direction> {
        if self.planet != o.planet {
            return None;
        }

        if o.x == self.x && o.y == self.y {
            return Some(Center);
        }

        let dx = (o.x - self.x) as f32;
        let dy = (o.y - self.y) as f32;

        if dx.abs() >= 2.414 * dy.abs() {
            if dx > 0. {
                Some(East)
            } else {
                Some(West)
            }
        } else if dy.abs() >= 2.414 * dx.abs() {
            if dy > 0. {
                Some(North)
            } else {
                Some(South)
            }
        } else {
            if dy > 0. {
                if dx > 0. {
                    Some(Northeast)
                } else {
                    Some(Northwest)
                }
            } else {
                if dx > 0. {
                    Some(Southeast)
                } else {
                    Some(Southwest)
                }
            }
        }
    }

    /// Determines whether this location is adjacent to the specified location,
    /// including diagonally. Note that squares are not adjacent to themselves.
    pub fn is_adjacent_to(&self, o: MapLocation) -> bool {
        self.distance_squared_to(o) <= 2
    }

    /// Returns an array of all locations within a certain radius squared of
    /// this location. (Cannot be called with a radius of greater than 100.)
    ///
    /// The locations are ordered first by the x-coordinate, then the
    /// y-coordinate. The radius squared is inclusive.
    ///
    /// * GameError::IllegalArgument - radius squared is greater than 100
    pub fn all_locations_within(&self, radius_squared: u32)
                                -> Result<Vec<MapLocation>, Error> {
        if radius_squared > 100 {
            Err(GameError::IllegalArgument)?
        }

        let mut locations = vec![];

        let radius = (radius_squared as f32).sqrt() as i32;
        let min_x = self.x - radius;
        let max_x = self.x + radius;
        let min_y = self.y - radius;
        let max_y = self.y + radius;

        for x in min_x..max_x + 1 {
            for y in min_y..max_y + 1 {
                let loc = MapLocation::new(self.planet, x, y);
                if self.distance_squared_to(loc) <= radius_squared {
                    locations.push(loc);
                }
            }
        }

        Ok(locations)
    }
}

/// Any location in the Battlecode world.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Location {
    /// Directly on a square of a planet.
    OnMap(MapLocation),
    /// Inside of the garrison of a rocket or factory, which is either on the
    /// map or in space. The rocket/factory has the given ID.
    InGarrison(UnitID),
    /// Traveling from one planet to another. Only rockets can be in space.
    InSpace,
    /// Somewhere in the great unknown. The location of a unit after it has
    /// died, or before it has been placed in the game.
    Unknown,
}

impl Location {
    /// True if and only if the location is on the map and on this planet.
    pub fn on_planet(&self, planet: Planet) -> bool {
        match *self {
            Location::OnMap(map_loc) => map_loc.planet == planet,
            _ => false,
        }
    }

    /// Whether the unit is on a map.
    pub fn on_map(&self) -> bool {
        match *self {
            Location::OnMap(_) => true,
            _ => false,
        }
    }

    /// The map location of the unit. Errors if the unit is not on a map.
    pub fn map_location(&self) -> Result<MapLocation, Error> {
        match *self {
            Location::OnMap(map_loc) => Ok(map_loc),
            _ => Err(GameError::InvalidLocation)?,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Planet::*;

    #[test]
    fn test_direction() {
        assert_eq!(North.delta(), (0, 1));
        assert_eq!(Northeast.delta(), (1, 1));
        assert_eq!(East.delta(), (1, 0));
        assert_eq!(Southeast.delta(), (1, -1));
        assert_eq!(South.delta(), (0, -1));
        assert_eq!(Southwest.delta(), (-1, -1));
        assert_eq!(West.delta(), (-1, 0));
        assert_eq!(Northwest.delta(), (-1, 1));
        assert_eq!(Center.delta(), (0, 0));

        assert!(!North.is_diagonal());
        assert!(Northeast.is_diagonal());
        assert!(!East.is_diagonal());
        assert!(Southeast.is_diagonal());
        assert!(!South.is_diagonal());
        assert!(Southwest.is_diagonal());
        assert!(!West.is_diagonal());
        assert!(Northwest.is_diagonal());
        assert!(!Center.is_diagonal());

        assert_eq!(North.opposite(), South);
        assert_eq!(Northeast.opposite(), Southwest);
        assert_eq!(East.opposite(), West);
        assert_eq!(Southeast.opposite(), Northwest);
        assert_eq!(South.opposite(), North);
        assert_eq!(Southwest.opposite(), Northeast);
        assert_eq!(West.opposite(), East);
        assert_eq!(Northwest.opposite(), Southeast);
        assert_eq!(Center.opposite(), Center);

        assert_eq!(North.rotate_left(), Northwest);
        assert_eq!(Northeast.rotate_left(), North);
        assert_eq!(East.rotate_left(), Northeast);
        assert_eq!(Southeast.rotate_left(), East);
        assert_eq!(South.rotate_left(), Southeast);
        assert_eq!(Southwest.rotate_left(), South);
        assert_eq!(West.rotate_left(), Southwest);
        assert_eq!(Northwest.rotate_left(), West);
        assert_eq!(Center.rotate_left(), Center);

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
        let loc = MapLocation::new(Earth, 0, 0);
        assert_eq!(loc.add(North),     MapLocation::new(Earth, 0, 1));
        assert_eq!(loc.add(Northeast), MapLocation::new(Earth, 1, 1));
        assert_eq!(loc.add(East),      MapLocation::new(Earth, 1, 0));
        assert_eq!(loc.add(Southeast), MapLocation::new(Earth, 1, -1));
        assert_eq!(loc.add(South),     MapLocation::new(Earth, 0, -1));
        assert_eq!(loc.add(Southwest), MapLocation::new(Earth, -1, -1));
        assert_eq!(loc.add(West),      MapLocation::new(Earth, -1, 0));
        assert_eq!(loc.add(Northwest), MapLocation::new(Earth, -1, 1));
        assert_eq!(loc.add(Center),    MapLocation::new(Earth, 0, 0));

        assert_eq!(loc.subtract(North),     MapLocation::new(Earth, 0, -1));
        assert_eq!(loc.subtract(Northeast), MapLocation::new(Earth, -1, -1));
        assert_eq!(loc.subtract(East),      MapLocation::new(Earth, -1, 0));
        assert_eq!(loc.subtract(Southeast), MapLocation::new(Earth, -1, 1));
        assert_eq!(loc.subtract(South),     MapLocation::new(Earth, 0, 1));
        assert_eq!(loc.subtract(Southwest), MapLocation::new(Earth, 1, 1));
        assert_eq!(loc.subtract(West),      MapLocation::new(Earth, 1, 0));
        assert_eq!(loc.subtract(Northwest), MapLocation::new(Earth, 1, -1));
        assert_eq!(loc.subtract(Center),    MapLocation::new(Earth, 0, 0));

        assert_eq!(loc.add_multiple(Northeast, 10), MapLocation::new(Earth, 10, 10));
        assert_eq!(loc.add_multiple(South, -10), MapLocation::new(Earth, 0, 10));
        assert_eq!(loc.add_multiple(Center, 10), MapLocation::new(Earth, 0, 0));

        assert_eq!(loc.translate(-4, 1), MapLocation::new(Earth, -4, 1));
        assert_eq!(loc.translate(0, -2), MapLocation::new(Earth, 0, -2));
    }

    #[test]
    fn map_location_distance_squared_to() {
        let a = MapLocation::new(Earth, 4, 4);
        let b = MapLocation::new(Earth, 4, 6);
        let c = MapLocation::new(Earth, 7, 4);
        let d = MapLocation::new(Mars, 4, 4);
        assert_eq!(a.distance_squared_to(a), 0);
        assert_eq!(a.distance_squared_to(b), 4);
        assert_eq!(b.distance_squared_to(a), 4);
        assert_eq!(a.distance_squared_to(c), 9);
        assert_eq!(b.distance_squared_to(c), 13);
        assert!(a.distance_squared_to(d) == u32::max_value());
    }

    #[test]
    fn map_location_direction_to() {
        let origin = MapLocation::new(Mars, 0, 0);
        let nn = MapLocation::new(Mars, 0, 2);
        let ne = MapLocation::new(Mars, 2, 2);
        let ee = MapLocation::new(Mars, 2, 0);
        let se = MapLocation::new(Mars, 2, -2);
        assert_eq!(origin.direction_to(nn), Some(North));
        assert_eq!(nn.direction_to(origin), Some(South));
        assert_eq!(origin.direction_to(ne), Some(Northeast));
        assert_eq!(ne.direction_to(origin), Some(Southwest));
        assert_eq!(origin.direction_to(ee), Some(East));
        assert_eq!(ee.direction_to(origin), Some(West));
        assert_eq!(origin.direction_to(se), Some(Southeast));
        assert_eq!(se.direction_to(origin), Some(Northwest));

        // Top right and bottom left quadrant
        let a = MapLocation::new(Mars, 2, 5);
        let b = MapLocation::new(Mars, 3, 4);
        let c = MapLocation::new(Mars, 4, 3);
        let d = MapLocation::new(Mars, 5, 2);
        assert_eq!(origin.direction_to(a), Some(North));
        assert_eq!(a.direction_to(origin), Some(South));
        assert_eq!(origin.direction_to(b), Some(Northeast));
        assert_eq!(b.direction_to(origin), Some(Southwest));
        assert_eq!(origin.direction_to(c), Some(Northeast));
        assert_eq!(c.direction_to(origin), Some(Southwest));
        assert_eq!(origin.direction_to(d), Some(East));
        assert_eq!(d.direction_to(origin), Some(West));

        // Top left and bottom right quadrant
        let a = MapLocation::new(Mars, 5, -2);
        let b = MapLocation::new(Mars, 4, -3);
        let c = MapLocation::new(Mars, 3, -4);
        let d = MapLocation::new(Mars, 2, -5);
        assert_eq!(origin.direction_to(a), Some(East));
        assert_eq!(a.direction_to(origin), Some(West));
        assert_eq!(origin.direction_to(b), Some(Southeast));
        assert_eq!(b.direction_to(origin), Some(Northwest));
        assert_eq!(origin.direction_to(c), Some(Southeast));
        assert_eq!(c.direction_to(origin), Some(Northwest));
        assert_eq!(origin.direction_to(d), Some(South));
        assert_eq!(d.direction_to(origin), Some(North));

        assert_eq!(origin.direction_to(MapLocation::new(Earth, 0, 0)), None,
                   "expect none if locations are on different planets");
    }

    #[test]
    fn map_location_is_adjacent_to() {
        let a = MapLocation::new(Earth, 4, 4);
        let b = MapLocation::new(Earth, 4, 5);
        let c = MapLocation::new(Earth, 5, 5);
        let d = MapLocation::new(Earth, 6, 5);
        let e = MapLocation::new(Mars, 4, 5);
        assert!(a.is_adjacent_to(b));
        assert!(a.is_adjacent_to(c));
        assert!(b.is_adjacent_to(c));
        assert!(d.is_adjacent_to(c));
        assert!(!a.is_adjacent_to(d));
        assert!(!a.is_adjacent_to(e));
        assert!(a.is_adjacent_to(a), "a square is not adjacent to itself");
    }

    #[test]
    fn map_location_all_locations_within() {
        let loc = MapLocation::new(Earth, 2, 4);
        let locs = loc.all_locations_within(16).unwrap();
        assert_eq!(locs.len(), 49, "49 locations within 16 distance squared");
        for new_loc in locs {
            assert_lte!(loc.distance_squared_to(new_loc), 16);
        }
        assert_eq!(loc.all_locations_within(0).unwrap(), vec![loc]);
        assert_err!(loc.all_locations_within(101), GameError::IllegalArgument);
    }
}