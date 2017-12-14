import battlecode.engine as eng

def test_game_world():
    world = eng.GameWorld()
    assert world.round == 1
    world2 = eng.GameWorld(world)
    assert world2.round == 1
    assert int(eng.ffi.cast('size_t', world._game_world)) != \
        int(eng.ffi.cast('size_t', world2._game_world))