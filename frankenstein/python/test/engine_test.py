import battlecode as bc

def test_map_location():
    loc = bc.MapLocation(1,2)
    assert loc.x == 1
    assert loc.y == 2
    loc.y = 3
    assert loc.y == 3