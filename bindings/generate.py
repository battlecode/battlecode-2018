from frankenswig import *

p = Program(module='bc', crate='battlecode_engine', docs='''Battlecode engine.

Woo.''')

Planet = p.c_enum('location::Planet', docs='The planets in the Battlecode world.')
Planet.variant('Earth', 0)
Planet.variant('Mars', 1)
Planet.debug()
Planet.eq()
Planet.serialize()

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
Direction.serialize()

MapLocation = p.struct('location::MapLocation',
    'Represents two-dimensional coordinates in the Battlecode world. Naive of which planet it is on.')
MapLocation.constructor('new', [Var(Planet.type, 'planet'), Var(i32.type, 'x'), Var(i32.type, 'y')],
        docs='Create a new MapLocation.')
MapLocation.member(Planet.type, 'planet', docs='The planet lol.')
MapLocation.member(i32.type, 'x', docs='The x coordinate of the map location.')
MapLocation.member(i32.type, 'y', docs='The y coordinate of the map location.')
MapLocation.debug()
MapLocation.clone()
MapLocation.eq()
MapLocation.serialize()

Location = p.struct('location::Location')
Location.debug()
Location.clone()
Location.eq()
Location.serialize()

MapLocation.method(MapLocation.type, 'add', [Var(Direction.type, 'direction')])

UnitID = p.typedef('unit::UnitID', u16.type)
Rounds = p.typedef('world::Rounds', u32.type)

Team = p.c_enum('world::Team')
Team.variant('Red', 0)
Team.variant('Blue', 1)
Team.serialize()

Player = p.struct('world::Player')
Player.constructor('new', [Var(Team.type, 'team'), Var(Planet.type, 'planet')])
Player.member(Team.type, 'team')
Player.member(Planet.type, 'planet')
Player.debug()
Player.clone()
Player.eq()
Player.serialize()

Level = p.typedef('research::Level', usize.type)

Percent = p.typedef('unit::Percent', u32.type)
UnitID = p.typedef('unit::UnitID', u16.type)

UnitType = p.c_enum("unit::UnitType",
    docs="The different unit types, which include factories, rockets, and the robots.")
#UnitType.serialize()
UnitType.variant('Worker', 0, docs="Workers are the foundation of the civilization.")
UnitType.variant('Knight', 1, docs="Knights are a melee unit that is strong in numbers.")
UnitType.variant('Ranger', 2, docs="Rangers are a ranged unit with good all-around combat.")
UnitType.variant('Mage', 3, docs="Mages are a fragile but specialized ranged unit for large areas.")
UnitType.variant('Healer', 4, docs="Healers are a suport unit that can heal other units.")
UnitType.variant('Factory', 5, docs="Factories are the hub for producing combative robots.")
UnitType.variant('Rocket', 6, docs="Rockets are the only unit that can move between planets.")
UnitType.method(boolean.type, "is_robot", [], docs="Whether the unit type is a robot.")
UnitType.method(boolean.type, "is_structure", [], docs="Whether the unit type is a structure.")
UnitType.method(u32.type.result(), "factory_cost", [], docs="""The cost of the unit in a factory.
Errors if the unit cannot be produced in a factory.""")
UnitType.method(u32.type.result(), "blueprint_cost", [], docs="""The cost to blueprint the unit.
Errors if the unit cannot be blueprinted.""")
UnitType.method(u32.type.result(), "replicate_cost", [], docs="""The cost to replicate the unit.
Errors if the unit cannot be replicated.""")
UnitType.method(u32.type, "value", [], docs="The value of a unit, as relevant to tiebreakers.")

UnitInfo = p.struct('unit::UnitInfo',
    docs='''The public version of the unit. Contains all the unit's stats but none of
the action. The other team can see everything in the unit info.''')
UnitInfo.clone()
UnitInfo.serialize()
UnitInfo.eq()
UnitInfo.debug()
UnitInfo.member(UnitID.type, 'id', docs="The unique ID of the unit.")
UnitInfo.member(Team.type, 'team', docs="The team the unit is on.")
UnitInfo.member(UnitType.type, 'unit_type', docs="The type of the unit.")
UnitInfo.member(Location.type, 'location', docs="The current location of the unit.")
UnitInfo.member(u32.type, 'health', docs="The current health of the unit.")
#UnitInfo.method(UnitType.type.vec(), "all", (), docs="List all the unit types.", staticmethod=True)

