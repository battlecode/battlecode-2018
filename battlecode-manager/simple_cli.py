import os
import sys
import argparse
import battlecode_cli as cli

map_extension = ".bc18map"
# TODO: Is this correct?
replay_extension = ".bc18replay"


def start_game(args):
    args['map'] = cli.get_map(args['map'])
    print(args)

    (game, sandboxes, sock_file) = cli.create_game(args)

    try:
        print("running game")
        winner = cli.run_game(game, sandboxes, args, sock_file)
    finally:
        cli.cleanup(sandboxes, args, sock_file)

    print("Winner is player: " + str(1 if winner == 'player1' else 2))


def run_game(map_path, player1dir, player2dir, replay_dir, docker=False):
    args = {}
    args['map'] = map_path
    args['dir_p2'] = player1dir
    args['dir_p1'] = player2dir
    args['docker'] = docker
    # TODO: Will cause name collisions if multiple instances run at the same time!
    args['replay_filename'] = os.path.join(replay_dir, "replay_" + str(len(os.listdir(replay_dir))) + replay_extension)
    # Not sure what this is yet...
    args['time_pool'] = 50
    args['time_additional'] = 10 * 1000
    args['use_viewer'] = False
    start_game(args)


def get_maps(map_directory):
    maps = [o for o in os.listdir(map_directory) if o.endswith(map_extension)]
    # This map is built-in
    maps.append('testmap.bc18map')
    return maps


map_directory = 'battlecode-maps'
parser = argparse.ArgumentParser(description='Run BattleCode 2018 matches')
parser.add_argument('-p1', '--player1', help="Path to the directory for player 1", required=True)
parser.add_argument('-p2', '--player2', help="Path to the directory for player 2", required=True)
parser.add_argument('-m', '--map', help="The map to play on. The available maps are:\n" + ", ".join(get_maps(map_directory)), required=True)
parser.add_argument('--replay-dir', help="Directory to save replays to. This may not work with docker.", default="replays", required=False)
parser.add_argument('--docker', action='store_const', const=True, default=False, help="Use Docker to run the game. This requires Docker to be installed and the gods to be on your side")

args = parser.parse_args()
map_path = args.map

replay_dir = os.path.abspath(args.replay_dir)
if not os.path.isdir(replay_dir):
    prompt = "Replay directory '" + args.replay_dir + "' does not exist. Do you want to create it? [y/N] "
    if input(prompt).strip() == "y":
        os.mkdir(replay_dir)
    else:
        exit(1)

if not map_path.endswith(map_extension):
    map_path += map_extension

if map_path not in get_maps(map_directory):
    print("Could not find any map named " + str(map_path) + ". Use --help to see a list of all available maps.")

try:
    run_game(map_path, args.player1, args.player2, replay_dir, docker=args.docker)
except KeyboardInterrupt:
    print("Stopping game")
    raise
