'''
This is a test engine that plays tic tac toe to use with the server
'''
import numpy as np

def commit_actions(state, actions, player_id):
    '''
    Takes a series of actions and commits them to the state
    '''
    for action in actions:
        state = make_move(state, action, player_id)
    return state

def make_move(state, action, player_id):
    '''
    This function takes an action and a state and returns the new state for the
    next action
    '''
    # All errors are asserts since this is just for testing and stuff
    if state is None:
        assert False

    turn = state['turn']
    player_this_move = state['player_this']
    board = state['board']
    tile = action['tile']
    if player_id != action['player_id']:
        assert False

    if player_this_move != player_id:
        assert False

    new_state = {}
    new_state['turn'] = turn + 1

    if board[tile['x']][tile['y']] != 0:
        assert False

    board[tile['x']][tile['y']] = player_id

    new_state['board'] = board.copy()

    return new_state


def get_partial_state(state, player_id): #pylint: disable=unused-argument
    '''
    Get partial state for a given state
    '''
    return state


def get_state_diff(old_state, new_state, player_id): #pylint: disable=unused-argument
    '''
    Get a state diff between two planets
    '''
    return new_state

def init_state():
    '''
    Get a new state
    '''
    board_size = 3
    board = np.zeros((board_size, board_size))
    return board
