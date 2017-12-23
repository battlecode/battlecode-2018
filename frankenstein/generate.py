from frankenswig import *

p = Program(name='bc', crate='battlecode_engine', docs='''Battlecode engine.

Woo.''')

Planet = p.c_enum('location::Planet', docs='The planets in the Battlecode world.')\
    .variant('Earth', 0)\
    .variant('Mars', 1)

MapLocation = p.struct('location::MapLocation',
    'Represents two-dimensional coordinates in the Battlecode world. Naive of which planet it is on.')\
    .constructor('new', [Var(Planet.type, 'planet'), Var(i32.type, 'x'), Var(i32.type, 'y')],
        docs='Create a new MapLocation.')\
    .member(Planet.type, 'planet', docs='The planet lol.')\
    .member(i32.type, 'x', docs='The x coordinate of the map location.')\
    .member(i32.type, 'y', docs='The y coordinate of the map location.')

EntityId = p.typedef('entity::EntityId', u16.type)

#EntityInfo = p.struct('entity::EntityInfo',
#    'Generic info for a single entity, and the associated body.')\
#    .member(EntityId.type, 'id')\
#    .member(u32.type, 'max_health')\
#    .member(MapLocation.type, 'location')\
#    .member(u32.type, 'health')\
#    .constructor('new', [])\

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