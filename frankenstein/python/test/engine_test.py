import battlecode as bc

def test_map_location():
    loc = bc.MapLocation(bc.Planet.Earth,1,2)
    assert loc.planet == bc.Planet.Earth
    assert loc.x == 1
    assert loc.y == 2
    loc.y = 3
    assert loc.y == 3
    loc.planet = bc.Planet.Mars
    assert loc.planet == bc.Planet.Mars