Unit = p.struct("unit::Unit", docs="A single unit in the game and all its associated properties.")
Unit.debug()
Unit.clone()
Unit.serialize()
Unit.eq()
Unit.method(UnitID.type, "id", [], docs="The unique ID of a unit.", getter=True)
Unit.method(Team.type, "team", [], docs="The team the unit belongs to.", getter=True)
Unit.method(Level.type, "research_level", [], docs="The current research level.", getter=True)
Unit.method(UnitType.type, "unit_type", [], docs="The unit type.", getter=True)
Unit.method(Location.type, "location", [], docs="The location of the unit.", getter=True)
Unit.method(u32.type, "health", [], docs="The current health.", getter=True)
Unit.method(u32.type, "max_health", [], docs="The maximum health.", getter=True)
Unit.method(u32.type, "vision_range", [], docs="The unit vision range.", getter=True)
Unit.method(i32.type.result(), "damage", [], docs="""The damage inflicted by the robot during a normal attack.
Errors if the unit is not a robot.""")
Unit.method(u32.type.result(), "attack_range", [], docs="""The attack range.
Errors if the unit is not a robot.""", getter=True)
Unit.method(u32.type.result(), "movement_heat", [], docs="""The movement heat.
Errors if the unit is not a robot.""", getter=True)
Unit.method(u32.type.result(), "attack_heat", [], docs="""The attack heat.
Errors if the unit is not a robot.""", getter=True)
Unit.method(u32.type.result(), "movement_cooldown", [], docs="""The movement cooldown.
Errors if the unit is not a robot.""", getter=True)
Unit.method(u32.type.result(), "attack_cooldown", [], docs="""The attack cooldown.
Errors if the unit is not a robot.""", getter=True)
Unit.method(boolean.type.result(), "is_ability_unlocked", [], docs="""Whether the active ability is unlocked.
Errors if the unit is not a robot.""", getter=True)
Unit.method(u32.type.result(), "ability_heat", [], docs="""The active ability heat.
Errors if the unit is not a robot.""", getter=True)
Unit.method(u32.type.result(), "ability_cooldown", [], docs="""The active ability cooldown.
Errors if the unit is not a robot.""", getter=True)
Unit.method(u32.type.result(), "ability_range", [], docs="""The active ability range.
Errors if the unit is not a robot.""", getter=True)
Unit.method(boolean.type.result(), "worker_has_acted", [], docs="""Whether the worker has already acted (harveted, blueprinted, built, or
repaired) this round.
Errors if the unit is not a worker.""", getter=True)
Unit.method(u32.type.result(), "worker_build_health", [], docs="""The health restored when building or repairing a structure.
Errors if the unit is not a worker.""", getter=True)
Unit.method(u32.type.result(), "worker_harvest_amount", [], docs="""The maximum amount of karbonite harvested from a deposit in one turn.
Errors if the unit is not a worker.""", getter=True)
Unit.method(u32.type.result(), "knight_defense", [], docs="""The amount of damage resisted by a knight when attacked.
Errors if the unit is not a knight.""", getter=True)
Unit.method(u32.type.result(), "ranger_cannot_attack_range", [], docs="""The range within a ranger cannot attack.
Errors if the unit is not a ranger.""", getter=True)
Unit.method(u32.type.result(), "ranger_countdown", [], docs="""The countdown for ranger's snipe.
Errors if the unit is not a ranger.""", getter=True)
#Unit.method(Option<MapLocation>.type, "ranger_target_location", [], docs="""The target location for ranger's snipe.
#Errors if the unit is not a ranger.""")
Unit.method(boolean.type.result(), "ranger_is_sniping", [], docs="""Whether the ranger is sniping.
Errors if the unit is not a ranger.""", getter=True)
Unit.method(u32.type.result(), "healer_self_heal_amount", [], docs="""The amount of health passively restored to itself each round.
Errors if the unit is not a healer.""", getter=True)
Unit.method(boolean.type.result(), "structure_is_built", [], docs="""Whether this structure has been built.
Errors if the unit is not a structure.""", getter=True)
Unit.method(usize.type.result(), "structure_max_capacity", [], docs="""The max capacity of a structure.
Errors if the unit is not a structure.""", getter=True)
#Unit.method(Vec<UnitID>.type, "structure_garrison", [], docs="""Returns the units in the structure's garrison.
#Errors if the unit is not a structure.""")
#Unit.method(Option<UnitType>.type, "factory_unit_type", [], docs="""The unit type currently being produced by the factory, or None if the
#factory is not producing a unit.
#Errors if the unit is not a factory.""")
#Unit.method(Option<Rounds>.type, "factory_rounds_left", [], docs="""The number of rounds left to produce a robot in this factory. Returns
#None if no unit is currently being produced.
#Errors if the unit is not a factory.""")
Unit.method(boolean.type.result(), "rocket_is_used", [], docs="""Whether the rocket has already been used.
Errors if the unit is not a rocket.""", getter=True)
Unit.method(u32.type.result(), "rocket_travel_time_decrease", [], docs="""The number of rounds the rocket travel time is reduced by compared
to the travel time determined by the orbit of the planets.
Errors if the unit is not a rocket.""", getter=True)

GameMap = p.struct('map::GameMap')
GameMap.method(GameMap.type, 'test_map', [], static=True)
GameMap.clone()
GameMap.serialize()

Delta = p.struct('schema::Delta')
Delta.serialize()

StartGameMessage = p.struct('schema::StartGameMessage')
StartGameMessage.serialize()

TurnMessage = p.struct('schema::TurnMessage')
TurnMessage.serialize()

StartTurnMessage = p.struct('schema::StartTurnMessage')
StartTurnMessage.member(Rounds.type, 'round')
StartTurnMessage.serialize()

ViewerMessage = p.struct('schema::ViewerMessage')
ViewerMessage.serialize()

ErrorMessage = p.struct('schema::ErrorMessage')
ErrorMessage.member(p.string.type, "error")
ErrorMessage.serialize()
ErrorMessage.debug()

TurnApplication = p.struct("controller::TurnApplication")
TurnApplication.member(StartTurnMessage.type, 'start_turn')
TurnApplication.member(ViewerMessage.type, 'viewer')

GameController = p.struct('controller::GameController')
GameController.constructor('new_player', [Var(StartGameMessage.type, 'game')])
GameController.method(void.type.result(), 'start_turn', [Var(StartTurnMessage.type.ref(), 'turn')])
GameController.method(TurnMessage.type.result(), 'end_turn', [])
GameController.method(Rounds.type, 'round', [])
GameController.method(Planet.type, 'planet', [])
GameController.method(Team.type, 'team', [])
GameController.method(u32.type, 'karbonite', [])
# TODO: more methods
GameController.method(GameController.type, 'new_manager', [Var(GameMap.type, 'map')], static=True)
GameController.method(StartGameMessage.type, 'start_game', [Var(Player.type, 'player')])
GameController.method(TurnApplication.type.result(), 'apply_turn', [Var(TurnMessage.type.ref(), 'turn')])

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