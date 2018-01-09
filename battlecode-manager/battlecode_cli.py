'''
This file contains contains the CLI that starts games up

Requires the following env variables: PLAYER_MEM_LIMIT (eg '256m'), PLAYER_CPU_PERCENT (eg '20'), VIEWER, P1, P2, and
'''

import argparse
import time
import os
import logging
from sandbox import Sandbox
import server
import battlecode as bc
import ujson as json
import io

# TODO port number
PORT = 16147

class Logger(object):
    def __init__(self, prefix):
        self.logs = io.StringIO() 
        self.prefix = prefix

    def __call__(self, v):
        data = v.decode()
        self.logs.write(data)
        print(self.prefix, data, end='')

def run_game(game, dockers, args, sock_file):
    '''
    This contains the logic that needs to be cleaned up at the end of a game
    If there is something that needs to be cleaned up add it in the try catch
    loop surrounding the catch loop surrounding the call of the function
    '''

    # Start the unix stream server
    server.start_server(sock_file, game, dockers)

    if args['use_viewer']:
        viewer_server = server.start_viewer_server(PORT, game)

    # Start the docker instances
    for player_key in dockers:
        docker_inst = dockers[player_key]
        docker_inst.start()
        for player_ in game.players:
            if player_['id'] == player_key:
                player = player_['player']
                break
        if player.planet == bc.Planet.Earth:
            planet = 'earth'
        else:
            planet = 'mars'
        if player.team == bc.Team.Blue:
            team = 'blue'
        else:
            team = 'red'
        name = f'[{planet}:{team}]'
        logger = Logger(name)
        docker_inst.stream_logs(line_action=logger)
        player_['logger'] = logger

    # Wait until all the code is done then clean up
    while not game.game_over:
        time.sleep(1)

    print("Dumping matchfile")
    match_ptr = open("/player/" + str(args['replay_filename']), mode='w')
    match_file = {}
    match_file['message'] = game.viewer_messages
    if not game.disconnected:
        winner = game.manager.winning_team
    else:
        winner = game.winner

    match_file['metadata'] = {'player1': os.path.dirname(args['dir_p1']),
            'player2' : os.path.dirname(args['dir_p2']), 'winner': 'player1'}
    json.dump(match_file, match_ptr)
    match_ptr.close()
    if args['use_viewer']:
        viewer_server.shutdown()

def cleanup(dockers, args, sock_file):
    '''
    Clean up that needs to be done at the end of a game
    '''
    print("Cleaning up Docker and Socket")
    for player_key in dockers:
        docker_inst = dockers[player_key]
        logs = docker_inst.destroy()
        
    os.unlink(sock_file)

def parse_args():
    '''
    Parse the arguments given as env variables
    '''

    return_args = {}
    return_args['use_viewer'] = (os.environ['VIEWER'] != None)
    return_args['dir_p1'] = os.path.abspath(os.environ['P1'])
    return_args['dir_p2'] = os.path.abspath(os.environ['P2'])
    return_args['map'] = get_map(os.environ['MAP'])

    return return_args

def get_map(map_name):
    '''
    Read a map of a given name, and return a GameMap.

    TODO: actually read map files
    '''

    return bc.GameMap.test_map()

def create_game(args):
    '''
    Create all the semi-permanent game structures (i.e. sockets and dockers and
    stuff
    '''

    # Load the Game state info
    game = server.Game(logging_level=logging.ERROR,
                       game_map=args['map'], time_pool=args['time_pool'],
                       time_additional=args['time_additional'])

    # Find a good filename to use as socket file
    for index in range(10000):
        sock_file = "/tmp/battlecode-"+str(index)
        print(sock_file)
        print(os.path.exists(sock_file))
        if not os.path.exists(sock_file):
            break

    # Assign the docker instances client ids
    dockers = {}
    Sandbox.initialize()
    for index in range(len(game.players)):
        key = [player['id'] for player in game.players][index]
        dockers[key] = Sandbox(sock_file, player_key=key,
                               local_dir=args['dir_p1' if index % 2 == 0 else 'dir_p2'])

    return (game, dockers, sock_file)


if __name__ == "__main__":

    # Pars Arguments
    ARGS = parse_args()

    # Create static Game stuff
    (GAME, DOCKERS, SOCK_FILE) = create_game(ARGS)

    # After Parsing the arguments we create the docker instancse
    try:
        run_game(GAME, DOCKERS, ARGS, SOCK_FILE)
    except KeyboardInterrupt:
        cleanup(DOCKERS, ARGS, SOCK_FILE)
    finally:
        cleanup(DOCKERS, ARGS, SOCK_FILE)
