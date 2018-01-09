from frankenswig import *

p = Program(module='bc', crate='battlecode_engine', docs='''Battlecode engine.

Woo.''')

Planet = p.c_enum('location::Planet', docs='The planets in the Battlecode world.')
Planet.variant('Earth', 0)
Planet.variant('Mars', 1)
Planet.method(Planet.type, 'other', [], docs='''The other planet.''', self_ref=True)
Planet.debug()
Planet.eq()
Planet.serialize()

Direction = p.c_enum('location::Direction', docs='''A direction from one MapLocation to another.

Directions for each of the cardinals (north, south, east, west), and each
of the diagonals (northwest, southwest, northeast, southeast). There is
also a "center" direction, representing no direction.

Coordinates increase in the north and east directions.''')
Direction.variant('North', 0)
Direction.variant('Northeast', 1)
Direction.variant('East', 2)
Direction.variant('Southeast', 3)
Direction.variant('South', 4)
Direction.variant('Southwest', 5)
Direction.variant('West', 6)
Direction.variant('Northwest', 7)
Direction.variant('Center', 8)
Direction.method(i32.type, 'dx', [], docs='''Returns the x displacement of this direction.''', self_ref=True)
Direction.method(i32.type, 'dy', [], docs='''Returns the y displacement of this direction.''', self_ref=True)
Direction.method(boolean.type, 'is_diagonal', [], docs='''Whether this direction is a diagonal one.''', self_ref=True)
Direction.method(Direction.type, 'opposite', [], docs='''Returns the direction opposite this one, or Center if it's Center.''', self_ref=True)
Direction.method(Direction.type, 'rotate_left', [], docs='''Returns the direction 45 degrees to the left (counter-clockwise) of
this one, or Center if it's Center.''', self_ref=True)
Direction.method(Direction.type, 'rotate_right', [], docs='''Returns the direction 45 degrees to the right (clockwise) of this one,
or Center if it's Center.''', self_ref=True)
Direction.serialize()

MapLocation = p.struct('location::MapLocation',
    'Two-dimensional coordinates in the Battlecode world.')
MapLocation.constructor('new', [Var(Planet.type, 'planet'), Var(i32.type, 'x'), Var(i32.type, 'y')],
    docs='''Returns a new MapLocation representing the location with the given
coordinates on a planet.''')
MapLocation.member(Planet.type, 'planet', docs='The planet of the map location.')
MapLocation.member(i32.type, 'x', docs='The x coordinate of the map location.')
MapLocation.member(i32.type, 'y', docs='The y coordinate of the map location.')
MapLocation.method(MapLocation.type, 'add', [Var(Direction.type, 'direction')], docs='''Returns the location one square from this one in the given direction.''')
MapLocation.method(MapLocation.type, 'subtract', [Var(Direction.type, 'direction')], docs='Returns the location one square from this one in the opposite direction.')
MapLocation.method(MapLocation.type, 'add_multiple', [Var(Direction.type, 'direction'), Var(i32.type, 'multiple')], docs='''Returns the location `multiple` squares from this one in the given
direction.''')
MapLocation.method(MapLocation.type, 'translate', [Var(i32.type, 'dx'), Var(i32.type, 'dy')], docs='''Returns the location translated from this location by `dx` in the x
direction and `dy` in the y direction.''')
MapLocation.method(u32.type, 'distance_squared_to', [Var(MapLocation.type, 'o')], docs='''Computes the square of the distance from this location to the specified
location. If on different planets, returns the maximum integer.''')
MapLocation.method(Direction.type.result(), 'direction_to', [Var(MapLocation.type, 'o')], docs='''Returns the Direction from this location to the specified location.
If the locations are equal this method returns Center.

 * DifferentPlanet - The locations are on different planets.''')
MapLocation.method(boolean.type, 'is_adjacent_to', [Var(MapLocation.type, 'o')], docs='''
Determines whether this location is adjacent to the specified location,
including diagonally. Note that squares are not adjacent to themselves,
and squares on different planets are not adjacent to each other.''')
MapLocation.method(boolean.type, 'is_within_range', [Var(u32.type, 'range'), Var(MapLocation.type, 'o')], docs='''
Whether this location is within the distance squared range of the
specified location, inclusive. False for locations on different planets.''')
MapLocation.debug()
MapLocation.clone()
MapLocation.eq()
MapLocation.serialize()

UnitID = p.typedef('unit::UnitID', u16.type)
Rounds = p.typedef('world::Rounds', u32.type)
TeamArray = p.vec(i32.type)

