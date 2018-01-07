from frankenswig import *

p = Program(module='bc', crate='battlecode_engine', docs='''Battlecode engine.

Woo.''')

Planet = p.c_enum('location::Planet', docs='The planets in the Battlecode world.')
Planet.variant('Earth', 0)
Planet.variant('Mars', 1)

Direction = p.c_enum('location::Direction', docs='''Represents a direction from one MapLocation to another.

Directions for each of the cardinals (north, south, east, west), and each
of the diagonals (northwest, southwest, northeast, southeast). There is
also a "center" direction, representing no direction.

Coordinates increase in the north and east directions.
''')
Direction.variant('North', 0)
Direction.variant('Northeast', 1)
Direction.variant('East', 2)
Direction.variant('Southeast', 3)
Direction.variant('South', 4)
Direction.variant('Southwest', 5)
Direction.variant('West', 6)
Direction.variant('Northwest', 7)
Direction.variant('Center', 8)
Direction.method(Direction.type, 'opposite', [])
Direction.method(Direction.type, 'rotate_left', [])
Direction.method(Direction.type, 'rotate_right', [])

MapLocation = p.struct('location::MapLocation',
    'Represents two-dimensional coordinates in the Battlecode world. Naive of which planet it is on.')
MapLocation.constructor('new', [Var(Planet.type, 'planet'), Var(i32.type, 'x'), Var(i32.type, 'y')],
        docs='Create a new MapLocation.')
MapLocation.member(Planet.type, 'planet', docs='The planet lol.')
MapLocation.member(i32.type, 'x', docs='The x coordinate of the map location.')
MapLocation.member(i32.type, 'y', docs='The y coordinate of the map location.')

MapLocation.method(MapLocation.type, 'add', [Var(Direction.type, 'direction')])

UnitID = p.typedef('unit::UnitID', u16.type)
Rounds = p.typedef('world::Rounds', u32.type)

Team = p.c_enum('world::Team')
Team.variant('Red', 0)
Team.variant('Blue', 1)

Player = p.struct('world::Player')
Player.constructor('new', [Var(Team.type, 'team'), Var(Planet.type, 'planet')])
Player.member(Team.type, 'team')
Player.member(Planet.type, 'planet')

GameMap = p.struct('map::GameMap')
GameMap.method(GameMap.type, 'test_map', [], static=True)
GameMap.serializeable()

Delta = p.struct('schema::Delta')
Delta.serializeable()

StartGameMessage = p.struct('schema::StartGameMessage')
StartGameMessage.serializeable()

TurnMessage = p.struct('schema::TurnMessage')
TurnMessage.serializeable()

StartTurnMessage = p.struct('schema::StartTurnMessage')
StartTurnMessage.member(Rounds.type, 'round')
StartTurnMessage.serializeable()

ViewerMessage = p.struct('schema::ViewerMessage')
ViewerMessage.serializeable()

ErrorMessage = p.struct('schema::ErrorMessage')
ErrorMessage.serializeable()

GameController = p.struct('controller::GameController')
GameController.constructor('new_player', [Var(StartGameMessage.type, 'game')])
GameController.method(void.type.result(), 'start_turn', [Var(StartTurnMessage.type, 'turn')])
GameController.method(TurnMessage.type.result(), 'end_turn', [])
GameController.method(Rounds.type, 'round', [])
GameController.method(Planet.type, 'planet', [])
GameController.method(Team.type, 'team', [])
GameController.method(u32.type, 'karbonite', [])
# TODO: more methods
GameController.method(GameController.type, 'new_manager', [Var(GameMap.type, 'map')], static=True)
GameController.method(StartGameMessage.type, 'start_game', [Var(Player.type, 'player')])

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