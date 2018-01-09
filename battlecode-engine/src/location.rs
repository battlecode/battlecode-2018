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
    Northeast = 1,
    East = 2,
    Southeast = 3,
    South = 4,
    Southwest = 5,
    West = 6,
    Northwest = 7,

    // No direction
    Center = 8,
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

    /// Returns the x displacement of this direction.
    pub fn dx(&self) -> i32 {
        match *self {
            North => 0,
            Northeast => 1,
            East => 1,
            Southeast => 1,
            South => 0,
            Southwest => -1,
            West => -1,
            Northwest => -1,
            Center => 0,
        }
    }

    /// Returns the y displacement of this direction.
    pub fn dy(&self) -> i32 {
        match *self {
            North => 1,
            Northeast => 1,
            East => 0,
            Southeast => -1,
            South => -1,
            Southwest => -1,
            West => 0,
            Northwest => 1,
            Center => 0,
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
        let new_dir = ((*self as u8) + 4) % 8;
        Direction::num_to_direction(new_dir)
    }

    /// Returns the direction 45 degrees to the left (counter-clockwise) of
    /// this one, or Center if it's Center.
    pub fn rotate_left(&self) -> Direction {
        if *self == Center {
            return Center;
        }
        let new_dir = ((*self as u8) + 7) % 8;
        Direction::num_to_direction(new_dir)
    }

    /// Returns the direction 45 degrees to the right (clockwise) of this one,
    /// or Center if it's Center.
    pub fn rotate_right(&self) -> Direction {
        if *self == Center {
            return Center;
        }
        let new_dir = ((*self as u8) + 1) % 8;
        Direction::num_to_direction(new_dir)
    }
}

/// The planets in the Battlecode world.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Planet {
    Earth = 0,
    Mars = 1,
}

