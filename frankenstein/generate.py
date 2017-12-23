from frankenswig import *

p = Program(name='bc', crate='battlecode_engine', docs='''Battlecode engine.

Woo.''')

Planet = p.c_enum('location::Planet', docs='The planets in the Battlecode world.')\
    .variant('Earth', 0)\
    .variant('Mars', 1)

Direction = p.c_enum('location::Direction', docs='''Represents a direction from one MapLocation to another.

Directions for each of the cardinals (north, south, east, west), and each
of the diagonals (northwest, southwest, northeast, southeast). There is
also a "center" direction, representing no direction.

Coordinates increase in the north and east directions.
''')\
    .variant('North', 0)\
    .variant('Northeast', 1)\
    .variant('East', 2)\
    .variant('Southeast', 3)\
    .variant('South', 4)\
    .variant('Southwest', 5)\
    .variant('West', 6)\
    .variant('Northwest', 7)\
    .variant('Center', 8)

Direction.method(Direction.type, 'opposite', [])
Direction.method(Direction.type, 'rotate_left', [])
Direction.method(Direction.type, 'rotate_right', [])

MapLocation = p.struct('location::MapLocation',
    'Represents two-dimensional coordinates in the Battlecode world. Naive of which planet it is on.')\
    .constructor('new', [Var(Planet.type, 'planet'), Var(i32.type, 'x'), Var(i32.type, 'y')],
        docs='Create a new MapLocation.')\
    .member(Planet.type, 'planet', docs='The planet lol.')\
    .member(i32.type, 'x', docs='The x coordinate of the map location.')\
    .member(i32.type, 'y', docs='The y coordinate of the map location.')

MapLocation.method(MapLocation.type, 'add', [Var(Direction.type, 'direction')])

UnitID = p.typedef('unit::UnitID', u16.type)

Team = p.c_enum('world::Team')\
    .variant('Red', 0)\
    .variant('Blue', 1)

UnitInfo = p.struct('unit::UnitInfo')\
    .constructor('default', [])

Unit = p.struct('unit::Unit')\
    .constructor('new', [
        Var(UnitID.type, 'id'),
        Var(Team.type, 'team'),
        Var(u32.type, 'max_health'),
        Var(MapLocation.type, 'location'),
        Var(u32.type, 'health'),
        Var(UnitInfo.type, 'unit_info')
    ])\
    .method(boolean.type, 'is_move_ready', [])

print('Generating...')
with open("src/bindings.rs", "w+") as f:
    f.write(p.to_rust())

with open("c/include/bc.h", "w+") as f:
    f.write(p.to_c())

with open("c/include/bc.i", "w+") as f:
    f.write(p.to_swig())

with open("python/battlecode/bc.py", "w+") as f:
    f.write(p.to_python())
print('Done.')