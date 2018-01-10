"""Battlecode engine.

Woo."""

from ._bc import ffi as _ffi
from ._bc import lib as _lib
import threading
import enum

# might be cheaper to just allocate new strings, TODO benchmark.
def _check_errors():
    if _lib.bc_has_err():
        _lasterror = _ffi.new('char**')
        err = _lib.bc_get_last_err(_lasterror)
        errtext = _ffi.string(_lasterror[0])
        _lib.bc_free_string(_lasterror[0])
        raise Exception(errtext)

def game_turns():
    """Usage:
    for controller in game_turns():
        #controller is a GameController; do things with it
        print(controller.round)
    """
    controller = GameController()
    while True:
        yield controller
        controller.next_turn()

class Planet(enum.IntEnum):
    Earth = 0
    Mars = 1
    def other(self):
        # type: () -> Planet
        '''The other planet.
        :type self: Planet
        :rtype: Planet
        '''

        result = _lib.bc_Planet_other(self)
        _check_errors()
        result = Planet(result)
        return result

    def __repr__(self):
        # type: () -> str
        '''Create a human-readable representation of a Planet
        :type self: Planet
        :rtype: str
        '''

        result = _lib.bc_Planet_debug(self)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    def __eq__(self, other):
        # type: (Planet) -> bool
        '''Compare two Planets for deep equality.
        :type self: Planet
        :type other: Planet
        :rtype: bool
        '''
        assert type(other) is Planet, "incorrect type of arg other: should be Planet, is {}".format(type(other))

        result = _lib.bc_Planet_eq(self, other)
        _check_errors()
        result = bool(result)
        return result

    @staticmethod
    def from_json(s):
        # type: (str) -> Planet
        '''Deserialize a Planet from a JSON string
        :type s: str
        :rtype: Planet
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_Planet_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        result = Planet(result)
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a Planet to a JSON string
        :type self: Planet
        :rtype: str
        '''

        result = _lib.bc_Planet_to_json(self)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

class Direction(enum.IntEnum):
    North = 0
    Northeast = 1
    East = 2
    Southeast = 3
    South = 4
    Southwest = 5
    West = 6
    Northwest = 7
    Center = 8
    def dx(self):
        # type: () -> int
        '''Returns the x displacement of this direction.
        :type self: Direction
        :rtype: int
        '''

        result = _lib.bc_Direction_dx(self)
        _check_errors()
        return result

    def dy(self):
        # type: () -> int
        '''Returns the y displacement of this direction.
        :type self: Direction
        :rtype: int
        '''

        result = _lib.bc_Direction_dy(self)
        _check_errors()
        return result

    def is_diagonal(self):
        # type: () -> bool
        '''Whether this direction is a diagonal one.
        :type self: Direction
        :rtype: bool
        '''

        result = _lib.bc_Direction_is_diagonal(self)
        _check_errors()
        result = bool(result)
        return result

    def opposite(self):
        # type: () -> Direction
        '''Returns the direction opposite this one, or Center if it's Center.
        :type self: Direction
        :rtype: Direction
        '''

        result = _lib.bc_Direction_opposite(self)
        _check_errors()
        result = Direction(result)
        return result

    def rotate_left(self):
        # type: () -> Direction
        '''Returns the direction 45 degrees to the left (counter-clockwise) of
        this one, or Center if it's Center.
        :type self: Direction
        :rtype: Direction
        '''

        result = _lib.bc_Direction_rotate_left(self)
        _check_errors()
        result = Direction(result)
        return result

    def rotate_right(self):
        # type: () -> Direction
        '''Returns the direction 45 degrees to the right (clockwise) of this one,
        or Center if it's Center.
        :type self: Direction
        :rtype: Direction
        '''

        result = _lib.bc_Direction_rotate_right(self)
        _check_errors()
        result = Direction(result)
        return result

    @staticmethod
    def from_json(s):
        # type: (str) -> Direction
        '''Deserialize a Direction from a JSON string
        :type s: str
        :rtype: Direction
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_Direction_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        result = Direction(result)
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a Direction to a JSON string
        :type self: Direction
        :rtype: str
        '''

        result = _lib.bc_Direction_to_json(self)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

