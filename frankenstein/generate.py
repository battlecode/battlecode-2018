from frankenswig import *

p = Program(name='bc', crate='battlecode_engine')

MapLocation = p.struct('location::MapLocation',
    'Represents two-dimensional coordinates in the Battlecode world. Naive of which planet it is on.')\
    .constructor('new', [Var(i32.type, 'x'), Var(i32.type, 'y')])\
    .member(i32.type, 'x')\
    .member(i32.type, 'y')

EntityId = p.typedef('entity::EntityId', u16.type)

#EntityInfo = p.struct('entity::EntityInfo',
#    'Generic info for a single entity, and the associated body.')\
#    .member(EntityId.type, 'id')\
#    .member(u32.type, 'max_health')\
#    .member(MapLocation.type, 'location')\
#    .member(u32.type, 'health')\
#    .constructor('new', [])\

p.elements.append(CEnum('bc_TestEnum', (('x',1),('y',2),('z',3))))

print(MapLocation.type.to_python())

print('Generating...')
with open("src/bindings.rs", "w+") as f:
    f.write(p.to_rust())

with open("bc.h", "w+") as f:
    f.write(p.to_c())

with open("bc.i", "w+") as f:
    f.write(p.to_swig())

print('Done.')