Location = p.struct('location::Location')
Location.method(Location.type, 'new_on_map', [Var(MapLocation.type, 'map_location')], docs='''Constructs a new location on the map.''', static=True)
Location.method(Location.type, 'new_in_garrison', [Var(UnitID.type, 'id')], docs='''Constructs a new location in a garrison.''', static=True)
Location.method(Location.type, 'new_in_space', [], docs='''Constructs a new location in space.''', static=True)
Location.method(boolean.type, 'is_on_map', [], docs='''Whether the unit is on a map.''')
Location.method(boolean.type, 'is_on_planet', [Var(Planet.type, 'planet')], docs='''True if and only if the location is on the map and on this planet.''')
Location.method(MapLocation.type.result(), 'map_location', [], docs='''The map location of the unit.

 * UnitNotOnMap - The unit is in a garrison or in space, and does not
   have a map location.''')
Location.method(boolean.type, 'is_in_garrison', [], docs='''Whether the unit is in a garrison.''')
Location.method(UnitID.type.result(), 'structure', [], docs='''The structure whose garrison the unit is in.

 * UnitNotInGarrison - the unit is not in a garrison.''')
Location.method(boolean.type, 'is_in_space', [], docs='''Whether the unit is in space.''')
Location.method(boolean.type, 'is_adjacent_to', [Var(Location.type, 'o')], docs='''Determines whether this location is adjacent to the specified location,
including diagonally. Note that squares are not adjacent to themselves,
and squares on different planets are not adjacent to each other. Also,
nothing is adjacent to something not on a map.''')
Location.method(boolean.type, 'is_within_range', [Var(u32.type, 'range'), Var(Location.type, 'o')], docs='''Whether this location is within the distance squared range of the
specified location, inclusive. False for locations on different planets.
Note that nothing is within the range of something not on the map.''')
Location.debug()
Location.clone()
Location.eq()
Location.serialize()

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
UnitIDVec = p.vec(UnitID.type)

UnitType = p.c_enum("unit::UnitType",
    docs="The different unit types, which include factories, rockets, and the robots.")
UnitType.variant('Worker', 0, docs="Workers are the foundation of the civilization.")
UnitType.variant('Knight', 1, docs="Knights are a melee unit that is strong in numbers.")
UnitType.variant('Ranger', 2, docs="Rangers are a ranged unit with good all-around combat.")
UnitType.variant('Mage', 3, docs="Mages are a fragile but specialized ranged unit for large areas.")
UnitType.variant('Healer', 4, docs="Healers are a suport unit that can heal other units.")
UnitType.variant('Factory', 5, docs="Factories are the hub for producing combative robots.")
UnitType.variant('Rocket', 6, docs="Rockets are the only unit that can move between planets.")
UnitType.serialize()
UnitType.method(boolean.type, "is_robot", [], docs="Whether the unit type is a robot.")
UnitType.method(boolean.type, "is_structure", [], docs="Whether the unit type is a structure.")
UnitType.method(u32.type.result(), "factory_cost", [], docs="""The cost of the unit in a factory.
Errors if the unit cannot be produced in a factory.""")
UnitType.method(u32.type.result(), "blueprint_cost", [], docs="""The cost to blueprint the unit.
Errors if the unit cannot be blueprinted.""")
UnitType.method(u32.type.result(), "replicate_cost", [], docs="""The cost to replicate the unit.
Errors if the unit cannot be replicated.""")
UnitType.method(u32.type, "value", [], docs="The value of a unit, as relevant to tiebreakers.")
UnitTypeVec = p.vec(UnitType.type)

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
Errors if the unit is not a robot.""", getter=True)
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
Unit.method(MapLocation.type.result(), "ranger_target_location", [], docs="""The target location for ranger's snipe.
Errors if the unit is not a ranger, or if the ranger is not sniping.""")
Unit.method(boolean.type.result(), "ranger_is_sniping", [], docs="""Whether the ranger is sniping.
Errors if the unit is not a ranger.""", getter=True)
Unit.method(boolean.type.result(), "ranger_max_countdown", [], docs="""The maximum countdown for ranger's snipe, which is the number of turns
that must pass before the snipe is executed.
Errors if the unit is not a ranger.""", getter=True)
Unit.method(u32.type.result(), "healer_self_heal_amount", [], docs="""The amount of health passively restored to itself each round.
Errors if the unit is not a healer.""", getter=True)
Unit.method(boolean.type.result(), "structure_is_built", [], docs="""Whether this structure has been built.
Errors if the unit is not a structure.""", getter=True)
Unit.method(usize.type.result(), "structure_max_capacity", [], docs="""The max capacity of a structure.
Errors if the unit is not a structure.""", getter=True)
Unit.method(UnitIDVec.type.result(), "structure_garrison", [], docs="""Returns the units in the structure's garrison.
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
UnitVec = p.vec(Unit.type)

