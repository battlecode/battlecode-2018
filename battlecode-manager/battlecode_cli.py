'''
t
This file contains contains the CLI that starts games up
'''

import time
import os
import logging
from os.path import abspath
from shutil import copytree, rmtree
from player_plain import PlainPlayer
from player_sandboxed import SandboxedPlayer
import server
import battlecode as bc
try:
    import ujson as json
except:
    import json
import io
import sys

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


def working_dir_message(working_dir):
    print('Working directory:', working_dir)
    print('You may want to empty it periodically.')


def prepare_working_directory(working_dir):
    bcdir = os.path.join(working_dir, 'battlecode')
    # todo: copy battlecode to working_dir
    if not os.path.exists(working_dir):
        # print("Creating working directory at", working_dir)
        os.makedirs(working_dir)
    if os.path.exists(bcdir):
        # print("Cleaning up old battlecode folder", bcdir)
        rmtree(bcdir)

    prepath = abspath(os.path.join(os.path.dirname(abspath(__file__)), "../battlecode"))
    # print("Copying battlecode resources from {} to {}".format(prepath, working_dir))
    copytree(prepath, bcdir)
    # print("Working dir ready!")


def run_game(game, dockers, args, sock_file):
    '''
    This contains the logic that needs to be cleaned up at the end of a game
    If there is something that needs to be cleaned up add it in the try catch
    loop surrounding the catch loop surrounding the call of the function
    '''

    # Start the unix stream server
    main_server = server.start_server(sock_file, game, dockers)

    viewer_server = server.start_viewer_server(PORT, game) if args['use_viewer'] else None

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
        time.sleep(0.1)

    print('Killing game server.')
    main_server.shutdown()
    try:
        main_server.server_close()
    except e:
        print(e)

    match_file = {}
    match_file['message'] = game.viewer_messages
    if not game.disconnected:
        if bc.Team.Red == game.manager.winning_team():
            winner = 'player1'
        else:
            winner = 'player2'
    else:
        winner = game.winner

    match_file['metadata'] = {
        'player1': args['dir_p1'][8:],
        'player2': args['dir_p2'][8:],
        'winner': winner
    }

    if args['docker']:
        match_output = abspath(os.path.join('/player', str(args['replay_filename'])))
    else:
        match_output = args['replay_filename']
        if not os.path.isabs(match_output):
            match_output = abspath(os.path.join('..', str(match_output)))

    print("Dumping matchfile to", match_output)
    match_ptr = open(match_output, 'w')
    json.dump(match_file, match_ptr)
    match_ptr.close()
    if viewer_server is not None:
        viewer_server.shutdown()

    return winner


def cleanup(dockers, args, sock_file):
    '''
    Clean up that needs to be done at the end of a game
    '''
    print("Cleaning up Docker and Socket...")
    for player_key in dockers:
        docker_inst = dockers[player_key]
        docker_inst.destroy()

    if isinstance(sock_file, str) or isinstance(sock_file, bytes):
        # only unlink unix sockets
        os.unlink(sock_file)

    print("Ready to run next game.")


def get_map(map_name):
    '''
    Read a map of a given name, and return a GameMap.
    '''

    try:
        with open(map_name) as f:
            contents = f.read()
        print("Loading map " + map_name)
        return bc.GameMap.from_json(contents)
    except Exception as e:
        try:
            with open('/player/' + map_name) as f:
               contents = f.read()
            print("Loading map " + map_name)
            return bc.GameMap.from_json(contents)
        except Exception as e:
            print("Loading test map...")
            return bc.GameMap.test_map()

def create_game(args):
    '''
    Create all the semi-permanent game structures (i.e. sockets and dockers and
    stuff
    '''

    # Load the Game state info
    game = server.Game(logging_level=logging.ERROR,
                       game_map=args['map'],
                       time_pool=args['time_pool'],
                       time_additional=args['time_additional'],
                       terminal_viewer=args['terminal_viewer'],
                       extra_delay=args['extra_delay'])

    working_dir = abspath("working_dir")
    prepare_working_directory(working_dir)

    # pick server location
    if 'USE_TCP' in os.environ or sys.platform == 'win32':
        print('Running game server on port tcp://localhost:16148')
        # int indicates tcp
        sock_file = ('localhost', 16148)
    else:
        # Find a good filename to use as socket file
        for index in range(10000):
            sock_file = "/tmp/battlecode-" + str(index)
            if not os.path.exists(sock_file):
                break
        else:
            raise Exception("Do you really have 10000 /tmp/battlecode sockets???")
        print('Running game server on socket unix://{}'.format(sock_file))

    # Assign the docker instances client ids
    dockers = {}
    for index in range(len(game.players)):
        key = [player['id'] for player in game.players][index]
        local_dir = args['dir_p1' if index % 2 == 0 else 'dir_p2']
        if args['docker']:
            # Note, importing a module multiple times is ok in python.
            # It might take a bit of time to verify that it is already imported though, but that should be negligable.
            import docker
            docker_instance = docker.from_env()
            dockers[key] = SandboxedPlayer(sock_file, working_dir=working_dir, docker_client=docker_instance, player_key=key, local_dir=local_dir)
        else:
            dockers[key] = PlainPlayer(sock_file, working_dir=working_dir, player_key=key, local_dir=local_dir)

    return (game, dockers, sock_file)
