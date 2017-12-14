'''The core engine of battlecode.'''

from __future__ import absolute_import, division, print_function
from builtins import (bytes, str, open, super, range,
                      zip, round, input, int, pow, object)

try:
    from ._engine import lib, ffi # type: ignore
except:
    raise ImportError('''Failed to import engine python extension;
    have you run setup.py build_ext --inplace?''')

_bc = lib.bc_init()
if _bc == ffi.NULL:
    raise ImportError("Failed to initialize battlecode engine")

class BattlecodeError(Exception):
    '''An internal error.'''
    def __init__(self, message):
        super().__init__()
        self.message = message

def _check_errors() -> None:
    '''Check for recent errors.'''
    err = lib.bc_extract_error(_bc)
    if err:
        message = ffi.string(err)
        lib.bc_free_error(_bc, err)
        raise BattlecodeError(message)

class GameWorld(object):
    '''A game world.'''

    def __init__(self, copyfrom: 'GameWorld'=None) -> None:
        '''Create a new game world.
        copyfrom: another world to deep copy.
        '''
        if copyfrom is None:
            self._game_world = lib.bc_new_game_world(_bc)
            _check_errors()
        else:
            self._game_world = lib.bc_clone_game_world(_bc, copyfrom._game_world)
            _check_errors()

    def __del__(self):
        lib.bc_free_game_world(_bc, self._game_world)
        _check_errors()
        self._game_world = None

    @property
    def round(self) -> int:
        '''Get the current round of the game world.'''
        result = lib.bc_get_round(_bc, self._game_world)
        _check_errors()
        return result

    def __copy__(self) -> 'GameWorld':
        return GameWorld(self)

    def __deepcopy__(self, _memo) -> 'GameWorld':
        return GameWorld(self)