PlanetMap = p.struct('map::PlanetMap', docs="The map for one of the planets in the Battlecode world. This information defines the terrain, dimensions, and initial units of the planet.")
PlanetMap.member(Planet.type, 'planet', docs="The planet of the map.")
PlanetMap.member(usize.type, 'height', docs="The height of this map, in squares. Must be in the range [MAP_HEIGHT_MIN, MAP_HEIGHT_MAX], inclusive.")
PlanetMap.member(usize.type, 'width', docs="The height of this map, in squares. Must be in the range [MAP_WIDTH_MIN, MAP_WIDTH_MAX], inclusive.")
PlanetMap.member(UnitVec.type, 'initial_units', docs="The initial units on the map. Each team starts with 1 to 3 Workers on Earth.")
PlanetMap.method(void.type.result(), 'validate', [], docs='''Validates the map and checks some invariants are followed.

 * InvalidMapObject - the planet map is invalid.''')
PlanetMap.method(boolean.type, 'on_map', [Var(MapLocation.type, 'location')], docs="Whether a location is on the map.")
PlanetMap.method(boolean.type.result(), 'is_passable_terrain_at', [Var(MapLocation.type, 'location')], docs='''
Whether the location on the map contains passable terrain. Is only false when the square contains impassable terrain (distinct from containing a building, for instance).

LocationOffMap - the location is off the map.''')
PlanetMap.method(u32.type.result(), 'initial_karbonite_at', [Var(MapLocation.type, 'location')], docs='''The amount of Karbonite initially deposited at the given location.

LocationOffMap - the location is off the map.''')
PlanetMap.clone()
PlanetMap.serialize()

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
ViewerKeyframe = p.struct('schema::ViewerKeyframe')
ViewerKeyframe.serialize()

ErrorMessage = p.struct('schema::ErrorMessage')
ErrorMessage.member(p.string.type, "error")
ErrorMessage.serialize()
ErrorMessage.debug()

TurnApplication = p.struct("controller::TurnApplication")
TurnApplication.member(StartTurnMessage.type, 'start_turn')
TurnApplication.member(ViewerMessage.type, 'viewer')

InitialTurnApplication = p.struct("controller::InitialTurnApplication")
InitialTurnApplication.member(StartTurnMessage.type, 'start_turn')
InitialTurnApplication.member(ViewerKeyframe.type, 'viewer')

AsteroidStrike = p.struct("map::AsteroidStrike", docs="A single asteroid strike on Mars.")
AsteroidStrike.constructor("new", [Var(u32.type, "karbonite"), Var(MapLocation.type, "location")])
AsteroidStrike.member(u32.type, "karbonite")
AsteroidStrike.member(MapLocation.type, "location")
AsteroidStrike.clone()
AsteroidStrike.debug()
AsteroidStrike.serialize()
AsteroidStrike.eq()

AsteroidPattern = p.struct("map::AsteroidPattern", docs="The asteroid pattern, defined by the timing and contents of each asteroid strike.")
AsteroidPattern.constructor("random", [Var(u16.type, "seed"), Var(PlanetMap.type.ref(), "mars_map")], docs='''Constructs a pseudorandom asteroid pattern given a map of Mars.''')
AsteroidPattern.method(void.type.result(), "validate", [], docs='''Validates the asteroid pattern.

 * InvalidMapObject - the asteroid pattern is invalid.''')
AsteroidPattern.method(boolean.type, "has_asteroid", [Var(Rounds.type, "round")], docs='''Whether there is an asteroid strike at the given round.''')
AsteroidPattern.method(AsteroidStrike.type.ref().result(), "asteroid", [Var(Rounds.type, "round")], docs='''Get the asteroid strike at the given round.

 * NullValue - There is no asteroid strike at this round.''')
AsteroidPattern.clone()
AsteroidPattern.debug()
AsteroidPattern.serialize()

OrbitPattern = p.struct("map::OrbitPattern", docs="The orbit pattern that determines a rocket's flight duration. This pattern is a sinusoidal function y=a*sin(bx)+c.")
OrbitPattern.member(Rounds.type, "amplitude", docs="Amplitude of the orbit.")
OrbitPattern.member(Rounds.type, "period", docs="The period of the orbit.")
OrbitPattern.member(Rounds.type, "center", docs="The center of the orbit.")
OrbitPattern.constructor('new', [Var(Rounds.type, 'amplitude'), Var(Rounds.type, 'period'), Var(Rounds.type, 'center')], docs='''Construct a new orbit pattern. This pattern is a sinusoidal function y=a*sin(bx)+c, where the x-axis is the round number of takeoff and the the y-axis is the duration of flight to the nearest integer.

The amplitude, period, and center are measured in rounds.''')
OrbitPattern.method(void.type.result(), 'validate', [], docs='''Validates the orbit pattern.

 * InvalidMapObject - the orbit pattern is invalid.''')
OrbitPattern.method(Rounds.type, 'duration', [Var(Rounds.type, 'round')], "Get the duration of flight if the rocket were to take off from either planet on the given round.")
OrbitPattern.serialize()

