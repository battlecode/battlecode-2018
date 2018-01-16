import os
import argparse
import battlecode_cli as cli
import sys
try:
    import colorama
    colorama.init()
    CINIT=True
except:
    CINIT=False
    pass

map_extension = ".bc18map"
map_extension_text = ".bc18t"
replay_extension = ".bc18"

# ANSI escape codes
# See https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
color_red = "\033[31m"
color_reset = "\033[0m"

def run_game(map_path, player1dir, player2dir, replay_dir, docker, terminal_viewer, extra_delay, max_memory, initial_time, per_frame_time, proxy_test):
    args = {}
    args['dir_p1'] = player1dir
    args['dir_p2'] = player2dir
    args['docker'] = docker
    # TODO: Will cause name collisions if multiple instances run at the same time!
    args['replay_filename'] = os.path.join(replay_dir, "replay_" + str(len(os.listdir(replay_dir))) + replay_extension)
    args['player_memory'] = max_memory
    args['player_cpu'] = 20
    args['time_pool'] = initial_time
    args['time_additional'] = per_frame_time
    args['use_viewer'] = False
    args['terminal_viewer'] = terminal_viewer
    args['extra_delay'] = extra_delay
    args['map_name'] = map_path
    args['map'] = cli.get_map(map_path)

    if terminal_viewer and sys.platform == 'win32' and not CINIT:
        print('To get pretty output with -tv on windows, run `py -3 -m pip install colorama`')

    (game, sandboxes, sock_file) = cli.create_game(args)

    if proxy_test:
        import proxyuploader
        up = proxyuploader.ProxyUploader()
        up.game_id = 12312
        up.blue_id = 100
        up.blue_id = 1000
        up.game = game

    try:
        winner = cli.run_game(game, sandboxes, args, sock_file)
    finally:
        cli.cleanup(sandboxes, args, sock_file)

    if proxy_test:
        up.done = True

    print("Winner is player " + str(1 if winner == 'player1' else 2))

def get_maps(map_directory):
    maps = [o for o in os.listdir(map_directory) if o.endswith(map_extension) or o.endswith(map_extension_text)]
    # This map is built-in
    maps.append('testmap.bc18map')
    return maps

file_dir = os.path.dirname(os.path.realpath(__file__))
map_directory = os.path.abspath(file_dir + '/../battlecode-maps')
parser = argparse.ArgumentParser(
    "battlecode.sh",
    description='Run BattleCode 2018 matches'
)

parser.add_argument('-p1', '--player1', help="Path to the directory for player 1", required=True)
parser.add_argument('-p2', '--player2', help="Path to the directory for player 2", required=True)
map_names = ", ".join(s.replace(map_extension, "").replace(map_extension_text, "") for s in get_maps(map_directory))
parser.add_argument('-m', '--map', help="The map to play on. The available maps are:\n" + map_names, required=True)
parser.add_argument('--replay-dir', help="Directory to save replays to. This may not work with docker. (default: %(default)s)", default="replays", required=False)
parser.add_argument('--mem', type=int, help='Memory in megabytes that a player is allowed to use. (default: %(default)s)', default=256)
parser.add_argument('--docker', action='store_const', const=True, default=False, help="Use Docker to run the game. This requires Docker to be installed and the gods to be on your side")
parser.add_argument('--unlimited-time', action='store_const', const=True, default=False, help='Allow players to use an unlimited amount of time')
parser.add_argument('-tv', '--terminal-viewer', action='store_const', const=True, default=False, help="Print game images in the terminal.")
parser.add_argument('-ed', '--extra-delay', type=int, default=0, help="add extra delay after each turn (make -tv slower)")
parser.add_argument('--proxy-test', action='store_true', help="do some useless nonsense")

args = parser.parse_args()
map_path = args.map

# Input validation

replay_dir = os.path.abspath(args.replay_dir)
if not os.path.isdir(replay_dir):
    prompt = "Replay directory '" + args.replay_dir + "' does not exist. Do you want to create it? [y/N] "
    if input(prompt).strip() == "y":
        os.mkdir(replay_dir)
    else:
        exit(1)

if not map_path.endswith(map_extension) or map_path.endswith(map_extension_text):
    for ext in (map_extension, map_extension_text):
        t = os.path.join(map_directory, map_path + ext)
        print(t)
        if os.path.isfile(t):
            map_path = t

if not os.path.isfile(map_path):
    print("Could not find any map named " + str(args.map) + ". Use --help to see a list of all available maps.\nExpected path: " + str(map_path))
    exit(1)


if args.mem <= 0:
    print("Max memory to use cannot be negative")
    exit(1)


def validate_player_dir(path, require_bat):
    if not os.path.exists(path):
        return "Cannot find the directory '" + path + "'. You should pass a relative or absolute path to a directory with your player code"

    if not os.path.isdir(path):
        return "'" + path + "' is not a directory. You should pass a relative or absolute path to a directory with your player code"

    if not os.path.exists(os.path.join(path, "run.sh")):
        return "Your player directory ('" + path + "') does not contain a run.sh file. See the example player folders to see how it should look."

    if require_bat and not os.path.exists(os.path.join(path, "run.bat")):
        return "Your player directory ('" + path + "') does not contain a run.bat file which is required when not using docker on Windows. See the example player folders to see how it should look."

    return None


require_run_bat = sys.platform == "win32" and not args.docker
err1 = validate_player_dir(args.player1, require_run_bat)
if err1 is not None:
    print(color_red + "Player 1: " + err1 + color_reset)
    exit(1)
err2 = validate_player_dir(args.player2, require_run_bat)
if err2 is not None:
    print(color_red + "Player 2: " + err2 + color_reset)
    exit(1)

initial_time = 1000000000 if args.unlimited_time else 10 * 1000
per_frame_time= 50

try:
    run_game(
        map_path,
        args.player1,
        args.player2,
        replay_dir,
        docker=args.docker,
        terminal_viewer=args.terminal_viewer,
        extra_delay=args.extra_delay,
        max_memory=args.mem,
        initial_time=initial_time,
        per_frame_time=per_frame_time,
        proxy_test=args.proxy_test
    )
except KeyboardInterrupt:
    print("Game Stopped")
    exit(0)