class MapLocation(object):
    __slots__ = ['_ptr']
    def __init__(self, planet, x, y):
        # type: (Planet, int, int) -> MapLocation
        '''Returns a new MapLocation representing the location with the given
        coordinates on a planet.
        :type self: MapLocation
        :type planet: Planet
        :type x: int
        :type y: int
        :rtype: MapLocation
        '''
        assert type(planet) is Planet, "incorrect type of arg planet: should be Planet, is {}".format(type(planet))
        assert type(x) is int, "incorrect type of arg x: should be int, is {}".format(type(x))
        assert type(y) is int, "incorrect type of arg y: should be int, is {}".format(type(y))

        ptr = _lib.new_bc_MapLocation(planet, x, y)
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: MapLocation
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_MapLocation(self._ptr)
            _check_errors()
    @property
    def planet(self):
        # type: () -> Planet
        '''The planet of the map location.
        :type self: MapLocation
        :rtype: Planet
        '''

        result = _lib.bc_MapLocation_planet_get(self._ptr)
        _check_errors()
        result = Planet(result)
        return result

    @property
    def x(self):
        # type: () -> int
        '''The x coordinate of the map location.
        :type self: MapLocation
        :rtype: int
        '''

        result = _lib.bc_MapLocation_x_get(self._ptr)
        _check_errors()
        return result

    @property
    def y(self):
        # type: () -> int
        '''The y coordinate of the map location.
        :type self: MapLocation
        :rtype: int
        '''

        result = _lib.bc_MapLocation_y_get(self._ptr)
        _check_errors()
        return result

    @planet.setter
    def planet(self, planet):
        # type: (Planet) -> None
        '''The planet of the map location.
        :type self: MapLocation
        :type planet: Planet
        :rtype: None
        '''
        assert type(planet) is Planet, "incorrect type of arg planet: should be Planet, is {}".format(type(planet))

        result = _lib.bc_MapLocation_planet_set(self._ptr, planet)
        _check_errors()
        return result

    @x.setter
    def x(self, x):
        # type: (int) -> None
        '''The x coordinate of the map location.
        :type self: MapLocation
        :type x: int
        :rtype: None
        '''
        assert type(x) is int, "incorrect type of arg x: should be int, is {}".format(type(x))

        result = _lib.bc_MapLocation_x_set(self._ptr, x)
        _check_errors()
        return result

    @y.setter
    def y(self, y):
        # type: (int) -> None
        '''The y coordinate of the map location.
        :type self: MapLocation
        :type y: int
        :rtype: None
        '''
        assert type(y) is int, "incorrect type of arg y: should be int, is {}".format(type(y))

        result = _lib.bc_MapLocation_y_set(self._ptr, y)
        _check_errors()
        return result

    def add(self, direction):
        # type: (Direction) -> MapLocation
        '''Returns the location one square from this one in the given direction.
        :type self: MapLocation
        :type direction: Direction
        :rtype: MapLocation
        '''
        assert type(direction) is Direction, "incorrect type of arg direction: should be Direction, is {}".format(type(direction))

        result = _lib.bc_MapLocation_add(self._ptr, direction)
        _check_errors()
        _result = MapLocation.__new__(MapLocation)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def subtract(self, direction):
        # type: (Direction) -> MapLocation
        '''Returns the location one square from this one in the opposite direction.
        :type self: MapLocation
        :type direction: Direction
        :rtype: MapLocation
        '''
        assert type(direction) is Direction, "incorrect type of arg direction: should be Direction, is {}".format(type(direction))

        result = _lib.bc_MapLocation_subtract(self._ptr, direction)
        _check_errors()
        _result = MapLocation.__new__(MapLocation)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def add_multiple(self, direction, multiple):
        # type: (Direction, int) -> MapLocation
        '''Returns the location `multiple` squares from this one in the given
        direction.
        :type self: MapLocation
        :type direction: Direction
        :type multiple: int
        :rtype: MapLocation
        '''
        assert type(direction) is Direction, "incorrect type of arg direction: should be Direction, is {}".format(type(direction))
        assert type(multiple) is int, "incorrect type of arg multiple: should be int, is {}".format(type(multiple))

        result = _lib.bc_MapLocation_add_multiple(self._ptr, direction, multiple)
        _check_errors()
        _result = MapLocation.__new__(MapLocation)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def translate(self, dx, dy):
        # type: (int, int) -> MapLocation
        '''Returns the location translated from this location by `dx` in the x
        direction and `dy` in the y direction.
        :type self: MapLocation
        :type dx: int
        :type dy: int
        :rtype: MapLocation
        '''
        assert type(dx) is int, "incorrect type of arg dx: should be int, is {}".format(type(dx))
        assert type(dy) is int, "incorrect type of arg dy: should be int, is {}".format(type(dy))

        result = _lib.bc_MapLocation_translate(self._ptr, dx, dy)
        _check_errors()
        _result = MapLocation.__new__(MapLocation)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def distance_squared_to(self, o):
        # type: (MapLocation) -> int
        '''Computes the square of the distance from this location to the specified
        location. If on different planets, returns the maximum integer.
        :type self: MapLocation
        :type o: MapLocation
        :rtype: int
        '''
        assert type(o) is MapLocation, "incorrect type of arg o: should be MapLocation, is {}".format(type(o))

        result = _lib.bc_MapLocation_distance_squared_to(self._ptr, o._ptr)
        _check_errors()
        return result

    def direction_to(self, o):
        # type: (MapLocation) -> Direction
        '''Returns the Direction from this location to the specified location.
        If the locations are equal this method returns Center.

         * DifferentPlanet - The locations are on different planets.
        :type self: MapLocation
        :type o: MapLocation
        :rtype: Direction
        '''
        assert type(o) is MapLocation, "incorrect type of arg o: should be MapLocation, is {}".format(type(o))

        result = _lib.bc_MapLocation_direction_to(self._ptr, o._ptr)
        _check_errors()
        result = Direction(result)
        return result

    def is_adjacent_to(self, o):
        # type: (MapLocation) -> bool
        '''
        Determines whether this location is adjacent to the specified location,
        including diagonally. Note that squares are not adjacent to themselves,
        and squares on different planets are not adjacent to each other.
        :type self: MapLocation
        :type o: MapLocation
        :rtype: bool
        '''
        assert type(o) is MapLocation, "incorrect type of arg o: should be MapLocation, is {}".format(type(o))

        result = _lib.bc_MapLocation_is_adjacent_to(self._ptr, o._ptr)
        _check_errors()
        result = bool(result)
        return result

    def is_within_range(self, range, o):
        # type: (int, MapLocation) -> bool
        '''
        Whether this location is within the distance squared range of the
        specified location, inclusive. False for locations on different planets.
        :type self: MapLocation
        :type range: int
        :type o: MapLocation
        :rtype: bool
        '''
        assert type(range) is int, "incorrect type of arg range: should be int, is {}".format(type(range))
        assert type(o) is MapLocation, "incorrect type of arg o: should be MapLocation, is {}".format(type(o))

        result = _lib.bc_MapLocation_is_within_range(self._ptr, range, o._ptr)
        _check_errors()
        result = bool(result)
        return result

    def __repr__(self):
        # type: () -> str
        '''Create a human-readable representation of a MapLocation
        :type self: MapLocation
        :rtype: str
        '''

        result = _lib.bc_MapLocation_debug(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    def clone(self):
        # type: () -> MapLocation
        '''Deep-copy a MapLocation
        :type self: MapLocation
        :rtype: MapLocation
        '''

        result = _lib.bc_MapLocation_clone(self._ptr)
        _check_errors()
        _result = MapLocation.__new__(MapLocation)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def __eq__(self, other):
        # type: (MapLocation) -> bool
        '''Compare two MapLocations for deep equality.
        :type self: MapLocation
        :type other: MapLocation
        :rtype: bool
        '''
        assert type(other) is MapLocation, "incorrect type of arg other: should be MapLocation, is {}".format(type(other))

        result = _lib.bc_MapLocation_eq(self._ptr, other._ptr)
        _check_errors()
        result = bool(result)
        return result

    @staticmethod
    def from_json(s):
        # type: (str) -> MapLocation
        '''Deserialize a MapLocation from a JSON string
        :type s: str
        :rtype: MapLocation
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_MapLocation_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = MapLocation.__new__(MapLocation)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a MapLocation to a JSON string
        :type self: MapLocation
        :rtype: str
        '''

        result = _lib.bc_MapLocation_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result



class VecMapLocation(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> VecMapLocation
        '''An immutable list of bc::location::MapLocation objects
        :type self: VecMapLocation
        :rtype: VecMapLocation
        '''

        ptr = _lib.new_bc_VecMapLocation()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: VecMapLocation
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_VecMapLocation(self._ptr)
            _check_errors()


    def __repr__(self):
        # type: () -> str
        '''Create a human-readable representation of a VecMapLocation
        :type self: VecMapLocation
        :rtype: str
        '''

        result = _lib.bc_VecMapLocation_debug(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    def clone(self):
        # type: () -> VecMapLocation
        '''Deep-copy a VecMapLocation
        :type self: VecMapLocation
        :rtype: VecMapLocation
        '''

        result = _lib.bc_VecMapLocation_clone(self._ptr)
        _check_errors()
        _result = VecMapLocation.__new__(VecMapLocation)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def __len__(self):
        # type: () -> int
        '''The length of the vector.
        :type self: VecMapLocation
        :rtype: int
        '''

        result = _lib.bc_VecMapLocation_len(self._ptr)
        _check_errors()
        return result

    def __getitem__(self, index):
        # type: (int) -> MapLocation
        '''Copy an element out of the vector.
        :type self: VecMapLocation
        :type index: int
        :rtype: MapLocation
        '''
        assert type(index) is int, "incorrect type of arg index: should be int, is {}".format(type(index))

        result = _lib.bc_VecMapLocation_index(self._ptr, index)
        _check_errors()
        _result = MapLocation.__new__(MapLocation)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result


    def __iter__(self):
        l = len(self)
        for i in range(l):
            yield self[i]



class Veci32(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> Veci32
        '''An immutable list of i32 objects
        :type self: Veci32
        :rtype: Veci32
        '''

        ptr = _lib.new_bc_Veci32()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: Veci32
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_Veci32(self._ptr)
            _check_errors()


    def __repr__(self):
        # type: () -> str
        '''Create a human-readable representation of a Veci32
        :type self: Veci32
        :rtype: str
        '''

        result = _lib.bc_Veci32_debug(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    def clone(self):
        # type: () -> Veci32
        '''Deep-copy a Veci32
        :type self: Veci32
        :rtype: Veci32
        '''

        result = _lib.bc_Veci32_clone(self._ptr)
        _check_errors()
        _result = Veci32.__new__(Veci32)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def __len__(self):
        # type: () -> int
        '''The length of the vector.
        :type self: Veci32
        :rtype: int
        '''

        result = _lib.bc_Veci32_len(self._ptr)
        _check_errors()
        return result

    def __getitem__(self, index):
        # type: (int) -> int
        '''Copy an element out of the vector.
        :type self: Veci32
        :type index: int
        :rtype: int
        '''
        assert type(index) is int, "incorrect type of arg index: should be int, is {}".format(type(index))

        result = _lib.bc_Veci32_index(self._ptr, index)
        _check_errors()
        return result


    def __iter__(self):
        l = len(self)
        for i in range(l):
            yield self[i]

class Location(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> Location
        '''
        :type self: Location
        :rtype: Location
        '''

        ptr = _lib.new_bc_Location()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: Location
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_Location(self._ptr)
            _check_errors()


    @staticmethod
    def new_on_map(map_location):
        # type: (MapLocation) -> Location
        '''Constructs a new location on the map.
        :type map_location: MapLocation
        :rtype: Location
        '''
        assert type(map_location) is MapLocation, "incorrect type of arg map_location: should be MapLocation, is {}".format(type(map_location))

        result = _lib.bc_Location_new_on_map(map_location._ptr)
        _check_errors()
        _result = Location.__new__(Location)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    @staticmethod
    def new_in_garrison(id):
        # type: (int) -> Location
        '''Constructs a new location in a garrison.
        :type id: int
        :rtype: Location
        '''
        assert type(id) is int, "incorrect type of arg id: should be int, is {}".format(type(id))

        result = _lib.bc_Location_new_in_garrison(id)
        _check_errors()
        _result = Location.__new__(Location)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    @staticmethod
    def new_in_space():
        # type: () -> Location
        '''Constructs a new location in space.
        :rtype: Location
        '''

        result = _lib.bc_Location_new_in_space()
        _check_errors()
        _result = Location.__new__(Location)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def is_on_map(self):
        # type: () -> bool
        '''Whether the unit is on a map.
        :type self: Location
        :rtype: bool
        '''

        result = _lib.bc_Location_is_on_map(self._ptr)
        _check_errors()
        result = bool(result)
        return result

    def is_on_planet(self, planet):
        # type: (Planet) -> bool
        '''True if and only if the location is on the map and on this planet.
        :type self: Location
        :type planet: Planet
        :rtype: bool
        '''
        assert type(planet) is Planet, "incorrect type of arg planet: should be Planet, is {}".format(type(planet))

        result = _lib.bc_Location_is_on_planet(self._ptr, planet)
        _check_errors()
        result = bool(result)
        return result

    def map_location(self):
        # type: () -> MapLocation
        '''The map location of the unit.

         * UnitNotOnMap - The unit is in a garrison or in space, and does not
           have a map location.
        :type self: Location
        :rtype: MapLocation
        '''

        result = _lib.bc_Location_map_location(self._ptr)
        _check_errors()
        _result = MapLocation.__new__(MapLocation)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def is_in_garrison(self):
        # type: () -> bool
        '''Whether the unit is in a garrison.
        :type self: Location
        :rtype: bool
        '''

        result = _lib.bc_Location_is_in_garrison(self._ptr)
        _check_errors()
        result = bool(result)
        return result

    def structure(self):
        # type: () -> int
        '''The structure whose garrison the unit is in.

         * UnitNotInGarrison - the unit is not in a garrison.
        :type self: Location
        :rtype: int
        '''

        result = _lib.bc_Location_structure(self._ptr)
        _check_errors()
        return result

    def is_in_space(self):
        # type: () -> bool
        '''Whether the unit is in space.
        :type self: Location
        :rtype: bool
        '''

        result = _lib.bc_Location_is_in_space(self._ptr)
        _check_errors()
        result = bool(result)
        return result

    def is_adjacent_to(self, o):
        # type: (Location) -> bool
        '''Determines whether this location is adjacent to the specified location,
        including diagonally. Note that squares are not adjacent to themselves,
        and squares on different planets are not adjacent to each other. Also,
        nothing is adjacent to something not on a map.
        :type self: Location
        :type o: Location
        :rtype: bool
        '''
        assert type(o) is Location, "incorrect type of arg o: should be Location, is {}".format(type(o))

        result = _lib.bc_Location_is_adjacent_to(self._ptr, o._ptr)
        _check_errors()
        result = bool(result)
        return result

    def is_within_range(self, range, o):
        # type: (int, Location) -> bool
        '''Whether this location is within the distance squared range of the
        specified location, inclusive. False for locations on different planets.
        Note that nothing is within the range of something not on the map.
        :type self: Location
        :type range: int
        :type o: Location
        :rtype: bool
        '''
        assert type(range) is int, "incorrect type of arg range: should be int, is {}".format(type(range))
        assert type(o) is Location, "incorrect type of arg o: should be Location, is {}".format(type(o))

        result = _lib.bc_Location_is_within_range(self._ptr, range, o._ptr)
        _check_errors()
        result = bool(result)
        return result

    def __repr__(self):
        # type: () -> str
        '''Create a human-readable representation of a Location
        :type self: Location
        :rtype: str
        '''

        result = _lib.bc_Location_debug(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    def clone(self):
        # type: () -> Location
        '''Deep-copy a Location
        :type self: Location
        :rtype: Location
        '''

        result = _lib.bc_Location_clone(self._ptr)
        _check_errors()
        _result = Location.__new__(Location)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def __eq__(self, other):
        # type: (Location) -> bool
        '''Compare two Locations for deep equality.
        :type self: Location
        :type other: Location
        :rtype: bool
        '''
        assert type(other) is Location, "incorrect type of arg other: should be Location, is {}".format(type(other))

        result = _lib.bc_Location_eq(self._ptr, other._ptr)
        _check_errors()
        result = bool(result)
        return result

    @staticmethod
    def from_json(s):
        # type: (str) -> Location
        '''Deserialize a Location from a JSON string
        :type s: str
        :rtype: Location
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_Location_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = Location.__new__(Location)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a Location to a JSON string
        :type self: Location
        :rtype: str
        '''

        result = _lib.bc_Location_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result



class Team(enum.IntEnum):
    Red = 0
    Blue = 1
    @staticmethod
    def from_json(s):
        # type: (str) -> Team
        '''Deserialize a Team from a JSON string
        :type s: str
        :rtype: Team
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_Team_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        result = Team(result)
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a Team to a JSON string
        :type self: Team
        :rtype: str
        '''

        result = _lib.bc_Team_to_json(self)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

class Player(object):
    __slots__ = ['_ptr']
    def __init__(self, team, planet):
        # type: (Team, Planet) -> Player
        '''
        :type self: Player
        :type team: Team
        :type planet: Planet
        :rtype: Player
        '''
        assert type(team) is Team, "incorrect type of arg team: should be Team, is {}".format(type(team))
        assert type(planet) is Planet, "incorrect type of arg planet: should be Planet, is {}".format(type(planet))

        ptr = _lib.new_bc_Player(team, planet)
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: Player
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_Player(self._ptr)
            _check_errors()
    @property
    def team(self):
        # type: () -> Team
        '''
        :type self: Player
        :rtype: Team
        '''

        result = _lib.bc_Player_team_get(self._ptr)
        _check_errors()
        result = Team(result)
        return result

    @property
    def planet(self):
        # type: () -> Planet
        '''
        :type self: Player
        :rtype: Planet
        '''

        result = _lib.bc_Player_planet_get(self._ptr)
        _check_errors()
        result = Planet(result)
        return result

    @team.setter
    def team(self, team):
        # type: (Team) -> None
        '''
        :type self: Player
        :type team: Team
        :rtype: None
        '''
        assert type(team) is Team, "incorrect type of arg team: should be Team, is {}".format(type(team))

        result = _lib.bc_Player_team_set(self._ptr, team)
        _check_errors()
        return result

    @planet.setter
    def planet(self, planet):
        # type: (Planet) -> None
        '''
        :type self: Player
        :type planet: Planet
        :rtype: None
        '''
        assert type(planet) is Planet, "incorrect type of arg planet: should be Planet, is {}".format(type(planet))

        result = _lib.bc_Player_planet_set(self._ptr, planet)
        _check_errors()
        return result

    def __repr__(self):
        # type: () -> str
        '''Create a human-readable representation of a Player
        :type self: Player
        :rtype: str
        '''

        result = _lib.bc_Player_debug(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    def clone(self):
        # type: () -> Player
        '''Deep-copy a Player
        :type self: Player
        :rtype: Player
        '''

        result = _lib.bc_Player_clone(self._ptr)
        _check_errors()
        _result = Player.__new__(Player)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def __eq__(self, other):
        # type: (Player) -> bool
        '''Compare two Players for deep equality.
        :type self: Player
        :type other: Player
        :rtype: bool
        '''
        assert type(other) is Player, "incorrect type of arg other: should be Player, is {}".format(type(other))

        result = _lib.bc_Player_eq(self._ptr, other._ptr)
        _check_errors()
        result = bool(result)
        return result

    @staticmethod
    def from_json(s):
        # type: (str) -> Player
        '''Deserialize a Player from a JSON string
        :type s: str
        :rtype: Player
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_Player_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = Player.__new__(Player)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a Player to a JSON string
        :type self: Player
        :rtype: str
        '''

        result = _lib.bc_Player_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result






class VecUnitID(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> VecUnitID
        '''An immutable list of bc::unit::UnitID objects
        :type self: VecUnitID
        :rtype: VecUnitID
        '''

        ptr = _lib.new_bc_VecUnitID()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: VecUnitID
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_VecUnitID(self._ptr)
            _check_errors()


    def __repr__(self):
        # type: () -> str
        '''Create a human-readable representation of a VecUnitID
        :type self: VecUnitID
        :rtype: str
        '''

        result = _lib.bc_VecUnitID_debug(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    def clone(self):
        # type: () -> VecUnitID
        '''Deep-copy a VecUnitID
        :type self: VecUnitID
        :rtype: VecUnitID
        '''

        result = _lib.bc_VecUnitID_clone(self._ptr)
        _check_errors()
        _result = VecUnitID.__new__(VecUnitID)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def __len__(self):
        # type: () -> int
        '''The length of the vector.
        :type self: VecUnitID
        :rtype: int
        '''

        result = _lib.bc_VecUnitID_len(self._ptr)
        _check_errors()
        return result

    def __getitem__(self, index):
        # type: (int) -> int
        '''Copy an element out of the vector.
        :type self: VecUnitID
        :type index: int
        :rtype: int
        '''
        assert type(index) is int, "incorrect type of arg index: should be int, is {}".format(type(index))

        result = _lib.bc_VecUnitID_index(self._ptr, index)
        _check_errors()
        return result


    def __iter__(self):
        l = len(self)
        for i in range(l):
            yield self[i]

class UnitType(enum.IntEnum):
    Worker = 0
    Knight = 1
    Ranger = 2
    Mage = 3
    Healer = 4
    Factory = 5
    Rocket = 6
    @staticmethod
    def from_json(s):
        # type: (str) -> UnitType
        '''Deserialize a UnitType from a JSON string
        :type s: str
        :rtype: UnitType
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_UnitType_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        result = UnitType(result)
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a UnitType to a JSON string
        :type self: UnitType
        :rtype: str
        '''

        result = _lib.bc_UnitType_to_json(self)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    def factory_cost(self):
        # type: () -> int
        '''The cost of the unit in a factory.

         * InappropriateUnitType - the unit type cannot be produced in a factory.
        :type self: UnitType
        :rtype: int
        '''

        result = _lib.bc_UnitType_factory_cost(self)
        _check_errors()
        return result

    def blueprint_cost(self):
        # type: () -> int
        '''The cost to blueprint the unit.

         * InappropriateUnitType - the unit type cannot be blueprinted.
        :type self: UnitType
        :rtype: int
        '''

        result = _lib.bc_UnitType_blueprint_cost(self)
        _check_errors()
        return result

    def replicate_cost(self):
        # type: () -> int
        '''The cost to replicate the unit.

         * InappropriateUnitType - the unit type is not a worker.
        :type self: UnitType
        :rtype: int
        '''

        result = _lib.bc_UnitType_replicate_cost(self)
        _check_errors()
        return result

    def value(self):
        # type: () -> int
        '''The value of a unit, as relevant to tiebreakers.
        :type self: UnitType
        :rtype: int
        '''

        result = _lib.bc_UnitType_value(self)
        _check_errors()
        return result

class VecUnitType(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> VecUnitType
        '''An immutable list of bc::unit::UnitType objects
        :type self: VecUnitType
        :rtype: VecUnitType
        '''

        ptr = _lib.new_bc_VecUnitType()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: VecUnitType
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_VecUnitType(self._ptr)
            _check_errors()


    def __repr__(self):
        # type: () -> str
        '''Create a human-readable representation of a VecUnitType
        :type self: VecUnitType
        :rtype: str
        '''

        result = _lib.bc_VecUnitType_debug(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    def clone(self):
        # type: () -> VecUnitType
        '''Deep-copy a VecUnitType
        :type self: VecUnitType
        :rtype: VecUnitType
        '''

        result = _lib.bc_VecUnitType_clone(self._ptr)
        _check_errors()
        _result = VecUnitType.__new__(VecUnitType)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def __len__(self):
        # type: () -> int
        '''The length of the vector.
        :type self: VecUnitType
        :rtype: int
        '''

        result = _lib.bc_VecUnitType_len(self._ptr)
        _check_errors()
        return result

    def __getitem__(self, index):
        # type: (int) -> UnitType
        '''Copy an element out of the vector.
        :type self: VecUnitType
        :type index: int
        :rtype: UnitType
        '''
        assert type(index) is int, "incorrect type of arg index: should be int, is {}".format(type(index))

        result = _lib.bc_VecUnitType_index(self._ptr, index)
        _check_errors()
        result = UnitType(result)
        return result


    def __iter__(self):
        l = len(self)
        for i in range(l):
            yield self[i]

class Unit(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> Unit
        '''A single unit in the game and all its associated properties.
        :type self: Unit
        :rtype: Unit
        '''

        ptr = _lib.new_bc_Unit()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: Unit
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_Unit(self._ptr)
            _check_errors()


    def __repr__(self):
        # type: () -> str
        '''Create a human-readable representation of a Unit
        :type self: Unit
        :rtype: str
        '''

        result = _lib.bc_Unit_debug(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    def clone(self):
        # type: () -> Unit
        '''Deep-copy a Unit
        :type self: Unit
        :rtype: Unit
        '''

        result = _lib.bc_Unit_clone(self._ptr)
        _check_errors()
        _result = Unit.__new__(Unit)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    @staticmethod
    def from_json(s):
        # type: (str) -> Unit
        '''Deserialize a Unit from a JSON string
        :type s: str
        :rtype: Unit
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_Unit_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = Unit.__new__(Unit)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a Unit to a JSON string
        :type self: Unit
        :rtype: str
        '''

        result = _lib.bc_Unit_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    def __eq__(self, other):
        # type: (Unit) -> bool
        '''Compare two Units for deep equality.
        :type self: Unit
        :type other: Unit
        :rtype: bool
        '''
        assert type(other) is Unit, "incorrect type of arg other: should be Unit, is {}".format(type(other))

        result = _lib.bc_Unit_eq(self._ptr, other._ptr)
        _check_errors()
        result = bool(result)
        return result

    @property
    def id(self):
        # type: () -> int
        '''The unique ID of a unit.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_id(self._ptr)
        _check_errors()
        return result

    @property
    def team(self):
        # type: () -> Team
        '''The team the unit belongs to.
        :type self: Unit
        :rtype: Team
        '''

        result = _lib.bc_Unit_team(self._ptr)
        _check_errors()
        result = Team(result)
        return result

    @property
    def research_level(self):
        # type: () -> int
        '''The current research level.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_research_level(self._ptr)
        _check_errors()
        return result

    @property
    def unit_type(self):
        # type: () -> UnitType
        '''The unit type.
        :type self: Unit
        :rtype: UnitType
        '''

        result = _lib.bc_Unit_unit_type(self._ptr)
        _check_errors()
        result = UnitType(result)
        return result

    @property
    def location(self):
        # type: () -> Location
        '''The location of the unit.
        :type self: Unit
        :rtype: Location
        '''

        result = _lib.bc_Unit_location(self._ptr)
        _check_errors()
        _result = Location.__new__(Location)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    @property
    def health(self):
        # type: () -> int
        '''The current health.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_health(self._ptr)
        _check_errors()
        return result

    @property
    def max_health(self):
        # type: () -> int
        '''The maximum health.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_max_health(self._ptr)
        _check_errors()
        return result

    @property
    def vision_range(self):
        # type: () -> int
        '''The unit vision range.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_vision_range(self._ptr)
        _check_errors()
        return result

    def damage(self):
        # type: () -> int
        '''The damage inflicted by the robot during a normal attack.

         * InappropriateUnitType - the unit is not a robot.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_damage(self._ptr)
        _check_errors()
        return result

    def attack_range(self):
        # type: () -> int
        '''The attack range.

         * InappropriateUnitType - the unit is not a robot.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_attack_range(self._ptr)
        _check_errors()
        return result

    def movement_heat(self):
        # type: () -> int
        '''The movement heat.

         * InappropriateUnitType - the unit is not a robot.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_movement_heat(self._ptr)
        _check_errors()
        return result

    def attack_heat(self):
        # type: () -> int
        '''The attack heat.

         * InappropriateUnitType - the unit is not a robot.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_attack_heat(self._ptr)
        _check_errors()
        return result

    def movement_cooldown(self):
        # type: () -> int
        '''The movement cooldown.

         * InappropriateUnitType - the unit is not a robot.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_movement_cooldown(self._ptr)
        _check_errors()
        return result

    def attack_cooldown(self):
        # type: () -> int
        '''The attack cooldown.

         * InappropriateUnitType - the unit is not a robot.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_attack_cooldown(self._ptr)
        _check_errors()
        return result

    def is_ability_unlocked(self):
        # type: () -> bool
        '''Whether the active ability is unlocked.

         * InappropriateUnitType - the unit is not a robot.
        :type self: Unit
        :rtype: bool
        '''

        result = _lib.bc_Unit_is_ability_unlocked(self._ptr)
        _check_errors()
        result = bool(result)
        return result

    def ability_heat(self):
        # type: () -> int
        '''The active ability heat.

         * InappropriateUnitType - the unit is not a robot.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_ability_heat(self._ptr)
        _check_errors()
        return result

    def ability_cooldown(self):
        # type: () -> int
        '''The active ability cooldown.

         * InappropriateUnitType - the unit is not a robot.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_ability_cooldown(self._ptr)
        _check_errors()
        return result

    def ability_range(self):
        # type: () -> int
        '''The active ability range. This is the range in which: workers can replicate, knights can javelin, rangers can snipe, mages can blink, and healers can overcharge.

         * InappropriateUnitType - the unit is not a robot.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_ability_range(self._ptr)
        _check_errors()
        return result

    def worker_has_acted(self):
        # type: () -> bool
        '''Whether the worker has already acted (harveted, blueprinted, built, or repaired) this round.

         * InappropriateUnitType - the unit is not a worker.
        :type self: Unit
        :rtype: bool
        '''

        result = _lib.bc_Unit_worker_has_acted(self._ptr)
        _check_errors()
        result = bool(result)
        return result

    def worker_build_health(self):
        # type: () -> int
        '''The health restored when building a structure.

         * InappropriateUnitType - the unit is not a worker.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_worker_build_health(self._ptr)
        _check_errors()
        return result

    def worker_repair_health(self):
        # type: () -> int
        '''The health restored when repairing a structure.

         * InappropriateUnitType - the unit is not a worker.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_worker_repair_health(self._ptr)
        _check_errors()
        return result

    def worker_harvest_amount(self):
        # type: () -> int
        '''The maximum amount of karbonite harvested from a deposit in one turn.

         * InappropriateUnitType - the unit is not a worker.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_worker_harvest_amount(self._ptr)
        _check_errors()
        return result

    def knight_defense(self):
        # type: () -> int
        '''The amount of damage resisted by a knight when attacked.

         * InappropriateUnitType - the unit is not a knight.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_knight_defense(self._ptr)
        _check_errors()
        return result

    def ranger_cannot_attack_range(self):
        # type: () -> int
        '''The range within a ranger cannot attack.

         * InappropriateUnitType - the unit is not a ranger.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_ranger_cannot_attack_range(self._ptr)
        _check_errors()
        return result

    def ranger_max_countdown(self):
        # type: () -> int
        '''The maximum countdown for ranger's snipe, which is the number of turns that must pass before the snipe is executed.

         * InappropriateUnitType - the unit is not a ranger.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_ranger_max_countdown(self._ptr)
        _check_errors()
        return result

    def ranger_is_sniping(self):
        # type: () -> bool
        '''Whether the ranger is sniping.

         * InappropriateUnitType - the unit is not a ranger.
        :type self: Unit
        :rtype: bool
        '''

        result = _lib.bc_Unit_ranger_is_sniping(self._ptr)
        _check_errors()
        result = bool(result)
        return result

    def ranger_target_location(self):
        # type: () -> MapLocation
        '''The target location for ranger's snipe.

         * InappropriateUnitType - the unit is not a ranger.
         * NullValue - the ranger is not sniping.
        :type self: Unit
        :rtype: MapLocation
        '''

        result = _lib.bc_Unit_ranger_target_location(self._ptr)
        _check_errors()
        _result = MapLocation.__new__(MapLocation)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def ranger_countdown(self):
        # type: () -> int
        '''The countdown for ranger's snipe. Errors if the ranger is not sniping.

         * InappropriateUnitType - the unit is not a ranger.
         * NullValue - the ranger is not sniping.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_ranger_countdown(self._ptr)
        _check_errors()
        return result

    def healer_self_heal_amount(self):
        # type: () -> int
        '''The amount of health passively restored to itself each round.

         * InappropriateUnitType - the unit is not a healer.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_healer_self_heal_amount(self._ptr)
        _check_errors()
        return result

    def structure_is_built(self):
        # type: () -> bool
        '''Whether this structure has been built.

         * InappropriateUnitType - the unit is not a structure.
        :type self: Unit
        :rtype: bool
        '''

        result = _lib.bc_Unit_structure_is_built(self._ptr)
        _check_errors()
        result = bool(result)
        return result

    def structure_max_capacity(self):
        # type: () -> int
        '''The max capacity of a structure.

         * InappropriateUnitType - the unit is not a structure.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_structure_max_capacity(self._ptr)
        _check_errors()
        return result

    def structure_garrison(self):
        # type: () -> VecUnitID
        '''Returns the units in the structure's garrison.

         * InappropriateUnitType - the unit is not a structure.
        :type self: Unit
        :rtype: VecUnitID
        '''

        result = _lib.bc_Unit_structure_garrison(self._ptr)
        _check_errors()
        _result = VecUnitID.__new__(VecUnitID)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def is_factory_producing(self):
        # type: () -> bool
        '''Whether the factory is currently producing a unit.

        * InappropriateUnitType - the unit is not a factory.
        :type self: Unit
        :rtype: bool
        '''

        result = _lib.bc_Unit_is_factory_producing(self._ptr)
        _check_errors()
        result = bool(result)
        return result

    def factory_unit_type(self):
        # type: () -> UnitType
        '''The unit type currently being produced by the factory.

         * InappropriateUnitType - the unit is not a factory.
        * NullValue - the factory is not producing.
        :type self: Unit
        :rtype: UnitType
        '''

        result = _lib.bc_Unit_factory_unit_type(self._ptr)
        _check_errors()
        result = UnitType(result)
        return result

    def factory_rounds_left(self):
        # type: () -> int
        '''The number of rounds left to produce a robot in this factory.

         * InappropriateUnitType - the unit is not a factory.
         * NullValue - the factory is not producing.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_factory_rounds_left(self._ptr)
        _check_errors()
        return result

    def factory_max_rounds_left(self):
        # type: () -> int
        '''The maximum number of rounds left to produce a robot in this factory.

         * InappropriateUnitType - the unit is not a factory.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_factory_max_rounds_left(self._ptr)
        _check_errors()
        return result

    def rocket_is_used(self):
        # type: () -> bool
        '''Whether the rocket has already been used.

         * InappropriateUnitType - the unit is not a rocket.
        :type self: Unit
        :rtype: bool
        '''

        result = _lib.bc_Unit_rocket_is_used(self._ptr)
        _check_errors()
        result = bool(result)
        return result

    def rocket_blast_damage(self):
        # type: () -> int
        '''The damage a rocket deals to adjacent units upon landing.

         * InappropriateUnitType - the unit is not a rocket.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_rocket_blast_damage(self._ptr)
        _check_errors()
        return result

    def rocket_travel_time_decrease(self):
        # type: () -> int
        '''The number of rounds the rocket travel time is reduced by compared to the travel time determined by the orbit of the planets.

         * InappropriateUnitType - the unit is not a rocket.
        :type self: Unit
        :rtype: int
        '''

        result = _lib.bc_Unit_rocket_travel_time_decrease(self._ptr)
        _check_errors()
        return result



class VecUnit(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> VecUnit
        '''An immutable list of bc::unit::Unit objects
        :type self: VecUnit
        :rtype: VecUnit
        '''

        ptr = _lib.new_bc_VecUnit()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: VecUnit
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_VecUnit(self._ptr)
            _check_errors()


    def __repr__(self):
        # type: () -> str
        '''Create a human-readable representation of a VecUnit
        :type self: VecUnit
        :rtype: str
        '''

        result = _lib.bc_VecUnit_debug(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    def clone(self):
        # type: () -> VecUnit
        '''Deep-copy a VecUnit
        :type self: VecUnit
        :rtype: VecUnit
        '''

        result = _lib.bc_VecUnit_clone(self._ptr)
        _check_errors()
        _result = VecUnit.__new__(VecUnit)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def __len__(self):
        # type: () -> int
        '''The length of the vector.
        :type self: VecUnit
        :rtype: int
        '''

        result = _lib.bc_VecUnit_len(self._ptr)
        _check_errors()
        return result

    def __getitem__(self, index):
        # type: (int) -> Unit
        '''Copy an element out of the vector.
        :type self: VecUnit
        :type index: int
        :rtype: Unit
        '''
        assert type(index) is int, "incorrect type of arg index: should be int, is {}".format(type(index))

        result = _lib.bc_VecUnit_index(self._ptr, index)
        _check_errors()
        _result = Unit.__new__(Unit)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result


    def __iter__(self):
        l = len(self)
        for i in range(l):
            yield self[i]

class PlanetMap(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> PlanetMap
        '''The map for one of the planets in the Battlecode world. This information defines the terrain, dimensions, and initial units of the planet.
        :type self: PlanetMap
        :rtype: PlanetMap
        '''

        ptr = _lib.new_bc_PlanetMap()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: PlanetMap
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_PlanetMap(self._ptr)
            _check_errors()
    @property
    def planet(self):
        # type: () -> Planet
        '''The planet of the map.
        :type self: PlanetMap
        :rtype: Planet
        '''

        result = _lib.bc_PlanetMap_planet_get(self._ptr)
        _check_errors()
        result = Planet(result)
        return result

    @property
    def height(self):
        # type: () -> int
        '''The height of this map, in squares. Must be in the range [MAP_HEIGHT_MIN, MAP_HEIGHT_MAX], inclusive.
        :type self: PlanetMap
        :rtype: int
        '''

        result = _lib.bc_PlanetMap_height_get(self._ptr)
        _check_errors()
        return result

    @property
    def width(self):
        # type: () -> int
        '''The height of this map, in squares. Must be in the range [MAP_WIDTH_MIN, MAP_WIDTH_MAX], inclusive.
        :type self: PlanetMap
        :rtype: int
        '''

        result = _lib.bc_PlanetMap_width_get(self._ptr)
        _check_errors()
        return result

    @property
    def initial_units(self):
        # type: () -> VecUnit
        '''The initial units on the map. Each team starts with 1 to 3 Workers on Earth.
        :type self: PlanetMap
        :rtype: VecUnit
        '''

        result = _lib.bc_PlanetMap_initial_units_get(self._ptr)
        _check_errors()
        _result = VecUnit.__new__(VecUnit)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    @planet.setter
    def planet(self, planet):
        # type: (Planet) -> None
        '''The planet of the map.
        :type self: PlanetMap
        :type planet: Planet
        :rtype: None
        '''
        assert type(planet) is Planet, "incorrect type of arg planet: should be Planet, is {}".format(type(planet))

        result = _lib.bc_PlanetMap_planet_set(self._ptr, planet)
        _check_errors()
        return result

    @height.setter
    def height(self, height):
        # type: (int) -> None
        '''The height of this map, in squares. Must be in the range [MAP_HEIGHT_MIN, MAP_HEIGHT_MAX], inclusive.
        :type self: PlanetMap
        :type height: int
        :rtype: None
        '''
        assert type(height) is int, "incorrect type of arg height: should be int, is {}".format(type(height))

        result = _lib.bc_PlanetMap_height_set(self._ptr, height)
        _check_errors()
        return result

    @width.setter
    def width(self, width):
        # type: (int) -> None
        '''The height of this map, in squares. Must be in the range [MAP_WIDTH_MIN, MAP_WIDTH_MAX], inclusive.
        :type self: PlanetMap
        :type width: int
        :rtype: None
        '''
        assert type(width) is int, "incorrect type of arg width: should be int, is {}".format(type(width))

        result = _lib.bc_PlanetMap_width_set(self._ptr, width)
        _check_errors()
        return result

    @initial_units.setter
    def initial_units(self, initial_units):
        # type: (VecUnit) -> None
        '''The initial units on the map. Each team starts with 1 to 3 Workers on Earth.
        :type self: PlanetMap
        :type initial_units: VecUnit
        :rtype: None
        '''
        assert type(initial_units) is VecUnit, "incorrect type of arg initial_units: should be VecUnit, is {}".format(type(initial_units))

        result = _lib.bc_PlanetMap_initial_units_set(self._ptr, initial_units._ptr)
        _check_errors()
        return result

    def validate(self):
        # type: () -> None
        '''Validates the map and checks some invariants are followed.

         * InvalidMapObject - the planet map is invalid.
        :type self: PlanetMap
        :rtype: None
        '''

        result = _lib.bc_PlanetMap_validate(self._ptr)
        _check_errors()
        return result

    def on_map(self, location):
        # type: (MapLocation) -> bool
        '''Whether a location is on the map.
        :type self: PlanetMap
        :type location: MapLocation
        :rtype: bool
        '''
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))

        result = _lib.bc_PlanetMap_on_map(self._ptr, location._ptr)
        _check_errors()
        result = bool(result)
        return result

    def is_passable_terrain_at(self, location):
        # type: (MapLocation) -> bool
        '''
        Whether the location on the map contains passable terrain. Is only false when the square contains impassable terrain (distinct from containing a building, for instance).

        LocationOffMap - the location is off the map.
        :type self: PlanetMap
        :type location: MapLocation
        :rtype: bool
        '''
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))

        result = _lib.bc_PlanetMap_is_passable_terrain_at(self._ptr, location._ptr)
        _check_errors()
        result = bool(result)
        return result

    def initial_karbonite_at(self, location):
        # type: (MapLocation) -> int
        '''The amount of Karbonite initially deposited at the given location.

        LocationOffMap - the location is off the map.
        :type self: PlanetMap
        :type location: MapLocation
        :rtype: int
        '''
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))

        result = _lib.bc_PlanetMap_initial_karbonite_at(self._ptr, location._ptr)
        _check_errors()
        return result

    def clone(self):
        # type: () -> PlanetMap
        '''Deep-copy a PlanetMap
        :type self: PlanetMap
        :rtype: PlanetMap
        '''

        result = _lib.bc_PlanetMap_clone(self._ptr)
        _check_errors()
        _result = PlanetMap.__new__(PlanetMap)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    @staticmethod
    def from_json(s):
        # type: (str) -> PlanetMap
        '''Deserialize a PlanetMap from a JSON string
        :type s: str
        :rtype: PlanetMap
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_PlanetMap_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = PlanetMap.__new__(PlanetMap)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a PlanetMap to a JSON string
        :type self: PlanetMap
        :rtype: str
        '''

        result = _lib.bc_PlanetMap_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result



class Delta(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> Delta
        '''
        :type self: Delta
        :rtype: Delta
        '''

        ptr = _lib.new_bc_Delta()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: Delta
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_Delta(self._ptr)
            _check_errors()


    @staticmethod
    def from_json(s):
        # type: (str) -> Delta
        '''Deserialize a Delta from a JSON string
        :type s: str
        :rtype: Delta
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_Delta_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = Delta.__new__(Delta)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a Delta to a JSON string
        :type self: Delta
        :rtype: str
        '''

        result = _lib.bc_Delta_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result



class StartGameMessage(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> StartGameMessage
        '''
        :type self: StartGameMessage
        :rtype: StartGameMessage
        '''

        ptr = _lib.new_bc_StartGameMessage()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: StartGameMessage
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_StartGameMessage(self._ptr)
            _check_errors()


    @staticmethod
    def from_json(s):
        # type: (str) -> StartGameMessage
        '''Deserialize a StartGameMessage from a JSON string
        :type s: str
        :rtype: StartGameMessage
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_StartGameMessage_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = StartGameMessage.__new__(StartGameMessage)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a StartGameMessage to a JSON string
        :type self: StartGameMessage
        :rtype: str
        '''

        result = _lib.bc_StartGameMessage_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result



class TurnMessage(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> TurnMessage
        '''
        :type self: TurnMessage
        :rtype: TurnMessage
        '''

        ptr = _lib.new_bc_TurnMessage()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: TurnMessage
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_TurnMessage(self._ptr)
            _check_errors()


    @staticmethod
    def from_json(s):
        # type: (str) -> TurnMessage
        '''Deserialize a TurnMessage from a JSON string
        :type s: str
        :rtype: TurnMessage
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_TurnMessage_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = TurnMessage.__new__(TurnMessage)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a TurnMessage to a JSON string
        :type self: TurnMessage
        :rtype: str
        '''

        result = _lib.bc_TurnMessage_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result



class StartTurnMessage(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> StartTurnMessage
        '''
        :type self: StartTurnMessage
        :rtype: StartTurnMessage
        '''

        ptr = _lib.new_bc_StartTurnMessage()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: StartTurnMessage
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_StartTurnMessage(self._ptr)
            _check_errors()
    @property
    def round(self):
        # type: () -> int
        '''
        :type self: StartTurnMessage
        :rtype: int
        '''

        result = _lib.bc_StartTurnMessage_round_get(self._ptr)
        _check_errors()
        return result

    @round.setter
    def round(self, round):
        # type: (int) -> None
        '''
        :type self: StartTurnMessage
        :type round: int
        :rtype: None
        '''
        assert type(round) is int, "incorrect type of arg round: should be int, is {}".format(type(round))

        result = _lib.bc_StartTurnMessage_round_set(self._ptr, round)
        _check_errors()
        return result

    @staticmethod
    def from_json(s):
        # type: (str) -> StartTurnMessage
        '''Deserialize a StartTurnMessage from a JSON string
        :type s: str
        :rtype: StartTurnMessage
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_StartTurnMessage_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = StartTurnMessage.__new__(StartTurnMessage)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a StartTurnMessage to a JSON string
        :type self: StartTurnMessage
        :rtype: str
        '''

        result = _lib.bc_StartTurnMessage_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result



class ViewerMessage(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> ViewerMessage
        '''
        :type self: ViewerMessage
        :rtype: ViewerMessage
        '''

        ptr = _lib.new_bc_ViewerMessage()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: ViewerMessage
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_ViewerMessage(self._ptr)
            _check_errors()


    @staticmethod
    def from_json(s):
        # type: (str) -> ViewerMessage
        '''Deserialize a ViewerMessage from a JSON string
        :type s: str
        :rtype: ViewerMessage
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_ViewerMessage_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = ViewerMessage.__new__(ViewerMessage)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a ViewerMessage to a JSON string
        :type self: ViewerMessage
        :rtype: str
        '''

        result = _lib.bc_ViewerMessage_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result



class ViewerKeyframe(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> ViewerKeyframe
        '''
        :type self: ViewerKeyframe
        :rtype: ViewerKeyframe
        '''

        ptr = _lib.new_bc_ViewerKeyframe()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: ViewerKeyframe
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_ViewerKeyframe(self._ptr)
            _check_errors()


    @staticmethod
    def from_json(s):
        # type: (str) -> ViewerKeyframe
        '''Deserialize a ViewerKeyframe from a JSON string
        :type s: str
        :rtype: ViewerKeyframe
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_ViewerKeyframe_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = ViewerKeyframe.__new__(ViewerKeyframe)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a ViewerKeyframe to a JSON string
        :type self: ViewerKeyframe
        :rtype: str
        '''

        result = _lib.bc_ViewerKeyframe_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result



class ErrorMessage(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> ErrorMessage
        '''
        :type self: ErrorMessage
        :rtype: ErrorMessage
        '''

        ptr = _lib.new_bc_ErrorMessage()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: ErrorMessage
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_ErrorMessage(self._ptr)
            _check_errors()
    @property
    def error(self):
        # type: () -> str
        '''
        :type self: ErrorMessage
        :rtype: str
        '''

        result = _lib.bc_ErrorMessage_error_get(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    @error.setter
    def error(self, error):
        # type: (str) -> None
        '''
        :type self: ErrorMessage
        :type error: str
        :rtype: None
        '''
        assert type(error) is str, "incorrect type of arg error: should be str, is {}".format(type(error))

        result = _lib.bc_ErrorMessage_error_set(self._ptr, _ffi.new("char[]", error.encode()))
        _check_errors()
        return result

    @staticmethod
    def from_json(s):
        # type: (str) -> ErrorMessage
        '''Deserialize a ErrorMessage from a JSON string
        :type s: str
        :rtype: ErrorMessage
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_ErrorMessage_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = ErrorMessage.__new__(ErrorMessage)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a ErrorMessage to a JSON string
        :type self: ErrorMessage
        :rtype: str
        '''

        result = _lib.bc_ErrorMessage_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    def __repr__(self):
        # type: () -> str
        '''Create a human-readable representation of a ErrorMessage
        :type self: ErrorMessage
        :rtype: str
        '''

        result = _lib.bc_ErrorMessage_debug(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result



class TurnApplication(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> TurnApplication
        '''
        :type self: TurnApplication
        :rtype: TurnApplication
        '''

        ptr = _lib.new_bc_TurnApplication()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: TurnApplication
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_TurnApplication(self._ptr)
            _check_errors()
    @property
    def start_turn(self):
        # type: () -> StartTurnMessage
        '''
        :type self: TurnApplication
        :rtype: StartTurnMessage
        '''

        result = _lib.bc_TurnApplication_start_turn_get(self._ptr)
        _check_errors()
        _result = StartTurnMessage.__new__(StartTurnMessage)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    @property
    def viewer(self):
        # type: () -> ViewerMessage
        '''
        :type self: TurnApplication
        :rtype: ViewerMessage
        '''

        result = _lib.bc_TurnApplication_viewer_get(self._ptr)
        _check_errors()
        _result = ViewerMessage.__new__(ViewerMessage)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    @start_turn.setter
    def start_turn(self, start_turn):
        # type: (StartTurnMessage) -> None
        '''
        :type self: TurnApplication
        :type start_turn: StartTurnMessage
        :rtype: None
        '''
        assert type(start_turn) is StartTurnMessage, "incorrect type of arg start_turn: should be StartTurnMessage, is {}".format(type(start_turn))

        result = _lib.bc_TurnApplication_start_turn_set(self._ptr, start_turn._ptr)
        _check_errors()
        return result

    @viewer.setter
    def viewer(self, viewer):
        # type: (ViewerMessage) -> None
        '''
        :type self: TurnApplication
        :type viewer: ViewerMessage
        :rtype: None
        '''
        assert type(viewer) is ViewerMessage, "incorrect type of arg viewer: should be ViewerMessage, is {}".format(type(viewer))

        result = _lib.bc_TurnApplication_viewer_set(self._ptr, viewer._ptr)
        _check_errors()
        return result




class InitialTurnApplication(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> InitialTurnApplication
        '''
        :type self: InitialTurnApplication
        :rtype: InitialTurnApplication
        '''

        ptr = _lib.new_bc_InitialTurnApplication()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: InitialTurnApplication
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_InitialTurnApplication(self._ptr)
            _check_errors()
    @property
    def start_turn(self):
        # type: () -> StartTurnMessage
        '''
        :type self: InitialTurnApplication
        :rtype: StartTurnMessage
        '''

        result = _lib.bc_InitialTurnApplication_start_turn_get(self._ptr)
        _check_errors()
        _result = StartTurnMessage.__new__(StartTurnMessage)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    @property
    def viewer(self):
        # type: () -> ViewerKeyframe
        '''
        :type self: InitialTurnApplication
        :rtype: ViewerKeyframe
        '''

        result = _lib.bc_InitialTurnApplication_viewer_get(self._ptr)
        _check_errors()
        _result = ViewerKeyframe.__new__(ViewerKeyframe)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    @start_turn.setter
    def start_turn(self, start_turn):
        # type: (StartTurnMessage) -> None
        '''
        :type self: InitialTurnApplication
        :type start_turn: StartTurnMessage
        :rtype: None
        '''
        assert type(start_turn) is StartTurnMessage, "incorrect type of arg start_turn: should be StartTurnMessage, is {}".format(type(start_turn))

        result = _lib.bc_InitialTurnApplication_start_turn_set(self._ptr, start_turn._ptr)
        _check_errors()
        return result

    @viewer.setter
    def viewer(self, viewer):
        # type: (ViewerKeyframe) -> None
        '''
        :type self: InitialTurnApplication
        :type viewer: ViewerKeyframe
        :rtype: None
        '''
        assert type(viewer) is ViewerKeyframe, "incorrect type of arg viewer: should be ViewerKeyframe, is {}".format(type(viewer))

        result = _lib.bc_InitialTurnApplication_viewer_set(self._ptr, viewer._ptr)
        _check_errors()
        return result




class AsteroidStrike(object):
    __slots__ = ['_ptr']
    def __init__(self, karbonite, location):
        # type: (int, MapLocation) -> AsteroidStrike
        '''
        :type self: AsteroidStrike
        :type karbonite: int
        :type location: MapLocation
        :rtype: AsteroidStrike
        '''
        assert type(karbonite) is int, "incorrect type of arg karbonite: should be int, is {}".format(type(karbonite))
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))

        ptr = _lib.new_bc_AsteroidStrike(karbonite, location._ptr)
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: AsteroidStrike
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_AsteroidStrike(self._ptr)
            _check_errors()
    @property
    def karbonite(self):
        # type: () -> int
        '''
        :type self: AsteroidStrike
        :rtype: int
        '''

        result = _lib.bc_AsteroidStrike_karbonite_get(self._ptr)
        _check_errors()
        return result

    @property
    def location(self):
        # type: () -> MapLocation
        '''
        :type self: AsteroidStrike
        :rtype: MapLocation
        '''

        result = _lib.bc_AsteroidStrike_location_get(self._ptr)
        _check_errors()
        _result = MapLocation.__new__(MapLocation)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    @karbonite.setter
    def karbonite(self, karbonite):
        # type: (int) -> None
        '''
        :type self: AsteroidStrike
        :type karbonite: int
        :rtype: None
        '''
        assert type(karbonite) is int, "incorrect type of arg karbonite: should be int, is {}".format(type(karbonite))

        result = _lib.bc_AsteroidStrike_karbonite_set(self._ptr, karbonite)
        _check_errors()
        return result

    @location.setter
    def location(self, location):
        # type: (MapLocation) -> None
        '''
        :type self: AsteroidStrike
        :type location: MapLocation
        :rtype: None
        '''
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))

        result = _lib.bc_AsteroidStrike_location_set(self._ptr, location._ptr)
        _check_errors()
        return result

    def clone(self):
        # type: () -> AsteroidStrike
        '''Deep-copy a AsteroidStrike
        :type self: AsteroidStrike
        :rtype: AsteroidStrike
        '''

        result = _lib.bc_AsteroidStrike_clone(self._ptr)
        _check_errors()
        _result = AsteroidStrike.__new__(AsteroidStrike)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def __repr__(self):
        # type: () -> str
        '''Create a human-readable representation of a AsteroidStrike
        :type self: AsteroidStrike
        :rtype: str
        '''

        result = _lib.bc_AsteroidStrike_debug(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    @staticmethod
    def from_json(s):
        # type: (str) -> AsteroidStrike
        '''Deserialize a AsteroidStrike from a JSON string
        :type s: str
        :rtype: AsteroidStrike
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_AsteroidStrike_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = AsteroidStrike.__new__(AsteroidStrike)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a AsteroidStrike to a JSON string
        :type self: AsteroidStrike
        :rtype: str
        '''

        result = _lib.bc_AsteroidStrike_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    def __eq__(self, other):
        # type: (AsteroidStrike) -> bool
        '''Compare two AsteroidStrikes for deep equality.
        :type self: AsteroidStrike
        :type other: AsteroidStrike
        :rtype: bool
        '''
        assert type(other) is AsteroidStrike, "incorrect type of arg other: should be AsteroidStrike, is {}".format(type(other))

        result = _lib.bc_AsteroidStrike_eq(self._ptr, other._ptr)
        _check_errors()
        result = bool(result)
        return result



class AsteroidPattern(object):
    __slots__ = ['_ptr']
    def __init__(self, seed, mars_map):
        # type: (int, PlanetMap) -> AsteroidPattern
        '''Constructs a pseudorandom asteroid pattern given a map of Mars.
        :type self: AsteroidPattern
        :type seed: int
        :type mars_map: PlanetMap
        :rtype: AsteroidPattern
        '''
        assert type(seed) is int, "incorrect type of arg seed: should be int, is {}".format(type(seed))
        assert type(mars_map) is PlanetMap, "incorrect type of arg mars_map: should be PlanetMap, is {}".format(type(mars_map))

        ptr = _lib.new_bc_AsteroidPattern(seed, mars_map._ptr)
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: AsteroidPattern
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_AsteroidPattern(self._ptr)
            _check_errors()


    def validate(self):
        # type: () -> None
        '''Validates the asteroid pattern.

         * InvalidMapObject - the asteroid pattern is invalid.
        :type self: AsteroidPattern
        :rtype: None
        '''

        result = _lib.bc_AsteroidPattern_validate(self._ptr)
        _check_errors()
        return result

    def has_asteroid(self, round):
        # type: (int) -> bool
        '''Whether there is an asteroid strike at the given round.
        :type self: AsteroidPattern
        :type round: int
        :rtype: bool
        '''
        assert type(round) is int, "incorrect type of arg round: should be int, is {}".format(type(round))

        result = _lib.bc_AsteroidPattern_has_asteroid(self._ptr, round)
        _check_errors()
        result = bool(result)
        return result

    def asteroid(self, round):
        # type: (int) -> AsteroidStrike
        '''Get the asteroid strike at the given round.

         * NullValue - There is no asteroid strike at this round.
        :type self: AsteroidPattern
        :type round: int
        :rtype: AsteroidStrike
        '''
        assert type(round) is int, "incorrect type of arg round: should be int, is {}".format(type(round))

        result = _lib.bc_AsteroidPattern_asteroid(self._ptr, round)
        _check_errors()
        _result = AsteroidStrike.__new__(AsteroidStrike)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def clone(self):
        # type: () -> AsteroidPattern
        '''Deep-copy a AsteroidPattern
        :type self: AsteroidPattern
        :rtype: AsteroidPattern
        '''

        result = _lib.bc_AsteroidPattern_clone(self._ptr)
        _check_errors()
        _result = AsteroidPattern.__new__(AsteroidPattern)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def __repr__(self):
        # type: () -> str
        '''Create a human-readable representation of a AsteroidPattern
        :type self: AsteroidPattern
        :rtype: str
        '''

        result = _lib.bc_AsteroidPattern_debug(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    @staticmethod
    def from_json(s):
        # type: (str) -> AsteroidPattern
        '''Deserialize a AsteroidPattern from a JSON string
        :type s: str
        :rtype: AsteroidPattern
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_AsteroidPattern_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = AsteroidPattern.__new__(AsteroidPattern)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a AsteroidPattern to a JSON string
        :type self: AsteroidPattern
        :rtype: str
        '''

        result = _lib.bc_AsteroidPattern_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result



class OrbitPattern(object):
    __slots__ = ['_ptr']
    def __init__(self, amplitude, period, center):
        # type: (int, int, int) -> OrbitPattern
        '''Construct a new orbit pattern. This pattern is a sinusoidal function y=a*sin(bx)+c, where the x-axis is the round number of takeoff and the the y-axis is the duration of flight to the nearest integer.

        The amplitude, period, and center are measured in rounds.
        :type self: OrbitPattern
        :type amplitude: int
        :type period: int
        :type center: int
        :rtype: OrbitPattern
        '''
        assert type(amplitude) is int, "incorrect type of arg amplitude: should be int, is {}".format(type(amplitude))
        assert type(period) is int, "incorrect type of arg period: should be int, is {}".format(type(period))
        assert type(center) is int, "incorrect type of arg center: should be int, is {}".format(type(center))

        ptr = _lib.new_bc_OrbitPattern(amplitude, period, center)
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: OrbitPattern
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_OrbitPattern(self._ptr)
            _check_errors()
    @property
    def amplitude(self):
        # type: () -> int
        '''Amplitude of the orbit.
        :type self: OrbitPattern
        :rtype: int
        '''

        result = _lib.bc_OrbitPattern_amplitude_get(self._ptr)
        _check_errors()
        return result

    @property
    def period(self):
        # type: () -> int
        '''The period of the orbit.
        :type self: OrbitPattern
        :rtype: int
        '''

        result = _lib.bc_OrbitPattern_period_get(self._ptr)
        _check_errors()
        return result

    @property
    def center(self):
        # type: () -> int
        '''The center of the orbit.
        :type self: OrbitPattern
        :rtype: int
        '''

        result = _lib.bc_OrbitPattern_center_get(self._ptr)
        _check_errors()
        return result

    @amplitude.setter
    def amplitude(self, amplitude):
        # type: (int) -> None
        '''Amplitude of the orbit.
        :type self: OrbitPattern
        :type amplitude: int
        :rtype: None
        '''
        assert type(amplitude) is int, "incorrect type of arg amplitude: should be int, is {}".format(type(amplitude))

        result = _lib.bc_OrbitPattern_amplitude_set(self._ptr, amplitude)
        _check_errors()
        return result

    @period.setter
    def period(self, period):
        # type: (int) -> None
        '''The period of the orbit.
        :type self: OrbitPattern
        :type period: int
        :rtype: None
        '''
        assert type(period) is int, "incorrect type of arg period: should be int, is {}".format(type(period))

        result = _lib.bc_OrbitPattern_period_set(self._ptr, period)
        _check_errors()
        return result

    @center.setter
    def center(self, center):
        # type: (int) -> None
        '''The center of the orbit.
        :type self: OrbitPattern
        :type center: int
        :rtype: None
        '''
        assert type(center) is int, "incorrect type of arg center: should be int, is {}".format(type(center))

        result = _lib.bc_OrbitPattern_center_set(self._ptr, center)
        _check_errors()
        return result

    def validate(self):
        # type: () -> None
        '''Validates the orbit pattern.

         * InvalidMapObject - the orbit pattern is invalid.
        :type self: OrbitPattern
        :rtype: None
        '''

        result = _lib.bc_OrbitPattern_validate(self._ptr)
        _check_errors()
        return result

    def duration(self, round):
        # type: (int) -> int
        '''Get the duration of flight if the rocket were to take off from either planet on the given round.
        :type self: OrbitPattern
        :type round: int
        :rtype: int
        '''
        assert type(round) is int, "incorrect type of arg round: should be int, is {}".format(type(round))

        result = _lib.bc_OrbitPattern_duration(self._ptr, round)
        _check_errors()
        return result

    @staticmethod
    def from_json(s):
        # type: (str) -> OrbitPattern
        '''Deserialize a OrbitPattern from a JSON string
        :type s: str
        :rtype: OrbitPattern
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_OrbitPattern_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = OrbitPattern.__new__(OrbitPattern)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a OrbitPattern to a JSON string
        :type self: OrbitPattern
        :rtype: str
        '''

        result = _lib.bc_OrbitPattern_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result



class GameMap(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> GameMap
        '''The map defining the starting state for an entire game.
        :type self: GameMap
        :rtype: GameMap
        '''

        ptr = _lib.new_bc_GameMap()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: GameMap
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_GameMap(self._ptr)
            _check_errors()
    @property
    def seed(self):
        # type: () -> int
        '''Seed for random number generation.
        :type self: GameMap
        :rtype: int
        '''

        result = _lib.bc_GameMap_seed_get(self._ptr)
        _check_errors()
        return result

    @property
    def earth_map(self):
        # type: () -> PlanetMap
        '''Earth map.
        :type self: GameMap
        :rtype: PlanetMap
        '''

        result = _lib.bc_GameMap_earth_map_get(self._ptr)
        _check_errors()
        _result = PlanetMap.__new__(PlanetMap)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    @property
    def mars_map(self):
        # type: () -> PlanetMap
        '''Mars map.
        :type self: GameMap
        :rtype: PlanetMap
        '''

        result = _lib.bc_GameMap_mars_map_get(self._ptr)
        _check_errors()
        _result = PlanetMap.__new__(PlanetMap)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    @property
    def asteroids(self):
        # type: () -> AsteroidPattern
        '''The asteroid strike pattern on Mars.
        :type self: GameMap
        :rtype: AsteroidPattern
        '''

        result = _lib.bc_GameMap_asteroids_get(self._ptr)
        _check_errors()
        _result = AsteroidPattern.__new__(AsteroidPattern)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    @property
    def orbit(self):
        # type: () -> OrbitPattern
        '''The orbit pattern that determines a rocket's flight duration.
        :type self: GameMap
        :rtype: OrbitPattern
        '''

        result = _lib.bc_GameMap_orbit_get(self._ptr)
        _check_errors()
        _result = OrbitPattern.__new__(OrbitPattern)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    @seed.setter
    def seed(self, seed):
        # type: (int) -> None
        '''Seed for random number generation.
        :type self: GameMap
        :type seed: int
        :rtype: None
        '''
        assert type(seed) is int, "incorrect type of arg seed: should be int, is {}".format(type(seed))

        result = _lib.bc_GameMap_seed_set(self._ptr, seed)
        _check_errors()
        return result

    @earth_map.setter
    def earth_map(self, earth_map):
        # type: (PlanetMap) -> None
        '''Earth map.
        :type self: GameMap
        :type earth_map: PlanetMap
        :rtype: None
        '''
        assert type(earth_map) is PlanetMap, "incorrect type of arg earth_map: should be PlanetMap, is {}".format(type(earth_map))

        result = _lib.bc_GameMap_earth_map_set(self._ptr, earth_map._ptr)
        _check_errors()
        return result

    @mars_map.setter
    def mars_map(self, mars_map):
        # type: (PlanetMap) -> None
        '''Mars map.
        :type self: GameMap
        :type mars_map: PlanetMap
        :rtype: None
        '''
        assert type(mars_map) is PlanetMap, "incorrect type of arg mars_map: should be PlanetMap, is {}".format(type(mars_map))

        result = _lib.bc_GameMap_mars_map_set(self._ptr, mars_map._ptr)
        _check_errors()
        return result

    @asteroids.setter
    def asteroids(self, asteroids):
        # type: (AsteroidPattern) -> None
        '''The asteroid strike pattern on Mars.
        :type self: GameMap
        :type asteroids: AsteroidPattern
        :rtype: None
        '''
        assert type(asteroids) is AsteroidPattern, "incorrect type of arg asteroids: should be AsteroidPattern, is {}".format(type(asteroids))

        result = _lib.bc_GameMap_asteroids_set(self._ptr, asteroids._ptr)
        _check_errors()
        return result

    @orbit.setter
    def orbit(self, orbit):
        # type: (OrbitPattern) -> None
        '''The orbit pattern that determines a rocket's flight duration.
        :type self: GameMap
        :type orbit: OrbitPattern
        :rtype: None
        '''
        assert type(orbit) is OrbitPattern, "incorrect type of arg orbit: should be OrbitPattern, is {}".format(type(orbit))

        result = _lib.bc_GameMap_orbit_set(self._ptr, orbit._ptr)
        _check_errors()
        return result

    def validate(self):
        # type: () -> None
        '''Validate the game map.

         * InvalidMapObject - the game map is invalid.
        :type self: GameMap
        :rtype: None
        '''

        result = _lib.bc_GameMap_validate(self._ptr)
        _check_errors()
        return result

    @staticmethod
    def test_map():
        # type: () -> GameMap
        '''
        :rtype: GameMap
        '''

        result = _lib.bc_GameMap_test_map()
        _check_errors()
        _result = GameMap.__new__(GameMap)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def clone(self):
        # type: () -> GameMap
        '''Deep-copy a GameMap
        :type self: GameMap
        :rtype: GameMap
        '''

        result = _lib.bc_GameMap_clone(self._ptr)
        _check_errors()
        _result = GameMap.__new__(GameMap)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    @staticmethod
    def from_json(s):
        # type: (str) -> GameMap
        '''Deserialize a GameMap from a JSON string
        :type s: str
        :rtype: GameMap
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_GameMap_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = GameMap.__new__(GameMap)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a GameMap to a JSON string
        :type self: GameMap
        :rtype: str
        '''

        result = _lib.bc_GameMap_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result



def max_level(branch):
    # type: (UnitType) -> int
    '''
    :type branch: UnitType
    :rtype: int
    '''
    assert type(branch) is UnitType, "incorrect type of arg branch: should be UnitType, is {}".format(type(branch))

    result = _lib.max_level(branch)
    _check_errors()
    return result

def cost_of(branch, level):
    # type: (UnitType, int) -> int
    '''
    :type branch: UnitType
    :type level: int
    :rtype: int
    '''
    assert type(branch) is UnitType, "incorrect type of arg branch: should be UnitType, is {}".format(type(branch))
    assert type(level) is int, "incorrect type of arg level: should be int, is {}".format(type(level))

    result = _lib.cost_of(branch, level)
    _check_errors()
    return result

class ResearchInfo(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> ResearchInfo
        '''Construct an initial research state.
        :type self: ResearchInfo
        :rtype: ResearchInfo
        '''

        ptr = _lib.new_bc_ResearchInfo()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: ResearchInfo
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_ResearchInfo(self._ptr)
            _check_errors()


    def get_level(self, branch):
        # type: (UnitType) -> int
        '''Returns the current level of the research branch.
        :type self: ResearchInfo
        :type branch: UnitType
        :rtype: int
        '''
        assert type(branch) is UnitType, "incorrect type of arg branch: should be UnitType, is {}".format(type(branch))

        result = _lib.bc_ResearchInfo_get_level(self._ptr, branch)
        _check_errors()
        return result

    @property
    def queue(self):
        # type: () -> VecUnitType
        '''Returns the research queue, where the front of the queue is at the beginning of the list.
        :type self: ResearchInfo
        :rtype: VecUnitType
        '''

        result = _lib.bc_ResearchInfo_queue(self._ptr)
        _check_errors()
        _result = VecUnitType.__new__(VecUnitType)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def has_next_in_queue(self):
        # type: () -> bool
        '''Whether there is a branch in the research queue.
        :type self: ResearchInfo
        :rtype: bool
        '''

        result = _lib.bc_ResearchInfo_has_next_in_queue(self._ptr)
        _check_errors()
        result = bool(result)
        return result

    def next_in_queue(self):
        # type: () -> UnitType
        '''Returns the next branch to be researched, which is the branch at the front of the research queue.

         * NullValue - There is no branch to be researched.
        :type self: ResearchInfo
        :rtype: UnitType
        '''

        result = _lib.bc_ResearchInfo_next_in_queue(self._ptr)
        _check_errors()
        result = UnitType(result)
        return result

    def rounds_left(self):
        # type: () -> int
        '''Returns the number of rounds left until the upgrade at the front of the research queue is applied.

         * NullValue - There is no branch to be researched.
        :type self: ResearchInfo
        :rtype: int
        '''

        result = _lib.bc_ResearchInfo_rounds_left(self._ptr)
        _check_errors()
        return result

    @staticmethod
    def from_json(s):
        # type: (str) -> ResearchInfo
        '''Deserialize a ResearchInfo from a JSON string
        :type s: str
        :rtype: ResearchInfo
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_ResearchInfo_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = ResearchInfo.__new__(ResearchInfo)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a ResearchInfo to a JSON string
        :type self: ResearchInfo
        :rtype: str
        '''

        result = _lib.bc_ResearchInfo_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result



class RocketLanding(object):
    __slots__ = ['_ptr']
    def __init__(self, rocket_id, destination):
        # type: (int, MapLocation) -> RocketLanding
        '''
        :type self: RocketLanding
        :type rocket_id: int
        :type destination: MapLocation
        :rtype: RocketLanding
        '''
        assert type(rocket_id) is int, "incorrect type of arg rocket_id: should be int, is {}".format(type(rocket_id))
        assert type(destination) is MapLocation, "incorrect type of arg destination: should be MapLocation, is {}".format(type(destination))

        ptr = _lib.new_bc_RocketLanding(rocket_id, destination._ptr)
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: RocketLanding
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_RocketLanding(self._ptr)
            _check_errors()
    @property
    def rocket_id(self):
        # type: () -> int
        '''The ID of the rocket.
        :type self: RocketLanding
        :rtype: int
        '''

        result = _lib.bc_RocketLanding_rocket_id_get(self._ptr)
        _check_errors()
        return result

    @property
    def destination(self):
        # type: () -> MapLocation
        '''The landing destination of the rocket.
        :type self: RocketLanding
        :rtype: MapLocation
        '''

        result = _lib.bc_RocketLanding_destination_get(self._ptr)
        _check_errors()
        _result = MapLocation.__new__(MapLocation)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    @rocket_id.setter
    def rocket_id(self, rocket_id):
        # type: (int) -> None
        '''The ID of the rocket.
        :type self: RocketLanding
        :type rocket_id: int
        :rtype: None
        '''
        assert type(rocket_id) is int, "incorrect type of arg rocket_id: should be int, is {}".format(type(rocket_id))

        result = _lib.bc_RocketLanding_rocket_id_set(self._ptr, rocket_id)
        _check_errors()
        return result

    @destination.setter
    def destination(self, destination):
        # type: (MapLocation) -> None
        '''The landing destination of the rocket.
        :type self: RocketLanding
        :type destination: MapLocation
        :rtype: None
        '''
        assert type(destination) is MapLocation, "incorrect type of arg destination: should be MapLocation, is {}".format(type(destination))

        result = _lib.bc_RocketLanding_destination_set(self._ptr, destination._ptr)
        _check_errors()
        return result

    def clone(self):
        # type: () -> RocketLanding
        '''Deep-copy a RocketLanding
        :type self: RocketLanding
        :rtype: RocketLanding
        '''

        result = _lib.bc_RocketLanding_clone(self._ptr)
        _check_errors()
        _result = RocketLanding.__new__(RocketLanding)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def __repr__(self):
        # type: () -> str
        '''Create a human-readable representation of a RocketLanding
        :type self: RocketLanding
        :rtype: str
        '''

        result = _lib.bc_RocketLanding_debug(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    @staticmethod
    def from_json(s):
        # type: (str) -> RocketLanding
        '''Deserialize a RocketLanding from a JSON string
        :type s: str
        :rtype: RocketLanding
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_RocketLanding_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = RocketLanding.__new__(RocketLanding)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a RocketLanding to a JSON string
        :type self: RocketLanding
        :rtype: str
        '''

        result = _lib.bc_RocketLanding_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    def __eq__(self, other):
        # type: (RocketLanding) -> bool
        '''Compare two RocketLandings for deep equality.
        :type self: RocketLanding
        :type other: RocketLanding
        :rtype: bool
        '''
        assert type(other) is RocketLanding, "incorrect type of arg other: should be RocketLanding, is {}".format(type(other))

        result = _lib.bc_RocketLanding_eq(self._ptr, other._ptr)
        _check_errors()
        result = bool(result)
        return result



class VecRocketLanding(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> VecRocketLanding
        '''An immutable list of bc::rockets::RocketLanding objects
        :type self: VecRocketLanding
        :rtype: VecRocketLanding
        '''

        ptr = _lib.new_bc_VecRocketLanding()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: VecRocketLanding
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_VecRocketLanding(self._ptr)
            _check_errors()


    def __repr__(self):
        # type: () -> str
        '''Create a human-readable representation of a VecRocketLanding
        :type self: VecRocketLanding
        :rtype: str
        '''

        result = _lib.bc_VecRocketLanding_debug(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    def clone(self):
        # type: () -> VecRocketLanding
        '''Deep-copy a VecRocketLanding
        :type self: VecRocketLanding
        :rtype: VecRocketLanding
        '''

        result = _lib.bc_VecRocketLanding_clone(self._ptr)
        _check_errors()
        _result = VecRocketLanding.__new__(VecRocketLanding)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def __len__(self):
        # type: () -> int
        '''The length of the vector.
        :type self: VecRocketLanding
        :rtype: int
        '''

        result = _lib.bc_VecRocketLanding_len(self._ptr)
        _check_errors()
        return result

    def __getitem__(self, index):
        # type: (int) -> RocketLanding
        '''Copy an element out of the vector.
        :type self: VecRocketLanding
        :type index: int
        :rtype: RocketLanding
        '''
        assert type(index) is int, "incorrect type of arg index: should be int, is {}".format(type(index))

        result = _lib.bc_VecRocketLanding_index(self._ptr, index)
        _check_errors()
        _result = RocketLanding.__new__(RocketLanding)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result


    def __iter__(self):
        l = len(self)
        for i in range(l):
            yield self[i]

class RocketLandingInfo(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> RocketLandingInfo
        '''Construct an empty rocket landing info.
        :type self: RocketLandingInfo
        :rtype: RocketLandingInfo
        '''

        ptr = _lib.new_bc_RocketLandingInfo()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: RocketLandingInfo
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_RocketLandingInfo(self._ptr)
            _check_errors()


    def landings_on(self, round):
        # type: (int) -> VecRocketLanding
        '''Get the rocket landings on this round.
        :type self: RocketLandingInfo
        :type round: int
        :rtype: VecRocketLanding
        '''
        assert type(round) is int, "incorrect type of arg round: should be int, is {}".format(type(round))

        result = _lib.bc_RocketLandingInfo_landings_on(self._ptr, round)
        _check_errors()
        _result = VecRocketLanding.__new__(VecRocketLanding)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def clone(self):
        # type: () -> RocketLandingInfo
        '''Deep-copy a RocketLandingInfo
        :type self: RocketLandingInfo
        :rtype: RocketLandingInfo
        '''

        result = _lib.bc_RocketLandingInfo_clone(self._ptr)
        _check_errors()
        _result = RocketLandingInfo.__new__(RocketLandingInfo)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def __repr__(self):
        # type: () -> str
        '''Create a human-readable representation of a RocketLandingInfo
        :type self: RocketLandingInfo
        :rtype: str
        '''

        result = _lib.bc_RocketLandingInfo_debug(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    @staticmethod
    def from_json(s):
        # type: (str) -> RocketLandingInfo
        '''Deserialize a RocketLandingInfo from a JSON string
        :type s: str
        :rtype: RocketLandingInfo
        '''
        assert type(s) is str, "incorrect type of arg s: should be str, is {}".format(type(s))

        result = _lib.bc_RocketLandingInfo_from_json(_ffi.new("char[]", s.encode()))
        _check_errors()
        _result = RocketLandingInfo.__new__(RocketLandingInfo)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def to_json(self):
        # type: () -> str
        '''Serialize a RocketLandingInfo to a JSON string
        :type self: RocketLandingInfo
        :rtype: str
        '''

        result = _lib.bc_RocketLandingInfo_to_json(self._ptr)
        _check_errors()
        _result = _ffi.string(result)
        _lib.bc_free_string(result)
        result = _result.decode()
        return result

    def __eq__(self, other):
        # type: (RocketLandingInfo) -> bool
        '''Compare two RocketLandingInfos for deep equality.
        :type self: RocketLandingInfo
        :type other: RocketLandingInfo
        :rtype: bool
        '''
        assert type(other) is RocketLandingInfo, "incorrect type of arg other: should be RocketLandingInfo, is {}".format(type(other))

        result = _lib.bc_RocketLandingInfo_eq(self._ptr, other._ptr)
        _check_errors()
        result = bool(result)
        return result



class GameController(object):
    __slots__ = ['_ptr']
    def __init__(self):
        # type: () -> GameController
        '''Use environment variables to connect to the manager.
        :type self: GameController
        :rtype: GameController
        '''

        ptr = _lib.new_bc_GameController()
        if ptr != _ffi.NULL: self._ptr = ptr
        _check_errors()

    def __del__(self):
        # type: () -> None
        '''Clean up the object.
        :type self: GameController
        :rtype: None
        '''

        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bc_GameController(self._ptr)
            _check_errors()


    def next_turn(self):
        # type: () -> None
        '''Send the moves from the current turn and wait for the next turn.
        :type self: GameController
        :rtype: None
        '''

        result = _lib.bc_GameController_next_turn(self._ptr)
        _check_errors()
        return result

    def round(self):
        # type: () -> int
        '''The current round, starting at round 1 and up to ROUND_LIMIT rounds. A round consists of a turn from each team on each planet.
        :type self: GameController
        :rtype: int
        '''

        result = _lib.bc_GameController_round(self._ptr)
        _check_errors()
        return result

    def planet(self):
        # type: () -> Planet
        '''The current planet.
        :type self: GameController
        :rtype: Planet
        '''

        result = _lib.bc_GameController_planet(self._ptr)
        _check_errors()
        result = Planet(result)
        return result

    def team(self):
        # type: () -> Team
        '''The team whose turn it is.
        :type self: GameController
        :rtype: Team
        '''

        result = _lib.bc_GameController_team(self._ptr)
        _check_errors()
        result = Team(result)
        return result

    def starting_map(self, planet):
        # type: (Planet) -> PlanetMap
        '''The starting map of the given planet. Includes the map's planet, dimensions, impassable terrain, and initial units and karbonite.
        :type self: GameController
        :type planet: Planet
        :rtype: PlanetMap
        '''
        assert type(planet) is Planet, "incorrect type of arg planet: should be Planet, is {}".format(type(planet))

        result = _lib.bc_GameController_starting_map(self._ptr, planet)
        _check_errors()
        _result = PlanetMap.__new__(PlanetMap)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def karbonite(self):
        # type: () -> int
        '''The karbonite in the team's resource pool.
        :type self: GameController
        :rtype: int
        '''

        result = _lib.bc_GameController_karbonite(self._ptr)
        _check_errors()
        return result

    def unit(self, id):
        # type: (int) -> Unit
        '''The single unit with this ID. Use this method to get detailed statistics on a unit - heat, cooldowns, and properties of special abilities like units garrisoned in a rocket.

        * NoSuchUnit - the unit does not exist (inside the vision range).
        :type self: GameController
        :type id: int
        :rtype: Unit
        '''
        assert type(id) is int, "incorrect type of arg id: should be int, is {}".format(type(id))

        result = _lib.bc_GameController_unit(self._ptr, id)
        _check_errors()
        _result = Unit.__new__(Unit)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def units(self):
        # type: () -> VecUnit
        '''All the units within the vision range, in no particular order. Does not include units in space.
        :type self: GameController
        :rtype: VecUnit
        '''

        result = _lib.bc_GameController_units(self._ptr)
        _check_errors()
        _result = VecUnit.__new__(VecUnit)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def my_units(self):
        # type: () -> VecUnit
        '''All the units on your team. Does not include units in space.
        :type self: GameController
        :rtype: VecUnit
        '''

        result = _lib.bc_GameController_my_units(self._ptr)
        _check_errors()
        _result = VecUnit.__new__(VecUnit)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def units_in_space(self):
        # type: () -> VecUnit
        '''All the units of this team that are in space. You cannot see units on the other team that are in space.
        :type self: GameController
        :rtype: VecUnit
        '''

        result = _lib.bc_GameController_units_in_space(self._ptr)
        _check_errors()
        _result = VecUnit.__new__(VecUnit)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def karbonite_at(self, location):
        # type: (MapLocation) -> int
        '''The karbonite at the given location.

        * LocationOffMap - the location is off the map.
        * LocationNotVisible - the location is outside the vision range.
        :type self: GameController
        :type location: MapLocation
        :rtype: int
        '''
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))

        result = _lib.bc_GameController_karbonite_at(self._ptr, location._ptr)
        _check_errors()
        return result

    def all_locations_within(self, location, radius_squared):
        # type: (MapLocation, int) -> VecMapLocation
        '''Returns an array of all locations within a certain radius squared of this location that are on the map.

        The locations are ordered first by the x-coordinate, then the y-coordinate. The radius squared is inclusive.
        :type self: GameController
        :type location: MapLocation
        :type radius_squared: int
        :rtype: VecMapLocation
        '''
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))
        assert type(radius_squared) is int, "incorrect type of arg radius_squared: should be int, is {}".format(type(radius_squared))

        result = _lib.bc_GameController_all_locations_within(self._ptr, location._ptr, radius_squared)
        _check_errors()
        _result = VecMapLocation.__new__(VecMapLocation)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def can_sense_location(self, location):
        # type: (MapLocation) -> bool
        '''Whether the location is on the map and within the vision range.
        :type self: GameController
        :type location: MapLocation
        :rtype: bool
        '''
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))

        result = _lib.bc_GameController_can_sense_location(self._ptr, location._ptr)
        _check_errors()
        result = bool(result)
        return result

    def can_sense_unit(self, id):
        # type: (int) -> bool
        '''Whether there is a unit with this ID within the vision range.
        :type self: GameController
        :type id: int
        :rtype: bool
        '''
        assert type(id) is int, "incorrect type of arg id: should be int, is {}".format(type(id))

        result = _lib.bc_GameController_can_sense_unit(self._ptr, id)
        _check_errors()
        result = bool(result)
        return result

    def sense_nearby_units(self, location, radius):
        # type: (MapLocation, int) -> VecUnit
        '''Sense units near the location within the given radius, inclusive, in distance squared. The units are within the vision range.
        :type self: GameController
        :type location: MapLocation
        :type radius: int
        :rtype: VecUnit
        '''
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))
        assert type(radius) is int, "incorrect type of arg radius: should be int, is {}".format(type(radius))

        result = _lib.bc_GameController_sense_nearby_units(self._ptr, location._ptr, radius)
        _check_errors()
        _result = VecUnit.__new__(VecUnit)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def sense_nearby_units_by_team(self, location, radius, team):
        # type: (MapLocation, int, Team) -> VecUnit
        '''Sense units near the location within the given radius, inclusive, in distance squared. The units are within the vision range. Additionally filters the units by team.
        :type self: GameController
        :type location: MapLocation
        :type radius: int
        :type team: Team
        :rtype: VecUnit
        '''
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))
        assert type(radius) is int, "incorrect type of arg radius: should be int, is {}".format(type(radius))
        assert type(team) is Team, "incorrect type of arg team: should be Team, is {}".format(type(team))

        result = _lib.bc_GameController_sense_nearby_units_by_team(self._ptr, location._ptr, radius, team)
        _check_errors()
        _result = VecUnit.__new__(VecUnit)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def sense_nearby_units_by_type(self, location, radius, unit_type):
        # type: (MapLocation, int, UnitType) -> VecUnit
        '''Sense units near the location within the given radius, inclusive, in distance squared. The units are within the vision range. Additionally filters the units by unit type.
        :type self: GameController
        :type location: MapLocation
        :type radius: int
        :type unit_type: UnitType
        :rtype: VecUnit
        '''
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))
        assert type(radius) is int, "incorrect type of arg radius: should be int, is {}".format(type(radius))
        assert type(unit_type) is UnitType, "incorrect type of arg unit_type: should be UnitType, is {}".format(type(unit_type))

        result = _lib.bc_GameController_sense_nearby_units_by_type(self._ptr, location._ptr, radius, unit_type)
        _check_errors()
        _result = VecUnit.__new__(VecUnit)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def has_unit_at_location(self, location):
        # type: (MapLocation) -> bool
        '''Whether there is a visible unit at a location.
        :type self: GameController
        :type location: MapLocation
        :rtype: bool
        '''
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))

        result = _lib.bc_GameController_has_unit_at_location(self._ptr, location._ptr)
        _check_errors()
        result = bool(result)
        return result

    def sense_unit_at_location(self, location):
        # type: (MapLocation) -> Unit
        '''The unit at the location, if it exists.

        * LocationOffMap - the location is off the map.
        * LocationNotVisible - the location is outside the vision range.
        :type self: GameController
        :type location: MapLocation
        :rtype: Unit
        '''
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))

        result = _lib.bc_GameController_sense_unit_at_location(self._ptr, location._ptr)
        _check_errors()
        _result = Unit.__new__(Unit)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def asteroid_pattern(self):
        # type: () -> AsteroidPattern
        '''The asteroid strike pattern on Mars.
        :type self: GameController
        :rtype: AsteroidPattern
        '''

        result = _lib.bc_GameController_asteroid_pattern(self._ptr)
        _check_errors()
        _result = AsteroidPattern.__new__(AsteroidPattern)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def orbit_pattern(self):
        # type: () -> OrbitPattern
        '''The orbit pattern that determines a rocket's flight duration.
        :type self: GameController
        :rtype: OrbitPattern
        '''

        result = _lib.bc_GameController_orbit_pattern(self._ptr)
        _check_errors()
        _result = OrbitPattern.__new__(OrbitPattern)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def current_duration_of_flight(self):
        # type: () -> int
        '''The current duration of flight if a rocket were to be launched this round. Does not take into account any research done on rockets.
        :type self: GameController
        :rtype: int
        '''

        result = _lib.bc_GameController_current_duration_of_flight(self._ptr)
        _check_errors()
        return result

    def get_team_array(self, planet):
        # type: (Planet) -> Veci32
        '''Gets a read-only version of this planet's team array. If the given planet is different from the planet of the player, reads the version of the planet's team array from COMMUNICATION_DELAY rounds prior.
        :type self: GameController
        :type planet: Planet
        :rtype: Veci32
        '''
        assert type(planet) is Planet, "incorrect type of arg planet: should be Planet, is {}".format(type(planet))

        result = _lib.bc_GameController_get_team_array(self._ptr, planet)
        _check_errors()
        _result = Veci32.__new__(Veci32)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def write_team_array(self, index, value):
        # type: (int, int) -> None
        '''Writes the value at the index of this planet's team array.

        * ArrayOutOfBounds - the index of the array is out of bounds. It must be within [0, COMMUNICATION_ARRAY_LENGTH).
        :type self: GameController
        :type index: int
        :type value: int
        :rtype: None
        '''
        assert type(index) is int, "incorrect type of arg index: should be int, is {}".format(type(index))
        assert type(value) is int, "incorrect type of arg value: should be int, is {}".format(type(value))

        result = _lib.bc_GameController_write_team_array(self._ptr, index, value)
        _check_errors()
        return result

    def disintegrate_unit(self, unit_id):
        # type: (int) -> None
        '''Disintegrates the unit and removes it from the map. If the unit is a factory or a rocket, also disintegrates any units garrisoned inside it.

        * NoSuchUnit - the unit does not exist (inside the vision range).
        * TeamNotAllowed - the unit is not on the current player's team.
        :type self: GameController
        :type unit_id: int
        :rtype: None
        '''
        assert type(unit_id) is int, "incorrect type of arg unit_id: should be int, is {}".format(type(unit_id))

        result = _lib.bc_GameController_disintegrate_unit(self._ptr, unit_id)
        _check_errors()
        return result

    def is_occupiable(self, location):
        # type: (MapLocation) -> bool
        '''Whether the location is clear for a unit to occupy, either by movement or by construction.

        * LocationOffMap - the location is off the map.
        * LocationNotVisible - the location is outside the vision range.
        :type self: GameController
        :type location: MapLocation
        :rtype: bool
        '''
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))

        result = _lib.bc_GameController_is_occupiable(self._ptr, location._ptr)
        _check_errors()
        result = bool(result)
        return result

    def can_move(self, robot_id, direction):
        # type: (int, Direction) -> bool
        '''Whether the robot can move in the given direction, without taking into account the unit's movement heat. Takes into account only the map terrain, positions of other robots, and the edge of the game map.
        :type self: GameController
        :type robot_id: int
        :type direction: Direction
        :rtype: bool
        '''
        assert type(robot_id) is int, "incorrect type of arg robot_id: should be int, is {}".format(type(robot_id))
        assert type(direction) is Direction, "incorrect type of arg direction: should be Direction, is {}".format(type(direction))

        result = _lib.bc_GameController_can_move(self._ptr, robot_id, direction)
        _check_errors()
        result = bool(result)
        return result

    def is_move_ready(self, robot_id):
        # type: (int) -> bool
        '''Whether the robot is ready to move. Tests whether the robot's attack heat is sufficiently low.
        :type self: GameController
        :type robot_id: int
        :rtype: bool
        '''
        assert type(robot_id) is int, "incorrect type of arg robot_id: should be int, is {}".format(type(robot_id))

        result = _lib.bc_GameController_is_move_ready(self._ptr, robot_id)
        _check_errors()
        result = bool(result)
        return result

    def move_robot(self, robot_id, direction):
        # type: (int, Direction) -> None
        '''Moves the robot in the given direction.

        * NoSuchUnit - the robot does not exist (within the vision range).
        * TeamNotAllowed - the robot is not on the current player's team.
        * UnitNotOnMap - the robot is not on the map.
        * LocationNotVisible - the location is outside the vision range.
        * LocationOffMap - the location is off the map.
        * LocationNotEmpty - the location is occupied by a unit or terrain.
        * Overheated - the robot is not ready to move again.
        :type self: GameController
        :type robot_id: int
        :type direction: Direction
        :rtype: None
        '''
        assert type(robot_id) is int, "incorrect type of arg robot_id: should be int, is {}".format(type(robot_id))
        assert type(direction) is Direction, "incorrect type of arg direction: should be Direction, is {}".format(type(direction))

        result = _lib.bc_GameController_move_robot(self._ptr, robot_id, direction)
        _check_errors()
        return result

    def can_attack(self, robot_id, target_unit_id):
        # type: (int, int) -> bool
        '''Whether the robot can attack the given unit, without taking into account the robot's attack heat. Takes into account only the robot's attack range, and the location of the robot and target.

        Healers cannot attack, and should use can_heal() instead.
        :type self: GameController
        :type robot_id: int
        :type target_unit_id: int
        :rtype: bool
        '''
        assert type(robot_id) is int, "incorrect type of arg robot_id: should be int, is {}".format(type(robot_id))
        assert type(target_unit_id) is int, "incorrect type of arg target_unit_id: should be int, is {}".format(type(target_unit_id))

        result = _lib.bc_GameController_can_attack(self._ptr, robot_id, target_unit_id)
        _check_errors()
        result = bool(result)
        return result

    def is_attack_ready(self, robot_id):
        # type: (int) -> bool
        '''Whether the robot is ready to attack. Tests whether the robot's attack heat is sufficiently low.

        Healers cannot attack, and should use is_heal_ready() instead.
        :type self: GameController
        :type robot_id: int
        :rtype: bool
        '''
        assert type(robot_id) is int, "incorrect type of arg robot_id: should be int, is {}".format(type(robot_id))

        result = _lib.bc_GameController_is_attack_ready(self._ptr, robot_id)
        _check_errors()
        result = bool(result)
        return result

    def attack(self, robot_id, target_unit_id):
        # type: (int, int) -> None
        '''Commands a robot to attack a unit, dealing the robot's standard amount of damage.

        Healers cannot attack, and should use heal() instead.

        * NoSuchUnit - the unit does not exist (inside the vision range).
        * TeamNotAllowed - the unit is not on the current player's team.
        * InappropriateUnitType - the unit is not a robot, or is a healer.
        * UnitNotOnMap - the unit or target is not on the map.
        * OutOfRange - the target location is not in range.
        * Overheated - the unit is not ready to attack.
        :type self: GameController
        :type robot_id: int
        :type target_unit_id: int
        :rtype: None
        '''
        assert type(robot_id) is int, "incorrect type of arg robot_id: should be int, is {}".format(type(robot_id))
        assert type(target_unit_id) is int, "incorrect type of arg target_unit_id: should be int, is {}".format(type(target_unit_id))

        result = _lib.bc_GameController_attack(self._ptr, robot_id, target_unit_id)
        _check_errors()
        return result

    def research_info(self):
        # type: () -> ResearchInfo
        '''The research info of the current team, including what branch is currently being researched, the number of rounds left.
        :type self: GameController
        :rtype: ResearchInfo
        '''

        result = _lib.bc_GameController_research_info(self._ptr)
        _check_errors()
        _result = ResearchInfo.__new__(ResearchInfo)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def reset_research(self):
        # type: () -> bool
        '''Resets the research queue to be empty. Returns true if the queue was not empty before, and false otherwise.
        :type self: GameController
        :rtype: bool
        '''

        result = _lib.bc_GameController_reset_research(self._ptr)
        _check_errors()
        result = bool(result)
        return result

    def queue_research(self, branch):
        # type: (UnitType) -> bool
        '''Adds a branch to the back of the queue, if it is a valid upgrade, and starts research if it is the first in the queue.

        Returns whether the branch was successfully added.
        :type self: GameController
        :type branch: UnitType
        :rtype: bool
        '''
        assert type(branch) is UnitType, "incorrect type of arg branch: should be UnitType, is {}".format(type(branch))

        result = _lib.bc_GameController_queue_research(self._ptr, branch)
        _check_errors()
        result = bool(result)
        return result

    def can_harvest(self, worker_id, direction):
        # type: (int, Direction) -> bool
        '''Whether the worker is ready to harvest, and the given direction contains karbonite to harvest. The worker cannot already have performed an action this round.
        :type self: GameController
        :type worker_id: int
        :type direction: Direction
        :rtype: bool
        '''
        assert type(worker_id) is int, "incorrect type of arg worker_id: should be int, is {}".format(type(worker_id))
        assert type(direction) is Direction, "incorrect type of arg direction: should be Direction, is {}".format(type(direction))

        result = _lib.bc_GameController_can_harvest(self._ptr, worker_id, direction)
        _check_errors()
        result = bool(result)
        return result

    def harvest(self, worker_id, direction):
        # type: (int, Direction) -> None
        '''Harvests up to the worker's harvest amount of karbonite from the given location, adding it to the team's resource pool.

        * NoSuchUnit - the worker does not exist (within the vision range).
        * TeamNotAllowed - the worker is not on the current player's team.
        * InappropriateUnitType - the unit is not a worker.
        * Overheated - the worker has already performed an action this turn.
        * UnitNotOnMap - the worker is not on the map.
        * LocationOffMap - the location in the target direction is off the map.
        * LocationNotVisible - the location is not in the vision range.
        * KarboniteDepositEmpty - the location described contains no Karbonite.
        :type self: GameController
        :type worker_id: int
        :type direction: Direction
        :rtype: None
        '''
        assert type(worker_id) is int, "incorrect type of arg worker_id: should be int, is {}".format(type(worker_id))
        assert type(direction) is Direction, "incorrect type of arg direction: should be Direction, is {}".format(type(direction))

        result = _lib.bc_GameController_harvest(self._ptr, worker_id, direction)
        _check_errors()
        return result

    def can_blueprint(self, worker_id, unit_type, direction):
        # type: (int, UnitType, Direction) -> bool
        '''Whether the worker can blueprint a unit of the given type. The worker can only blueprint factories, and rockets if Rocketry has been researched. The team must have sufficient karbonite in its resource pool. The worker cannot already have performed an action this round.
        :type self: GameController
        :type worker_id: int
        :type unit_type: UnitType
        :type direction: Direction
        :rtype: bool
        '''
        assert type(worker_id) is int, "incorrect type of arg worker_id: should be int, is {}".format(type(worker_id))
        assert type(unit_type) is UnitType, "incorrect type of arg unit_type: should be UnitType, is {}".format(type(unit_type))
        assert type(direction) is Direction, "incorrect type of arg direction: should be Direction, is {}".format(type(direction))

        result = _lib.bc_GameController_can_blueprint(self._ptr, worker_id, unit_type, direction)
        _check_errors()
        result = bool(result)
        return result

    def blueprint(self, worker_id, structure_type, direction):
        # type: (int, UnitType, Direction) -> None
        '''Blueprints a unit of the given type in the given direction. Subtract cost of that unit from the team's resource pool.

        * NoSuchUnit - the worker does not exist (within the vision range).
        * TeamNotAllowed - the worker is not on the current player's team.
        * InappropriateUnitType - the unit is not a worker, or the unit type is not a structure.
        * Overheated - the worker has already performed an action this turn.
        * UnitNotOnMap - the unit is not on the map.
        * LocationOffMap - the location in the target direction is off the map.
        * LocationNotVisible - the location is outside the vision range.
        * LocationNotEmpty - the location in the target direction is already occupied.
        * CannotBuildOnMars - you cannot blueprint a structure on Mars.
        * ResearchNotUnlocked - you do not have the needed research to blueprint rockets.
        * InsufficientKarbonite - your team does not have enough Karbonite to build the requested structure.
        :type self: GameController
        :type worker_id: int
        :type structure_type: UnitType
        :type direction: Direction
        :rtype: None
        '''
        assert type(worker_id) is int, "incorrect type of arg worker_id: should be int, is {}".format(type(worker_id))
        assert type(structure_type) is UnitType, "incorrect type of arg structure_type: should be UnitType, is {}".format(type(structure_type))
        assert type(direction) is Direction, "incorrect type of arg direction: should be Direction, is {}".format(type(direction))

        result = _lib.bc_GameController_blueprint(self._ptr, worker_id, structure_type, direction)
        _check_errors()
        return result

    def can_build(self, worker_id, blueprint_id):
        # type: (int, int) -> bool
        '''Whether the worker can build a blueprint with the given ID. The worker and the blueprint must be adjacent to each other. The worker cannot already have performed an action this round.
        :type self: GameController
        :type worker_id: int
        :type blueprint_id: int
        :rtype: bool
        '''
        assert type(worker_id) is int, "incorrect type of arg worker_id: should be int, is {}".format(type(worker_id))
        assert type(blueprint_id) is int, "incorrect type of arg blueprint_id: should be int, is {}".format(type(blueprint_id))

        result = _lib.bc_GameController_can_build(self._ptr, worker_id, blueprint_id)
        _check_errors()
        result = bool(result)
        return result

    def build(self, worker_id, blueprint_id):
        # type: (int, int) -> None
        '''Builds a given blueprint, increasing its health by the worker's build amount. If raised to maximum health, the blueprint becomes a completed structure.

        * NoSuchUnit - either unit does not exist (within the vision range).
        * TeamNotAllowed - either unit is not on the current player's team.
        * UnitNotOnMap - the worker is not on the map.
        * InappropriateUnitType - the unit is not a worker, or the blueprint is not a structure.
        * Overheated - the worker has already performed an action this turn.
        * OutOfRange - the worker is not adjacent to the blueprint.
        * StructureAlreadyBuilt - the blueprint has already been completed.
        :type self: GameController
        :type worker_id: int
        :type blueprint_id: int
        :rtype: None
        '''
        assert type(worker_id) is int, "incorrect type of arg worker_id: should be int, is {}".format(type(worker_id))
        assert type(blueprint_id) is int, "incorrect type of arg blueprint_id: should be int, is {}".format(type(blueprint_id))

        result = _lib.bc_GameController_build(self._ptr, worker_id, blueprint_id)
        _check_errors()
        return result

    def can_repair(self, worker_id, structure_id):
        # type: (int, int) -> bool
        '''Whether the given worker can repair the given strucutre. Tests that the worker is able to execute a worker action, that the structure is built, and that the structure is within range.
        :type self: GameController
        :type worker_id: int
        :type structure_id: int
        :rtype: bool
        '''
        assert type(worker_id) is int, "incorrect type of arg worker_id: should be int, is {}".format(type(worker_id))
        assert type(structure_id) is int, "incorrect type of arg structure_id: should be int, is {}".format(type(structure_id))

        result = _lib.bc_GameController_can_repair(self._ptr, worker_id, structure_id)
        _check_errors()
        result = bool(result)
        return result

    def repair(self, worker_id, structure_id):
        # type: (int, int) -> None
        '''Commands the worker to repair a structure, repleneshing health to it. This can only be done to structures which have been fully built.

        * NoSuchUnit - either unit does not exist (within the vision range).
        * TeamNotAllowed - either unit is not on the current player's team.
        * UnitNotOnMap - the worker is not on the map.
        * InappropriateUnitType - the unit is not a worker, or the target is not a structure.
        * Overheated - the worker has already performed an action this turn.
        * OutOfRange - the worker is not adjacent to the structure.
        * StructureNotYetBuilt - the structure has not been completed.
        :type self: GameController
        :type worker_id: int
        :type structure_id: int
        :rtype: None
        '''
        assert type(worker_id) is int, "incorrect type of arg worker_id: should be int, is {}".format(type(worker_id))
        assert type(structure_id) is int, "incorrect type of arg structure_id: should be int, is {}".format(type(structure_id))

        result = _lib.bc_GameController_repair(self._ptr, worker_id, structure_id)
        _check_errors()
        return result

    def can_replicate(self, worker_id, direction):
        # type: (int, Direction) -> bool
        '''Whether the worker is ready to replicate. Tests that the worker's ability heat is sufficiently low, that the team has sufficient karbonite in its resource pool, and that the square in the given direction is empty.
        :type self: GameController
        :type worker_id: int
        :type direction: Direction
        :rtype: bool
        '''
        assert type(worker_id) is int, "incorrect type of arg worker_id: should be int, is {}".format(type(worker_id))
        assert type(direction) is Direction, "incorrect type of arg direction: should be Direction, is {}".format(type(direction))

        result = _lib.bc_GameController_can_replicate(self._ptr, worker_id, direction)
        _check_errors()
        result = bool(result)
        return result

    def replicate(self, worker_id, direction):
        # type: (int, Direction) -> None
        '''Replicates a worker in the given direction. Subtracts the cost of the worker from the team's resource pool.

        * NoSuchUnit - the worker does not exist (within the vision range).
        * TeamNotAllowed - the worker is not on the current player's team.
        * InappropriateUnitType - the unit is not a worker.
        * Overheated - the worker is not ready to replicate again.
        * InsufficientKarbonite - your team does not have enough Karbonite for the worker to replicate.
        * UnitNotOnMap - the worker is not on the map.
        * LocationOffMap - the location in the target direction is off the map.
        * LocationNotVisible - the location is outside the vision range.
        * LocationNotEmpty - the location in the target direction is already occupied.
        :type self: GameController
        :type worker_id: int
        :type direction: Direction
        :rtype: None
        '''
        assert type(worker_id) is int, "incorrect type of arg worker_id: should be int, is {}".format(type(worker_id))
        assert type(direction) is Direction, "incorrect type of arg direction: should be Direction, is {}".format(type(direction))

        result = _lib.bc_GameController_replicate(self._ptr, worker_id, direction)
        _check_errors()
        return result

    def can_javelin(self, knight_id, target_unit_id):
        # type: (int, int) -> bool
        '''Whether the knight can javelin the given robot, without taking into account the knight's ability heat. Takes into account only the knight's ability range, and the location of the robot.
        :type self: GameController
        :type knight_id: int
        :type target_unit_id: int
        :rtype: bool
        '''
        assert type(knight_id) is int, "incorrect type of arg knight_id: should be int, is {}".format(type(knight_id))
        assert type(target_unit_id) is int, "incorrect type of arg target_unit_id: should be int, is {}".format(type(target_unit_id))

        result = _lib.bc_GameController_can_javelin(self._ptr, knight_id, target_unit_id)
        _check_errors()
        result = bool(result)
        return result

    def is_javelin_ready(self, knight_id):
        # type: (int) -> bool
        '''Whether the knight is ready to javelin. Tests whether the knight's ability heat is sufficiently low.
        :type self: GameController
        :type knight_id: int
        :rtype: bool
        '''
        assert type(knight_id) is int, "incorrect type of arg knight_id: should be int, is {}".format(type(knight_id))

        result = _lib.bc_GameController_is_javelin_ready(self._ptr, knight_id)
        _check_errors()
        result = bool(result)
        return result

    def javelin(self, knight_id, target_unit_id):
        # type: (int, int) -> None
        '''Javelins the robot, dealing the knight's standard damage.

        * NoSuchUnit - either unit does not exist (inside the vision range).
        * TeamNotAllowed - the knight is not on the current player's team.
        * UnitNotOnMap - the knight is not on the map.
        * InappropriateUnitType - the unit is not a knight.
        * ResearchNotUnlocked - you do not have the needed research to use javelin.
        * OutOfRange - the target does not lie within ability range of the knight.
        * Overheated - the knight is not ready to use javelin again.
        :type self: GameController
        :type knight_id: int
        :type target_unit_id: int
        :rtype: None
        '''
        assert type(knight_id) is int, "incorrect type of arg knight_id: should be int, is {}".format(type(knight_id))
        assert type(target_unit_id) is int, "incorrect type of arg target_unit_id: should be int, is {}".format(type(target_unit_id))

        result = _lib.bc_GameController_javelin(self._ptr, knight_id, target_unit_id)
        _check_errors()
        return result

    def can_begin_snipe(self, ranger_id, location):
        # type: (int, MapLocation) -> bool
        '''Whether the ranger can begin to snipe the given location, without taking into account the ranger's ability heat. Takes into account only the target location and the unit's type and unlocked abilities.
        :type self: GameController
        :type ranger_id: int
        :type location: MapLocation
        :rtype: bool
        '''
        assert type(ranger_id) is int, "incorrect type of arg ranger_id: should be int, is {}".format(type(ranger_id))
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))

        result = _lib.bc_GameController_can_begin_snipe(self._ptr, ranger_id, location._ptr)
        _check_errors()
        result = bool(result)
        return result

    def is_begin_snipe_ready(self, ranger_id):
        # type: (int) -> bool
        '''Whether the ranger is ready to begin snipe. Tests whether the ranger's ability heat is sufficiently low.
        :type self: GameController
        :type ranger_id: int
        :rtype: bool
        '''
        assert type(ranger_id) is int, "incorrect type of arg ranger_id: should be int, is {}".format(type(ranger_id))

        result = _lib.bc_GameController_is_begin_snipe_ready(self._ptr, ranger_id)
        _check_errors()
        result = bool(result)
        return result

    def begin_snipe(self, ranger_id, location):
        # type: (int, MapLocation) -> None
        '''Begins the countdown to snipe a given location. Maximizes the units attack and movement heats until the ranger has sniped. The ranger may begin the countdown at any time, including resetting the countdown to snipe a different location.

        * NoSuchUnit - either unit does not exist (inside the vision range).
        * TeamNotAllowed - the ranger is not on the current player's team.
        * UnitNotOnMap - the ranger is not on the map.
        * InappropriateUnitType - the unit is not a ranger.
        * ResearchNotUnlocked - you do not have the needed research to use snipe.
        * Overheated - the ranger is not ready to use snipe again.
        :type self: GameController
        :type ranger_id: int
        :type location: MapLocation
        :rtype: None
        '''
        assert type(ranger_id) is int, "incorrect type of arg ranger_id: should be int, is {}".format(type(ranger_id))
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))

        result = _lib.bc_GameController_begin_snipe(self._ptr, ranger_id, location._ptr)
        _check_errors()
        return result

    def can_blink(self, mage_id, location):
        # type: (int, MapLocation) -> bool
        '''Whether the mage can blink to the given location, without taking into account the mage's ability heat. Takes into account only the mage's ability range, the map terrain, positions of other units, and the edge of the game map.
        :type self: GameController
        :type mage_id: int
        :type location: MapLocation
        :rtype: bool
        '''
        assert type(mage_id) is int, "incorrect type of arg mage_id: should be int, is {}".format(type(mage_id))
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))

        result = _lib.bc_GameController_can_blink(self._ptr, mage_id, location._ptr)
        _check_errors()
        result = bool(result)
        return result

    def is_blink_ready(self, mage_id):
        # type: (int) -> bool
        '''Whether the mage is ready to blink. Tests whether the mage's ability heat is sufficiently low.
        :type self: GameController
        :type mage_id: int
        :rtype: bool
        '''
        assert type(mage_id) is int, "incorrect type of arg mage_id: should be int, is {}".format(type(mage_id))

        result = _lib.bc_GameController_is_blink_ready(self._ptr, mage_id)
        _check_errors()
        result = bool(result)
        return result

    def blink(self, mage_id, location):
        # type: (int, MapLocation) -> None
        '''Blinks the mage to the given location.

        * NoSuchUnit - the mage does not exist (inside the vision range).
        * TeamNotAllowed - the mage is not on the current player's team.
        * UnitNotOnMap - the mage is not on the map.
        * InappropriateUnitType - the unit is not a mage.
        * ResearchNotUnlocked - you do not have the needed research to use blink.
        * OutOfRange - the target does not lie within ability range of the mage.
        * LocationOffMap - the target location is not on this planet's map.
        * LocationNotVisible - the target location is outside the vision range.
        * LocationNotEmpty - the target location is already occupied.
        * Overheated - the mage is not ready to use blink again.
        :type self: GameController
        :type mage_id: int
        :type location: MapLocation
        :rtype: None
        '''
        assert type(mage_id) is int, "incorrect type of arg mage_id: should be int, is {}".format(type(mage_id))
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))

        result = _lib.bc_GameController_blink(self._ptr, mage_id, location._ptr)
        _check_errors()
        return result

    def can_heal(self, healer_id, target_robot_id):
        # type: (int, int) -> bool
        '''Whether the healer can heal the given robot, without taking into account the healer's attack heat. Takes into account only the healer's attack range, and the location of the robot.
        :type self: GameController
        :type healer_id: int
        :type target_robot_id: int
        :rtype: bool
        '''
        assert type(healer_id) is int, "incorrect type of arg healer_id: should be int, is {}".format(type(healer_id))
        assert type(target_robot_id) is int, "incorrect type of arg target_robot_id: should be int, is {}".format(type(target_robot_id))

        result = _lib.bc_GameController_can_heal(self._ptr, healer_id, target_robot_id)
        _check_errors()
        result = bool(result)
        return result

    def is_heal_ready(self, healer_id):
        # type: (int) -> bool
        '''Whether the healer is ready to heal. Tests whether the healer's attack heat is sufficiently low.
        :type self: GameController
        :type healer_id: int
        :rtype: bool
        '''
        assert type(healer_id) is int, "incorrect type of arg healer_id: should be int, is {}".format(type(healer_id))

        result = _lib.bc_GameController_is_heal_ready(self._ptr, healer_id)
        _check_errors()
        result = bool(result)
        return result

    def heal(self, healer_id, target_robot_id):
        # type: (int, int) -> None
        '''Commands the healer to heal the target robot.

        * NoSuchUnit - either unit does not exist (inside the vision range).
        * InappropriateUnitType - the unit is not a healer, or the target is not a robot.
        * TeamNotAllowed - either robot is not on the current player's team.
        * UnitNotOnMap - the healer is not on the map.
        * OutOfRange - the target does not lie within "attack" range of the healer.
        * Overheated - the healer is not ready to heal again.
        :type self: GameController
        :type healer_id: int
        :type target_robot_id: int
        :rtype: None
        '''
        assert type(healer_id) is int, "incorrect type of arg healer_id: should be int, is {}".format(type(healer_id))
        assert type(target_robot_id) is int, "incorrect type of arg target_robot_id: should be int, is {}".format(type(target_robot_id))

        result = _lib.bc_GameController_heal(self._ptr, healer_id, target_robot_id)
        _check_errors()
        return result

    def can_overcharge(self, healer_id, target_robot_id):
        # type: (int, int) -> bool
        '''Whether the healer can overcharge the given robot, without taking into account the healer's ability heat. Takes into account only the healer's ability range, and the location of the robot.
        :type self: GameController
        :type healer_id: int
        :type target_robot_id: int
        :rtype: bool
        '''
        assert type(healer_id) is int, "incorrect type of arg healer_id: should be int, is {}".format(type(healer_id))
        assert type(target_robot_id) is int, "incorrect type of arg target_robot_id: should be int, is {}".format(type(target_robot_id))

        result = _lib.bc_GameController_can_overcharge(self._ptr, healer_id, target_robot_id)
        _check_errors()
        result = bool(result)
        return result

    def is_overcharge_ready(self, healer_id):
        # type: (int) -> bool
        '''Whether the healer is ready to overcharge. Tests whether the healer's ability heat is sufficiently low.
        :type self: GameController
        :type healer_id: int
        :rtype: bool
        '''
        assert type(healer_id) is int, "incorrect type of arg healer_id: should be int, is {}".format(type(healer_id))

        result = _lib.bc_GameController_is_overcharge_ready(self._ptr, healer_id)
        _check_errors()
        result = bool(result)
        return result

    def overcharge(self, healer_id, target_robot_id):
        # type: (int, int) -> None
        '''Overcharges the robot, resetting the robot's cooldowns. The robot must be on the same team as you.

        * NoSuchUnit - either unit does not exist (inside the vision range).
        * TeamNotAllowed - either robot is not on the current player's team.
        * UnitNotOnMap - the healer is not on the map.
        * InappropriateUnitType - the unit is not a healer, or the target is not a robot.
        * ResearchNotUnlocked - you do not have the needed research to use overcharge.
        * OutOfRange - the target does not lie within ability range of the healer.
        * Overheated - the healer is not ready to use overcharge again.
        :type self: GameController
        :type healer_id: int
        :type target_robot_id: int
        :rtype: None
        '''
        assert type(healer_id) is int, "incorrect type of arg healer_id: should be int, is {}".format(type(healer_id))
        assert type(target_robot_id) is int, "incorrect type of arg target_robot_id: should be int, is {}".format(type(target_robot_id))

        result = _lib.bc_GameController_overcharge(self._ptr, healer_id, target_robot_id)
        _check_errors()
        return result

    def can_load(self, structure_id, robot_id):
        # type: (int, int) -> bool
        '''Whether the robot can be loaded into the given structure's garrison. The robot must be ready to move and must be adjacent to the structure. The structure and the robot must be on the same team, and the structure must have space.
        :type self: GameController
        :type structure_id: int
        :type robot_id: int
        :rtype: bool
        '''
        assert type(structure_id) is int, "incorrect type of arg structure_id: should be int, is {}".format(type(structure_id))
        assert type(robot_id) is int, "incorrect type of arg robot_id: should be int, is {}".format(type(robot_id))

        result = _lib.bc_GameController_can_load(self._ptr, structure_id, robot_id)
        _check_errors()
        result = bool(result)
        return result

    def load(self, structure_id, robot_id):
        # type: (int, int) -> None
        '''Loads the robot into the garrison of the structure.

        * NoSuchUnit - either unit does not exist (inside the vision range).
        * TeamNotAllowed - either unit is not on the current player's team.
        * UnitNotOnMap - either unit is not on the map.
        * Overheated - the robot is not ready to move again.
        * InappropriateUnitType - the first unit is not a structure, or the second unit is not a robot.
        * StructureNotYetBuilt - the structure has not yet been completed.
        * GarrisonFull - the structure's garrison is already full.
        * OutOfRange - the robot is not adjacent to the structure.
        :type self: GameController
        :type structure_id: int
        :type robot_id: int
        :rtype: None
        '''
        assert type(structure_id) is int, "incorrect type of arg structure_id: should be int, is {}".format(type(structure_id))
        assert type(robot_id) is int, "incorrect type of arg robot_id: should be int, is {}".format(type(robot_id))

        result = _lib.bc_GameController_load(self._ptr, structure_id, robot_id)
        _check_errors()
        return result

    def can_unload(self, structure_id, direction):
        # type: (int, Direction) -> bool
        '''Tests whether the given structure is able to unload a unit in the given direction. There must be space in that direction, and the unit must be ready to move.
        :type self: GameController
        :type structure_id: int
        :type direction: Direction
        :rtype: bool
        '''
        assert type(structure_id) is int, "incorrect type of arg structure_id: should be int, is {}".format(type(structure_id))
        assert type(direction) is Direction, "incorrect type of arg direction: should be Direction, is {}".format(type(direction))

        result = _lib.bc_GameController_can_unload(self._ptr, structure_id, direction)
        _check_errors()
        result = bool(result)
        return result

    def unload(self, structure_id, direction):
        # type: (int, Direction) -> None
        '''Unloads a robot from the garrison of the specified structure into an adjacent space. Robots are unloaded in the order they were loaded.

        * NoSuchUnit - the unit does not exist (inside the vision range).
        * TeamNotAllowed - either unit is not on the current player's team.
        * UnitNotOnMap - the structure is not on the map.
        * InappropriateUnitType - the unit is not a structure.
        * StructureNotYetBuilt - the structure has not yet been completed.
        * GarrisonEmpty - the structure's garrison is already empty.
        * LocationOffMap - the location in the target direction is off the map.
        * LocationNotEmpty - the location in the target direction is already occupied.
        * Overheated - the robot inside the structure is not ready to move again.
        :type self: GameController
        :type structure_id: int
        :type direction: Direction
        :rtype: None
        '''
        assert type(structure_id) is int, "incorrect type of arg structure_id: should be int, is {}".format(type(structure_id))
        assert type(direction) is Direction, "incorrect type of arg direction: should be Direction, is {}".format(type(direction))

        result = _lib.bc_GameController_unload(self._ptr, structure_id, direction)
        _check_errors()
        return result

    def can_produce_robot(self, factory_id, robot_type):
        # type: (int, UnitType) -> bool
        '''Whether the factory can produce a robot of the given type. The factory must not currently be producing a robot, and the team must have sufficient resources in its resource pool.
        :type self: GameController
        :type factory_id: int
        :type robot_type: UnitType
        :rtype: bool
        '''
        assert type(factory_id) is int, "incorrect type of arg factory_id: should be int, is {}".format(type(factory_id))
        assert type(robot_type) is UnitType, "incorrect type of arg robot_type: should be UnitType, is {}".format(type(robot_type))

        result = _lib.bc_GameController_can_produce_robot(self._ptr, factory_id, robot_type)
        _check_errors()
        result = bool(result)
        return result

    def produce_robot(self, factory_id, robot_type):
        # type: (int, UnitType) -> None
        '''Starts producing the robot of the given type.

        * NoSuchUnit - the factory does not exist (inside the vision range).
        * TeamNotAllowed - the factory is not on the current player's team.
        * InappropriateUnitType - the unit is not a factory, or the unit type is not a robot.
        * StructureNotYetBuilt - the factory has not yet been completed.
        * FactoryBusy - the factory is already producing a unit.
        * InsufficientKarbonite - your team does not have enough Karbonite to produce the given robot.
        :type self: GameController
        :type factory_id: int
        :type robot_type: UnitType
        :rtype: None
        '''
        assert type(factory_id) is int, "incorrect type of arg factory_id: should be int, is {}".format(type(factory_id))
        assert type(robot_type) is UnitType, "incorrect type of arg robot_type: should be UnitType, is {}".format(type(robot_type))

        result = _lib.bc_GameController_produce_robot(self._ptr, factory_id, robot_type)
        _check_errors()
        return result

    def rocket_landings(self):
        # type: () -> RocketLandingInfo
        '''The landing rounds and locations of rockets in space that belong to the current team.
        :type self: GameController
        :rtype: RocketLandingInfo
        '''

        result = _lib.bc_GameController_rocket_landings(self._ptr)
        _check_errors()
        _result = RocketLandingInfo.__new__(RocketLandingInfo)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def can_launch_rocket(self, rocket_id, destination):
        # type: (int, MapLocation) -> bool
        '''Whether the rocket can launch into space to the given destination. The rocket can launch if the it has never been used before. The destination is valid if it contains passable terrain on the other planet.
        :type self: GameController
        :type rocket_id: int
        :type destination: MapLocation
        :rtype: bool
        '''
        assert type(rocket_id) is int, "incorrect type of arg rocket_id: should be int, is {}".format(type(rocket_id))
        assert type(destination) is MapLocation, "incorrect type of arg destination: should be MapLocation, is {}".format(type(destination))

        result = _lib.bc_GameController_can_launch_rocket(self._ptr, rocket_id, destination._ptr)
        _check_errors()
        result = bool(result)
        return result

    def launch_rocket(self, rocket_id, location):
        # type: (int, MapLocation) -> None
        '''Launches the rocket into space, damaging the units adjacent to the takeoff location.

        * NoSuchUnit - the rocket does not exist (inside the vision range).
        * TeamNotAllowed - the rocket is not on the current player's team.
        * SamePlanet - the rocket cannot fly to a location on the same planet.
        * InappropriateUnitType - the unit is not a rocket.
        * StructureNotYetBuilt - the rocket has not yet been completed.
        * RocketUsed - the rocket has already been used.
        * LocationOffMap - the given location is off the map.
        * LocationNotEmpty - the given location contains impassable terrain.
        :type self: GameController
        :type rocket_id: int
        :type location: MapLocation
        :rtype: None
        '''
        assert type(rocket_id) is int, "incorrect type of arg rocket_id: should be int, is {}".format(type(rocket_id))
        assert type(location) is MapLocation, "incorrect type of arg location: should be MapLocation, is {}".format(type(location))

        result = _lib.bc_GameController_launch_rocket(self._ptr, rocket_id, location._ptr)
        _check_errors()
        return result

    @staticmethod
    def new_manager(map):
        # type: (GameMap) -> GameController
        '''
        :type map: GameMap
        :rtype: GameController
        '''
        assert type(map) is GameMap, "incorrect type of arg map: should be GameMap, is {}".format(type(map))

        result = _lib.bc_GameController_new_manager(map._ptr)
        _check_errors()
        _result = GameController.__new__(GameController)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def start_game(self, player):
        # type: (Player) -> StartGameMessage
        '''
        :type self: GameController
        :type player: Player
        :rtype: StartGameMessage
        '''
        assert type(player) is Player, "incorrect type of arg player: should be Player, is {}".format(type(player))

        result = _lib.bc_GameController_start_game(self._ptr, player._ptr)
        _check_errors()
        _result = StartGameMessage.__new__(StartGameMessage)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def apply_turn(self, turn):
        # type: (TurnMessage) -> TurnApplication
        '''
        :type self: GameController
        :type turn: TurnMessage
        :rtype: TurnApplication
        '''
        assert type(turn) is TurnMessage, "incorrect type of arg turn: should be TurnMessage, is {}".format(type(turn))

        result = _lib.bc_GameController_apply_turn(self._ptr, turn._ptr)
        _check_errors()
        _result = TurnApplication.__new__(TurnApplication)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def initial_start_turn_message(self):
        # type: () -> InitialTurnApplication
        '''
        :type self: GameController
        :rtype: InitialTurnApplication
        '''

        result = _lib.bc_GameController_initial_start_turn_message(self._ptr)
        _check_errors()
        _result = InitialTurnApplication.__new__(InitialTurnApplication)
        if result != _ffi.NULL:
            _result._ptr = result
        result = _result
        return result

    def is_over(self):
        # type: () -> bool
        '''
        :type self: GameController
        :rtype: bool
        '''

        result = _lib.bc_GameController_is_over(self._ptr)
        _check_errors()
        result = bool(result)
        return result

    def winning_team(self):
        # type: () -> Team
        '''
        :type self: GameController
        :rtype: Team
        '''

        result = _lib.bc_GameController_winning_team(self._ptr)
        _check_errors()
        result = Team(result)
        return result