impl Planet {
    /// The other planet.
    pub fn other(&self) -> Planet {
        match *self {
            Planet::Earth => Planet::Mars,
            Planet::Mars => Planet::Earth,
        }
    }
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
            x: self.x + direction.dx(), 
            y: self.y + direction.dy(),
        }
    }

    /// Returns the location one square from this one in the opposite direction.
    pub fn subtract(&self, direction: Direction) -> MapLocation {
        MapLocation {
            planet: self.planet,
            x: self.x - direction.dx(),
            y: self.y - direction.dy(),
        }
    }

    /// Returns the location `multiple` squares from this one in the given
    /// direction.
    pub fn add_multiple(&self, direction: Direction,
                        multiple: i32) -> MapLocation {
        MapLocation {
            planet: self.planet,
            x: self.x + multiple * direction.dx(),
            y: self.y + multiple * direction.dy(),
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
    /// If the locations are equal this method returns Center.
    ///
    /// * DifferentPlanet - The locations are on different planets.
    pub fn direction_to(&self, o: MapLocation) -> Result<Direction, Error> {
        if self.planet != o.planet {
            Err(GameError::DifferentPlanet)?;
        }

        if o.x == self.x && o.y == self.y {
            return Ok(Center);
        }

        let dx = (o.x - self.x) as f32;
        let dy = (o.y - self.y) as f32;

        // 2.414 is an approximation of tan(67.5 degrees). It's a minor
        // optimization for an expensive trigonometric operation.
        //
        // This value is halfway between 45 and 90 degrees. So an angle x
        // such that 45 < x < 67.5 would be NE and an angle y such that
        // 67.5 < y < 90 would be N.
        if dx.abs() >= 2.414 * dy.abs() {
            if dx > 0. {
                Ok(East)
            } else {
                Ok(West)
            }
        } else if dy.abs() >= 2.414 * dx.abs() {
            if dy > 0. {
                Ok(North)
            } else {
                Ok(South)
            }
        } else {
            if dy > 0. {
                if dx > 0. {
                    Ok(Northeast)
                } else {
                    Ok(Northwest)
                }
            } else {
                if dx > 0. {
                    Ok(Southeast)
                } else {
                    Ok(Southwest)
                }
            }
        }
    }

    /// Determines whether this location is adjacent to the specified location,
    /// including diagonally. Note that squares are not adjacent to themselves,
    /// and squares on different planets are not adjacent to each other.
    pub fn is_adjacent_to(&self, o: MapLocation) -> bool {
        if self.planet != o.planet {
            return false;
        }
        let dist_squared = self.distance_squared_to(o);
        dist_squared <= 2 && dist_squared != 0
    }

    /// Whether this location is within the distance squared range of the
    /// specified location, inclusive. False for locations on different planets.
    pub fn is_within_range(&self, range: u32, o: MapLocation) -> bool {
        if self.planet != o.planet {
            return false;
        }
        range >= self.distance_squared_to(o)
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
    /// Constructs a new location on the map.
    pub fn new_on_map(map_location: MapLocation) -> Location {
        Location::OnMap(map_location)
    }

    /// Constructs a new location in a garrison.
    pub fn new_in_garrison(id: UnitID) -> Location {
        Location::InGarrison(id)
    }

    /// Constructs a new location in space.
    pub fn new_in_space() -> Location {
        Location::InSpace
    }

    /// Whether the unit is on a map.
    pub fn is_on_map(&self) -> bool {
        match *self {
            Location::OnMap(_) => true,
            _ => false,
        }
    }

    /// True if and only if the location is on the map and on this planet.
    pub fn is_on_planet(&self, planet: Planet) -> bool {
        match *self {
            Location::OnMap(map_loc) => map_loc.planet == planet,
            _ => false,
        }
    }

    /// The map location of the unit.
    ///
    /// * UnitNotOnMap - The unit is in a garrison or in space, and does not
    ///   have a map location.
    pub fn map_location(&self) -> Result<MapLocation, Error> {
        match *self {
            Location::OnMap(map_loc) => Ok(map_loc),
            _ => Err(GameError::UnitNotOnMap)?,
        }
    }

    /// Whether the unit is in a garrison.
    pub fn is_in_garrison(&self) -> bool {
        match *self {
            Location::InGarrison(_) => true,
            _ => false,
        }
    }

    /// The structure whose garrison the unit is in.
    ///
    /// * UnitNotInGarrison - the unit is not in a garrison.
    pub fn structure(&self) -> Result<UnitID, Error> {
        match *self {
            Location::InGarrison(id) => Ok(id),
            _ => Err(GameError::UnitNotInGarrison)?,
        }
    }

    /// Whether the unit is in space.
    pub fn is_in_space(&self) -> bool {
        *self == Location::InSpace
    }

    /// Determines whether this location is adjacent to the specified location,
    /// including diagonally. Note that squares are not adjacent to themselves,
    /// and squares on different planets are not adjacent to each other. Also,
    /// nothing is adjacent to something not on a map.
    pub fn is_adjacent_to(&self, o: Location) -> bool {
        if !self.is_on_map() || !o.is_on_map() {
            return false;
        }
        let this_loc = self.map_location().unwrap();
        let that_loc = o.map_location().unwrap();
        this_loc.is_adjacent_to(that_loc)
    }

    /// Whether this location is within the distance squared range of the
    /// specified location, inclusive. False for locations on different planets.
    /// Note that nothing is within the range of something not on the map.
    pub fn is_within_range(&self, range: u32, o: Location) -> bool {
        if !self.is_on_map() || !o.is_on_map() {
            return false;
        }
        let this_loc = self.map_location().unwrap();
        let that_loc = o.map_location().unwrap();
        this_loc.is_within_range(range, that_loc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Planet::*;

    #[test]
    fn test_direction() {
        assert_eq!((North.dx(), North.dy()), (0, 1));
        assert_eq!((Northeast.dx(), Northeast.dy()), (1, 1));
        assert_eq!((East.dx(), East.dy()), (1, 0));
        assert_eq!((Southeast.dx(), Southeast.dy()), (1, -1));
        assert_eq!((South.dx(), South.dy()), (0, -1));
        assert_eq!((Southwest.dx(), Southwest.dy()), (-1, -1));
        assert_eq!((West.dx(), West.dy()), (-1, 0));
        assert_eq!((Northwest.dx(), Northwest.dy()), (-1, 1));
        assert_eq!((Center.dx(), Center.dy()), (0, 0));

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
    fn test_map_location_add_subtract() {
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
    fn test_map_location_distance_squared_to() {
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
    fn test_map_location_direction_to() {
        let origin = MapLocation::new(Mars, 0, 0);
        let nn = MapLocation::new(Mars, 0, 2);
        let ne = MapLocation::new(Mars, 2, 2);
        let ee = MapLocation::new(Mars, 2, 0);
        let se = MapLocation::new(Mars, 2, -2);
        assert_eq!(origin.direction_to(nn).unwrap(), North);
        assert_eq!(nn.direction_to(origin).unwrap(), South);
        assert_eq!(origin.direction_to(ne).unwrap(), Northeast);
        assert_eq!(ne.direction_to(origin).unwrap(), Southwest);
        assert_eq!(origin.direction_to(ee).unwrap(), East);
        assert_eq!(ee.direction_to(origin).unwrap(), West);
        assert_eq!(origin.direction_to(se).unwrap(), Southeast);
        assert_eq!(se.direction_to(origin).unwrap(), Northwest);

        // Top right and bottom left quadrant
        let a = MapLocation::new(Mars, 2, 5);
        let b = MapLocation::new(Mars, 3, 4);
        let c = MapLocation::new(Mars, 4, 3);
        let d = MapLocation::new(Mars, 5, 2);
        assert_eq!(origin.direction_to(a).unwrap(), North);
        assert_eq!(a.direction_to(origin).unwrap(), South);
        assert_eq!(origin.direction_to(b).unwrap(), Northeast);
        assert_eq!(b.direction_to(origin).unwrap(), Southwest);
        assert_eq!(origin.direction_to(c).unwrap(), Northeast);
        assert_eq!(c.direction_to(origin).unwrap(), Southwest);
        assert_eq!(origin.direction_to(d).unwrap(), East);
        assert_eq!(d.direction_to(origin).unwrap(), West);

        // Top left and bottom right quadrant
        let a = MapLocation::new(Mars, 5, -2);
        let b = MapLocation::new(Mars, 4, -3);
        let c = MapLocation::new(Mars, 3, -4);
        let d = MapLocation::new(Mars, 2, -5);
        assert_eq!(origin.direction_to(a).unwrap(), East);
        assert_eq!(a.direction_to(origin).unwrap(), West);
        assert_eq!(origin.direction_to(b).unwrap(), Southeast);
        assert_eq!(b.direction_to(origin).unwrap(), Northwest);
        assert_eq!(origin.direction_to(c).unwrap(), Southeast);
        assert_eq!(c.direction_to(origin).unwrap(), Northwest);
        assert_eq!(origin.direction_to(d).unwrap(), South);
        assert_eq!(d.direction_to(origin).unwrap(), North);

        assert_err!(origin.direction_to(MapLocation::new(Earth, 0, 0)), GameError::DifferentPlanet);
    }

    #[test]
    fn test_is_adjacent_to() {
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
        assert!(!a.is_adjacent_to(a), "a square is not adjacent to itself");

        assert!(Location::OnMap(a).is_adjacent_to(Location::OnMap(c)));
        assert!(Location::OnMap(d).is_adjacent_to(Location::OnMap(c)));
        assert!(!Location::OnMap(a).is_adjacent_to(Location::OnMap(d)));
        assert!(!Location::OnMap(a).is_adjacent_to(Location::OnMap(e)));
        assert!(!Location::OnMap(a).is_adjacent_to(Location::OnMap(a)));
    }

    #[test]
    fn test_is_within_range() {
        let a = MapLocation::new(Earth, 4, 4);
        let b = MapLocation::new(Earth, 6, 5);
        let c = MapLocation::new(Mars, 4, 5);

        assert!(!a.is_within_range(10, c), "different planets");
        assert!(a.is_within_range(10, a), "location to itself");
        assert!(a.is_within_range(10, b), "location is within range");
        assert!(a.is_within_range(5, b), "location is on the border");

        assert!(!Location::OnMap(a).is_within_range(10, Location::OnMap(c)),
            "different planets");
        assert!(Location::OnMap(a).is_within_range(10, Location::OnMap(a)),
            "location to itself");
        assert!(Location::OnMap(a).is_within_range(10, Location::OnMap(b)),
            "location is within range");
        assert!(Location::OnMap(a).is_within_range(5, Location::OnMap(b)),
            "location is on the border");
    }

    #[test]
    fn test_location_is_adjacent_to() {
        let loc_a = Location::OnMap(MapLocation::new(Planet::Earth, 0, 0));
        let loc_b = Location::OnMap(MapLocation::new(Planet::Earth, 1, 1));
        let loc_c = Location::OnMap(MapLocation::new(Planet::Earth, 1, 2));

        // B is adjacent to both A and C, but A is not adjacent to C.
        assert!(loc_a.is_adjacent_to(loc_b));
        assert!(loc_b.is_adjacent_to(loc_a));
        assert!(loc_c.is_adjacent_to(loc_b));
        assert!(loc_b.is_adjacent_to(loc_c));
        assert!(!loc_a.is_adjacent_to(loc_c));
        assert!(!loc_c.is_adjacent_to(loc_a));
    }

    #[test]
    fn test_location_not_on_map() {
        let loc = Location::OnMap(MapLocation::new(Planet::Mars, 10, 10));
        let garrison = Location::InGarrison(1);
        let space = Location::InSpace;

        assert!(loc.is_on_planet(Planet::Mars));
        assert!(!loc.is_on_planet(Planet::Earth));
        assert!(!garrison.is_on_planet(Planet::Mars));
        assert!(!space.is_on_planet(Planet::Mars));

        assert!(loc.is_on_map());
        assert!(!garrison.is_on_map());
        assert!(!space.is_on_map());

        assert!(loc.map_location().is_ok());
        assert_err!(garrison.map_location(), GameError::UnitNotOnMap);
        assert_err!(space.map_location(), GameError::UnitNotOnMap);

        assert!(!loc.is_adjacent_to(garrison));
        assert!(!garrison.is_adjacent_to(space));
        assert!(!space.is_adjacent_to(loc));

        assert!(!loc.is_within_range(100, garrison));
        assert!(!garrison.is_within_range(100, space));
        assert!(!space.is_within_range(100, loc));
    }
}