GameMap = p.struct('map::GameMap', docs="The map defining the starting state for an entire game.")
GameMap.member(u16.type, 'seed', docs="Seed for random number generation.")
GameMap.member(PlanetMap.type, 'earth_map', docs="Earth map.")
GameMap.member(PlanetMap.type, 'mars_map', docs="Mars map.")
GameMap.member(AsteroidPattern.type, 'asteroids', docs="The asteroid strike pattern on Mars.")
GameMap.member(OrbitPattern.type, 'orbit', docs="The orbit pattern that determines a rocket's flight duration.")
GameMap.method(void.type.result(), 'validate', [], docs='''Validate the game map.

 * InvalidMapObject - the game map is invalid.''')
GameMap.method(GameMap.type, 'test_map', [], static=True)
GameMap.clone()
GameMap.serialize()

# p.function(Level.type, 'research::max_level', [Var(UnitType.type.ref(), 'branch')])
# p.function(Rounds.type.result(), 'research::cost_of', [Var(UnitType.type.ref(), 'branch'), Var(Level.type, 'level')])
ResearchInfo = p.struct("research::ResearchInfo", docs="The status of research for a single team.")
ResearchInfo.constructor('new', [], docs="Construct an initial research state.")
ResearchInfo.method(Level.type, 'get_level', [Var(UnitType.type.ref(), 'branch')], docs='''Returns the current level of the research branch.''')
ResearchInfo.method(UnitTypeVec.type, 'get_queue', [], docs="Returns the research queue, where the front of the queue is at the beginning of the list.")
ResearchInfo.method(boolean.type, 'has_next_in_queue', [], "Whether there is a branch in the research queue.")
ResearchInfo.method(UnitType.type.result(), 'next_in_queue', [], docs='''Returns the next branch to be researched, which is the branch at the front of the research queue.

 * NullValue - There is no branch to be researched.''')
ResearchInfo.method(Rounds.type.result(), 'rounds_left', [], docs='''Returns the number of rounds left until the upgrade at the front of the research queue is applied.

 * NullValue - There is no branch to be researched.''')
ResearchInfo.serialize()

RocketLanding = p.struct("rockets::RocketLanding")
RocketLanding.constructor("new", [Var(UnitID.type, "rocket_id"), Var(MapLocation.type, "destination")])
RocketLanding.member(UnitID.type, "rocket_id", docs="The ID of the rocket.")
RocketLanding.member(MapLocation.type, "destination", docs="The landing destination of the rocket.")
RocketLanding.clone()
RocketLanding.debug()
RocketLanding.serialize()
RocketLanding.eq()
RocketLandingVec = p.vec(RocketLanding.type)

RocketLandingInfo = p.struct("rockets::RocketLandingInfo")
RocketLandingInfo.constructor("new", [], docs="Construct an empty rocket landing info.")
RocketLandingInfo.method(RocketLandingVec.type, 'landings_on', [Var(Rounds.type, 'round')], docs="Get the rocket landings on this round.")
RocketLandingInfo.clone()
RocketLandingInfo.debug()
RocketLandingInfo.serialize()
RocketLandingInfo.eq()

GameController = p.struct('controller::GameController')
GameController.constructor("new_player_env", [], docs="Use environment variables to connect to the manager.", result=True)
GameController.method(void.type.result(), "next_turn", [], docs="Send the moves from the current turn and wait for the next turn.")


