from frankenswig import *

p = Program(name='bc', crate='battlecode_engine', docs='''Battlecode engine.

Woo.''')

MapLocation = p.struct('location::MapLocation',
    'Represents two-dimensional coordinates in the Battlecode world. Naive of which planet it is on.')\
    .constructor('new', [Var(i32.type, 'x'), Var(i32.type, 'y')], docs='Create a new MapLocation.')\
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

cenum = CEnum('bc', 'TestEnum')
cenum.variant('BANANS', 1)
cenum.variant('DOGGOS', 2)

p.elements.append(cenum)

print('Generating...')
with open("src/bindings.rs", "w+") as f:
    f.write(p.to_rust())

with open("bc.h", "w+") as f:
    f.write(p.to_c())

with open("bc.i", "w+") as f:
    f.write(p.to_swig())

with open("python/battlecode/bc.py", "w+") as f:
    f.write(p.to_python())
print('Done.')