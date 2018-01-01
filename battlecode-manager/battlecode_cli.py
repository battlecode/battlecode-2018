'''
This file contains contains the CLI that starts games up
'''

import argparse
import time
import os
import sandbox
import server

def run_game(game, dockers, parse_args, sock_file):
    '''
    This contains the logic that needs to be cleaned up at the end of a game
    If there is something that needs to be cleaned up add it in the try catch
    loop surrounding the catch loop surrounding the call of the function
    '''

    # Start the unix stream server
    server.start_server(sock_file, game, dockers,
                        use_docker=parse_args['use_docker'])

    if parse_args['use_viewer']:
        # TODO Start the Viewer
        pass

    # Start the docker instances
    if parse_args['use_docker']:
        for player_key in DOCKERS:
            docker_inst = DOCKERS[player_key]
            docker_inst.start()


    # Wait until all the code is done then clean up
    while not GAME.game_over:
        time.sleep(1)

def cleanup(dockers, parse_args, sock_file):
    '''
    Clean up that needs to be done at the end of a game
    '''
    print("Cleaning up Docker and Socket")
    if parse_args['use_docker']:
        for player_key in dockers:
            docker_inst = DOCKERS[player_key]
            docker_inst.destroy()
    os.unlink(sock_file)

def parse_args():
    '''
    Parse the arguments given to main
    '''
    # All the argument parsing will be in here
    parser = argparse.ArgumentParser(description="Run a battlecode game")

    parser.add_argument("-p1", help="Run file for player one", dest='p1', \
            required=True)
    parser.add_argument("-p2", help="Directory containing player two", dest='p2', \
            required=True)
    parser.add_argument("-m", "--map", help="map file to use this game", \
            dest="map")
    parser.add_argument("-d", "--use-docker", help="use docker for security", \
            dest="ud")
    parser.add_argument("-v", "--enable-viewer", help="Allow live streaming to \
            viewer", dest="vw")

    args = parser.parse_args()

    # Pre-processing based off arguments
    return_args = {}
    return_args['use_docker'] = (args.ud != None)
    return_args['use_viewer'] = (args.vw != None)

    return_args['dir_p1'] = os.path.abspath(args.p1)
    return_args['dir_p2'] = os.path.abspath(args.p2)

    # TODO Read the map, and load into game
    return_args['map_state'] = ''

    return return_args


def create_game(args):
    '''
    Create all the semi-permanent game structures (i.e. sockets and dockers and
    stuff
    '''
    # Load the Game state info
    game = server.Game(4, "null")


    # Find a good filename to use as socket file
    for index in range(10000):
        sock_file = "/tmp/battlecode-"+str(index)
        if not os.path.exists(sock_file):
            break

    # Assign the docker instances client ids
    dockers = {}
    if ARGS['use_docker']:
        for index in range(len(game.player_ids)):
            key = game.player_ids[index]
            if index % 2 == 0:
                new_docker = sandbox.Sandbox(sock_file, player_key=key,
                                             working_dir=args['dir_p1'])
            else:
                new_docker = sandbox.Sandbox(sock_file, player_key=key,
                                             working_dir=args['dir_p2'])
            dockers[key] = new_docker
    else:
        print("Socket: " + sock_file)

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
