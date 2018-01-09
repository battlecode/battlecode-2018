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

# TODO port number
PORT = 808

def run_game(game, dockers, args, sock_file):
    '''
    This contains the logic that needs to be cleaned up at the end of a game
    If there is something that needs to be cleaned up add it in the try catch
    loop surrounding the catch loop surrounding the call of the function
    '''

    # Start the unix stream server
    server.start_server(sock_file, game, dockers)

    if args.use_viewer:
        # TODO check this function
        server.start_viewer_server(PORT, game)

    # Start the docker instances
    for player_key in DOCKERS:
        docker_inst = DOCKERS[player_key]
        docker_inst.start()
        docker_inst.stream_logs()

    # Wait until all the code is done then clean up
    while not GAME.game_over:
        time.sleep(1)

def cleanup(dockers, args, sock_file):
    '''
    Clean up that needs to be done at the end of a game
    '''
    print("Cleaning up Docker and Socket")
    for player_key in dockers:
        docker_inst = DOCKERS[player_key]
        logs = docker_inst.destroy()
    os.unlink(sock_file)

def parse_args():
    '''
    Parse the arguments given as env variables
    '''
    parser = argparse.ArgumentParser(description='The entrypoint for Battlecode 2017')
    command = parser.add_subparsers(dest="command")
    run = command.add_parser("run", help="Run a game.")
    run.add_argument('--use_viewer', default=True)
    run.add_argument('--p1', metavar='PLAYER1', help='The directory of player 1', dest='dir_p1')
    run.add_argument('--p1language', help='The language used by player 1', default='python')
    run.add_argument('--p2', metavar='PLAYER2', help='The directory of player 2', dest='dir_p2')
    run.add_argument('--p2language', help='The language used by player 2', default='python')
    run.add_argument('--map', metavar='MAP', help='The map to run the game on', default='default', dest='map')
    listmaps = command.add_parser("listmaps", help="List available maps.")

    result = parser.parse_args()
    print(result)
    return result

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
                       game_map=get_map(args.map))

    # Find a good filename to use as socket file
    for index in range(10000):
        sock_file = "/tmp/battlecode-"+str(index)
        if not os.path.exists(sock_file):
            break

    # Assign the docker instances client ids
    dockers = {}
    Sandbox.initialize()
    for index in range(len(game.players)):
        key = [player['id'] for player in game.players][index]
        dockers[key] = Sandbox(sock_file, player_key=key,
                               local_dir=args.dir_p1 if index % 2 == 0 else args.dir_p2)

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
else:
    raise("Should be called from command line")
