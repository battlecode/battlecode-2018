//! Helper conversions between battlecode types and FFI types.
//! Should compile to no-ops.

use super::*;
use battlecode_engine::location::*;
use battlecode_engine::location::Direction::*;
use std::convert::From;

impl From<MapLocation> for bc_map_location_t {
    fn from(location: MapLocation) -> bc_map_location_t {
        bc_map_location_t {
            x: location.x,
            y: location.y
        }
    }
}
// should compile to a no-op.
impl From<bc_map_location_t> for MapLocation {
    fn from(location: bc_map_location_t) -> MapLocation {
        MapLocation {
            // TODO: make bc_map_location_t encode planet
            planet: Planet::Earth,
            x: location.x,
            y: location.y
        }
    }
}
impl From<Direction> for bc_direction {
    fn from(direction: Direction) -> bc_direction {
        match direction {
            North => bc_direction::north,
            Northeast => bc_direction::northeast,
            East => bc_direction::east,
            Southeast => bc_direction::southeast,
            South => bc_direction::south,
            Southwest => bc_direction::southwest,
            West => bc_direction::west,
            Northwest => bc_direction::northwest,
            Center => bc_direction::center,
        }
    }
}
impl From<bc_direction> for Direction {
    fn from(direction: bc_direction) -> Direction {
        match direction {
            bc_direction::north => North,
            bc_direction::northeast => Northeast,
            bc_direction::east => East,
            bc_direction::southeast => Southeast,
            bc_direction::south => South,
            bc_direction::southwest => Southwest,
            bc_direction::west => West,
            bc_direction::northwest => Northwest,
            bc_direction::center => Center
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_location_convertions() {
        let l = bc_map_location_t {
            planet: Planet::Earth,
            x: -32,
            y: 57
        };
        let l2: MapLocation = l.into();
        assert_eq!(l2.x, l.x);
        assert_eq!(l2.y, l.y);
        let l3: bc_map_location_t = l2.into();
        assert_eq!(l3.x, l.x);
        assert_eq!(l3.y, l.y);

    }

    #[test]
    fn direction_convertions() {
        let d = bc_direction::west;
        let d2: Direction = d.into();
        assert_eq!(d2, Direction::West);
        let d3: bc_direction = d2.into();
        assert_eq!(d3, d);
    }
}