GameController.method(Rounds.type, 'round', [])
GameController.method(Planet.type, 'planet', [])
GameController.method(Team.type, 'team', [])
GameController.method(u32.type, 'karbonite', [])
GameController.method(Unit.type.result(), "unit", [Var(UnitID.type, "id"),], docs="""The unit controller for the unit of this ID. Use this method to get
detailed statistics on a unit in your Var(heat.type, "team"), cooldowns, and
properties of special abilities like units garrisoned in a rocket.

Note that mutating this object does NOT have any effect on the actual
game. You MUST call the mutators in world!!

* GameError::NoSuchUnit - the unit does not exist (inside the vision range).
* GameError::TeamNotAllowed - the unit is not on the current player's team.

""")
GameController.method(UnitVec.type, "units", [], docs="""All the units within the vision range, in no particular order.
Does not include units in space.
""")
GameController.method(UnitVec.type, "my_units", [], docs="""All the units on your team, in no particular order.
Does not include units in space.
""")
GameController.method(UnitVec.type, "units_in_space", [], docs="""All the units of this team that are in space.""")
GameController.method(u32.type.result(), "karbonite_at", [Var(MapLocation.type, "location")], docs="""The karbonite at the given location.
""")
GameController.method(boolean.type, "can_sense_location", [Var(MapLocation.type, "location")], docs="""
Whether the location is within the vision range.

* GameError::InvalidLocation - the location is outside the vision range.
""")
GameController.method(boolean.type, "can_sense_unit", [Var(UnitID.type, "id")], docs="""Whether there is a unit with this ID within the vision range.
""")
GameController.method(UnitVec.type, "sense_nearby_units", [Var(MapLocation.type, "location"), Var(u32.type, "radius")], docs="""Sense units near the location within the given radius, inclusive, in
distance squared. The units are within the vision range.
""")
GameController.method(UnitVec.type, "sense_nearby_units_by_team", [Var(MapLocation.type, "location"),Var(u32.type, "radius"), Var(Team.type, "team")], docs="""Sense units near the location within the given radius, inclusive, in
distance squared. The units are within the vision range. Additionally
filters the units by team.
""")
GameController.method(UnitVec.type, "sense_nearby_units_by_type", [Var(MapLocation.type, "location"),Var(u32.type, "radius"), Var(UnitType.type, "unit_type")], docs="""Sense units near the location within the given radius, inclusive, in
distance squared. The units are within the vision range. Additionally
filters the units by unit type.
""")
GameController.method(Unit.type.result(), "sense_unit_at_location", [Var(MapLocation.type, "location")], docs="""The unit at the location, if it exists.
* GameError::InvalidLocation - the location is outside the vision range.
""")
GameController.method(boolean.type, "has_unit_at_location", [Var(MapLocation.type, "location")], docs="""Whether there is a visible unit at a location.
* GameError::InvalidLocation - the location is outside the vision range.
""")

GameController.method(AsteroidPattern.type, "asteroid_pattern", [], docs="""The asteroid strike pattern on Mars.
""")
GameController.method(OrbitPattern.type, "orbit_pattern", [], docs="""The orbit pattern that determines a rocket's flight duration.
""")
GameController.method(Rounds.type, "current_duration_of_flight", [], docs="""The current duration of flight if a rocket were to be launched this
round. Does not take into account any research done on rockets.
""")
#GameController.method(TeamArray.type.ref(), "get_team_array", [Var(Planet.type, "planet")], docs="""Gets a read-only version of this planet's team array. If the given
#planet is different from the planet of the player, reads the version
#of the planet's team array from COMMUNICATION_DELAY rounds prior.
#""")
GameController.method(void.type.result(), "write_team_array", [Var(usize.type, "index"), Var(i32.type, "value")], docs="""Writes the value at the index of this planet's team array.
* GameError::ArrayOutOfBounds - the index of the array is out of
  bounds. It must be within [0, COMMUNICATION_ARRAY_LENGTH).
""")
GameController.method(void.type.result(), "disintegrate_unit", [Var(UnitID.type, "unit_id")], docs="""Disintegrates the unit and removes it from the map. If the unit is a
factory or a rocket, also disintegrates any units garrisoned inside it.

* GameError::NoSuchUnit - the unit does not exist (inside the vision range).
* GameError::TeamNotAllowed - the unit is not on the current player's team.
""")
GameController.method(boolean.type.result(), "is_occupiable", [Var(MapLocation.type, "location")], docs="""Whether the location is clear for a unit to occupy, either by movement
or by construction.
* GameError::InvalidLocation - the location is outside the vision range.
""")
GameController.method(boolean.type, "can_move", [Var(UnitID.type, "robot_id"), Var(Direction.type, "direction")], docs="""Whether the robot can move in the given direction, without taking into
account the unit's movement heat. Takes into account only the map
terrain, positions of other robots, and the edge of the game map.
""")
GameController.method(boolean.type, "is_move_ready", [Var(UnitID.type, "robot_id")], docs="""Whether the robot is ready to move. Tests whether the robot's attack
heat is sufficiently low.
""")
GameController.method(void.type.result(), "move_robot", [Var(UnitID.type, "robot_id"), Var(Direction.type, "direction")], docs="""Moves the robot in the given direction.""")
GameController.method(boolean.type, "can_attack", [Var(UnitID.type, "robot_id"), Var(UnitID.type, "target_unit_id")], docs="""* GameError::NoSuchUnit - the unit does not exist (inside the vision range).
Whether the robot can attack the given unit, without taking into
account the unit's attack heat. Takes into account only the unit's
attack range, and the location of the unit.

* GameError::TeamNotAllowed - the unit is not on the current player's team.
* GameError::InappropriateUnitType - the unit is not a robot.
* GameError::InvalidAction - the robot cannot move in that direction.
""")
GameController.method(boolean.type, "is_attack_ready", [Var(UnitID.type, "robot_id")], docs="""Whether the robot is ready to attack. Tests whether the robot's attack
heat is sufficiently low.
* GameError::NoSuchUnit - the unit does not exist (inside the vision range).
* GameError::TeamNotAllowed - the unit is not on the current player's team.
* GameError::InappropriateUnitType - the unit is a healer, or not a robot.
""")
GameController.method(void.type.result(), "attack", [Var(UnitID.type, "robot_id"), Var(UnitID.type, "target_unit_id")], docs="""Attacks the robot, dealing the unit's standard amount of damage.
* GameError::NoSuchUnit - the unit does not exist (inside the vision range).
* GameError::TeamNotAllowed - the unit is not on the current player's team.
* GameError::InappropriateUnitType - the unit is a healer, or not a robot.
* GameError::InvalidAction - the robot cannot attack that location.
""")
GameController.method(ResearchInfo.type.result(), "research_info", [], docs="""The research info of the current team, including what branch is
currently being researched, the number of rounds left.

Note that mutating this object by resetting or queueing research
does not have any effect. You must call the mutators on world.
""")
GameController.method(boolean.type.result(), "reset_research", [], docs="""Resets the research queue to be empty. Returns true if the queue was
not empty before, and false otherwise.
""")
GameController.method(boolean.type.result(), "queue_research", [Var(UnitType.type, "branch")], docs="""Adds a branch to the back of the queue, if it is a valid upgrade, and
starts research if it is the first in the queue.

Returns whether the branch was successfully added.
""")
GameController.method(boolean.type, "can_harvest", [Var(UnitID.type, "worker_id"), Var(Direction.type, "direction")], docs="""Whether the worker is ready to harvest, and the given direction contains
karbonite to harvest. The worker cannot already have performed an action 
this round.
""")
GameController.method(void.type.result(), "harvest", [Var(UnitID.type, "worker_id"), Var(Direction.type, "direction")], docs="""Harvests up to the worker's harvest amount of karbonite from the given
location, adding it to the team's resource pool.

* GameError::NoSuchUnit - the unit does not exist (inside the vision range).
* GameError::TeamNotAllowed - the unit is not on the current player's team.
* GameError::InappropriateUnitType - the unit is not a worker.
* GameError::InvalidLocation - the location is off the map.
* GameError::InvalidAction - the worker is not ready to harvest, or there is no karbonite.
""")

