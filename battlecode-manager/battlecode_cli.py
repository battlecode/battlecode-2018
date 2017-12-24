'''
This file contains contains the CLI that starts games up
'''

import argparse

import server


class CLI(object):
    '''
    This will contain all of the CLI variables and parameters that we need to
    call on
    '''
    def __init__(self):
        pass


if __name__ == "__main__":
    # All the argument parsing will be in here
    PARSER = argparse.ArgumentParser(description="Run a battlecode game")

    PARSER.add_argument("-p1", help="Run file for player one", dest='p1', \
            required=True)

    PARSER.add_argument("-p2", help="Run file for player one", dest='p1', \
            required=True)

    ARGS = PARSER.parse_args()

    # Pre-processing based off arguments

    # Load the Game state info
    GAME = server.Game(2, "NULL")

    # After Parsing the arguments we create the docker instancse
    DOCKERS = {}

    # Assign the docker instances client ids

    # Start the unix stream server
    server.start_server(ARGS.sock_file, GAME, DOCKERS)

    # Start the docker instances


    # Wait until all the code is done then clean up