GameController.method(boolean.type, "can_blueprint", [Var(UnitID.type, "worker_id"), Var(UnitType.type, "unit_type"), Var(Direction.type, "direction")], docs="""Whether the worker can blueprint a unit of the given type. The worker
can only blueprint factories, and rockets if Rocketry has been
researched. The team must have sufficient karbonite in its resource
pool. The worker cannot already have performed an action this round.
""")
GameController.method(void.type.result(), "blueprint", [Var(UnitID.type, "worker_id"), Var(UnitType.type, "structure_type"), Var(Direction.type, "direction")], docs="""Blueprints a unit of the given type in the given direction. Subtract
cost of that unit from the team's resource pool.

* GameError::NoSuchUnit - the unit does not exist (inside the vision range).
* GameError::TeamNotAllowed - the unit is not on the current player's team.
* GameError::InappropriateUnitType - the unit is not a worker, or the
  unit type is not a factory or rocket.
* GameError::InvalidLocation - the location is off the map.
* GameError::InvalidAction - the worker is not ready to blueprint.
""")
GameController.method(boolean.type, "can_build", [Var(UnitID.type, "worker_id"), Var(UnitID.type, "blueprint_id")], docs="""Whether the worker can build a blueprint with the given ID. The worker
and the blueprint must be adjacent to each other. The worker cannot
already have performed an action this round.
""")
GameController.method(void.type.result(), "build", [Var(UnitID.type, "worker_id"), Var(UnitID.type, "blueprint_id")], docs="""Builds a given blueprint, increasing its health by the worker's build
amount. If raised to maximum health, the blueprint becomes a completed
structure.

* GameError::NoSuchUnit - a unit does not exist.
* GameError::TeamNotAllowed - a unit is not on the current player's team.
* GameError::InappropriateUnitType - the unit or blueprint is the wrong type.
* GameError::InvalidAction - the worker cannot build the blueprint.
""")
GameController.method(boolean.type, "can_repair", [Var(UnitID.type, "worker_id"), Var(UnitID.type, "structure_id")], docs="""Whether the given worker can repair the given strucutre. Tests that the worker
is able to execute a worker action, that the structure is built, and that the
structure is within range.
""")
GameController.method(void.type.result(), "repair", [Var(UnitID.type, "worker_id"), Var(UnitID.type, "structure_id")], docs="""Commands the worker to repair a structure, repleneshing health to it. This
can only be done to structures which have been fully built.
""")
GameController.method(boolean.type, "can_replicate", [Var(UnitID.type, "worker_id"), Var(Direction.type, "direction")], docs="""Whether the worker is ready to replicate. Tests that the worker's
ability heat is sufficiently low, that the team has sufficient
karbonite in its resource pool, and that the square in the given
direction is empty.
""")
GameController.method(void.type.result(), "replicate", [Var(UnitID.type, "worker_id"), Var(Direction.type, "direction")], docs="""Replicates a worker in the given direction. Subtracts the cost of the
worker from the team's resource pool.

* GameError::NoSuchUnit - the unit does not exist (inside the vision range).
* GameError::TeamNotAllowed - the unit is not on the current player's team.
* GameError::InappropriateUnitType - the unit is not a worker.
* GameError::InvalidLocation - the location is off the map.
* GameError::InvalidAction - the worker is not ready to replicate.
""")
GameController.method(boolean.type, "can_javelin", [Var(UnitID.type, "knight_id"), Var(UnitID.type, "target_unit_id")], docs="""Whether the knight can javelin the given robot, without taking into
account the knight's ability heat. Takes into account only the knight's
ability range, and the location of the robot.
""")
GameController.method(boolean.type, "is_javelin_ready", [Var(UnitID.type, "knight_id")], docs="""Whether the knight is ready to javelin. Tests whether the knight's
ability heat is sufficiently low.
""")
GameController.method(void.type.result(), "javelin", [Var(UnitID.type, "knight_id"), Var(UnitID.type, "target_unit_id")], docs="""Javelins the robot, dealing the amount of ability damage.
* GameError::InvalidResearchLevel - the ability has not been researched.
* GameError::NoSuchUnit - the unit does not exist (inside the vision range).
* GameError::TeamNotAllowed - the unit is not on the current player's team.
* GameError::InappropriateUnitType - the unit is not a knight.
* GameError::InvalidAction - the knight cannot javelin that unit.
""")
GameController.method(void.type.result(), "begin_snipe", [Var(UnitID.type, "ranger_id"), Var(MapLocation.type, "location")], docs="""Begins the countdown to snipe a given location. Maximizes the units
attack and movement heats until the ranger has sniped. The ranger may
begin the countdown at any time, including resetting the countdown
to snipe a different location.

* GameError::InvalidResearchLevel - the ability has not been researched.
* GameError::NoSuchUnit - the unit does not exist (inside the vision range).
* GameError::TeamNotAllowed - the unit is not on the current player's team.
* GameError::InappropriateUnitType - the unit is not a ranger.
* GameError::InvalidLocation - the location is off the map or on a different planet.
""")
GameController.method(boolean.type, "can_blink", [Var(UnitID.type, "mage_id"), Var(MapLocation.type, "location")], docs="""Whether the mage can blink to the given location, without taking into
account the mage's ability heat. Takes into account only the mage's
ability range, the map terrain, positions of other units, and the edge
of the game map.
""")
GameController.method(boolean.type, "is_blink_ready", [Var(UnitID.type, "mage_id")], docs="""Whether the mage is ready to blink. Tests whether the mage's ability
heat is sufficiently low.

* GameError::InvalidResearchLevel - the ability has not been researched.
* GameError::NoSuchUnit - the unit does not exist (inside the vision range).
* GameError::TeamNotAllowed - the unit is not on the current player's team.
* GameError::InappropriateUnitType - the unit is not a mage.
""")
GameController.method(void.type.result(), "blink", [Var(UnitID.type, "mage_id"), Var(MapLocation.type, "location")], docs="""Blinks the mage to the given location.
* GameError::InvalidResearchLevel - the ability has not been researched.
* GameError::NoSuchUnit - the unit does not exist (inside the vision range).
* GameError::TeamNotAllowed - the unit is not on the current player's team.
* GameError::InappropriateUnitType - the unit is not a mage.
* GameError::InvalidAction - the mage cannot blink to that location.
""")
GameController.method(boolean.type, "can_heal", [Var(UnitID.type, "healer_id"), Var(UnitID.type, "target_robot_id")], docs="""Whether the healer can heal the given robot, without taking into
account the healer's attack heat. Takes into account only the healer's
attack range, and the location of the robot.
""")
GameController.method(boolean.type, "is_heal_ready", [Var(UnitID.type, "healer_id")], docs="""Whether the healer is ready to heal. Tests whether the healer's attack
heat is sufficiently low.
""")
GameController.method(void.type.result(), "heal", [Var(UnitID.type, "healer_id"), Var(UnitID.type, "target_robot_id")], docs="""Heals the robot, dealing the healer's standard amount of "damage".
* GameError::NoSuchUnit - a unit does not exist.
* GameError::TeamNotAllowed - the first unit is not on the current player's team.
* GameError::InappropriateUnitType - the healer or robot is not the right type.
* GameError::InvalidAction - the healer cannot heal that unit.
""")
GameController.method(boolean.type, "can_overcharge", [Var(UnitID.type, "healer_id"), Var(UnitID.type, "target_robot_id")], docs="""Whether the healer can overcharge the given robot, without taking into
account the healer's ability heat. Takes into account only the healer's
ability range, and the location of the robot.
""")
GameController.method(boolean.type, "is_overcharge_ready", [Var(UnitID.type, "healer_id")], docs="""Whether the healer is ready to overcharge. Tests whether the healer's
ability heat is sufficiently low.
""")
GameController.method(void.type.result(), "overcharge", [Var(UnitID.type, "healer_id"), Var(UnitID.type, "target_robot_id")], docs="""Overcharges the robot, resetting the robot's cooldowns.
* GameError::InvalidResearchLevel - the ability has not been researched.
* GameError::NoSuchUnit - a unit does not exist.
* GameError::TeamNotAllowed - the first unit is not on the current player's team.
* GameError::InappropriateUnitType - the healer or robot is not the right type.
* GameError::InvalidAction - the healer cannot overcharge that unit.
""")
GameController.method(boolean.type, "can_load", [Var(UnitID.type, "structure_id"), Var(UnitID.type, "robot_id")], docs="""Whether the robot can be loaded into the given structure's garrison. The robot
must be ready to move and must be adjacent to the structure. The structure
and the robot must be on the same team, and the structure must have space.
""")
GameController.method(void.type.result(), "load", [Var(UnitID.type, "structure_id"), Var(UnitID.type, "robot_id")], docs="""Loads the robot into the garrison of the structure.
* GameError::NoSuchUnit - a unit does not exist.
* GameError::TeamNotAllowed - either unit is not on the current player's team.
* GameError::InappropriateUnitType - the robot or structure are the wrong type.
* GameError::InvalidAction - the robot cannot be loaded inside the structure.
""")
GameController.method(boolean.type, "can_unload", [Var(UnitID.type, "structure_id"), Var(Direction.type, "direction")], docs="""Tests whether the given structure is able to unload a unit in the
given direction. There must be space in that direction, and the unit
must be ready to move.
""")
GameController.method(void.type.result(), "unload", [Var(UnitID.type, "structure_id"), Var(Direction.type, "direction")], docs="""Unloads a robot from the garrison of the specified structure into an 
adjacent space. Robots are unloaded in the order they were loaded.

* GameError::NoSuchUnit - the unit does not exist (inside the vision range).
* GameError::TeamNotAllowed - the unit is not on the current player's team.
* GameError::InappropriateUnitType - the unit is not a structure.
* GameError::InvalidLocation - the location is off the map.
* GameError::InvalidAction - the rocket cannot degarrison a unit.
""")
GameController.method(boolean.type, "can_produce_robot", [Var(UnitID.type, "factory_id"), Var(UnitType.type, "robot_type")], docs="""Whether the factory can produce a robot of the given type. The factory
must not currently be producing a robot, and the team must have
sufficient resources in its resource pool.
""")
GameController.method(void.type.result(), "produce_robot", [Var(UnitID.type, "factory_id"), Var(UnitType.type, "robot_type")], docs="""Starts producing the robot of the given type.
* GameError::NoSuchUnit - the unit does not exist.
* GameError::TeamNotAllowed - the unit is not on the current player's team.
* GameError::InappropriateUnitType - the unit is not a factory, or the
  queued unit type is not a robot.
* GameError::InvalidAction - the factory cannot produce the robot.
""")
GameController.method(RocketLandingInfo.type, "rocket_landings", [], docs="""The landing rounds and locations of rockets in space that belong to the
current team.
Note that mutating this object does NOT have any effect on the actual
game. You MUST call the mutators in world!!
""")
GameController.method(boolean.type, "can_launch_rocket", [Var(UnitID.type, "rocket_id"), Var(MapLocation.type, "destination")], docs="""W
hether the rocket can launch into space. The rocket can launch if the
it has never been used before.
""")
GameController.method(void.type.result(), "launch_rocket", [Var(UnitID.type, "rocket_id"), Var(MapLocation.type, "location")], docs="""Launches the rocket into space. If the destination is not on the map of
the other planet, the rocket flies off, never to be seen again.
* GameError::NoSuchUnit - the unit does not exist (inside the vision range).
* GameError::TeamNotAllowed - the unit is not on the current player's team.
* GameError::InappropriateUnitType - the unit is not a rocket.
* GameError::InvalidAction - the rocket cannot launch.
""")
GameController.method(GameController.type, 'new_manager', [Var(GameMap.type, 'map')], static=True)
GameController.method(StartGameMessage.type, 'start_game', [Var(Player.type, 'player')])
GameController.method(TurnApplication.type, 'apply_turn', [Var(TurnMessage.type.ref(), 'turn')])
GameController.method(InitialTurnApplication.type, 'initial_start_turn_message', [])
GameController.method(boolean.type, "is_over", [])
GameController.method(Team.type.result(), "winning_team", [])

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