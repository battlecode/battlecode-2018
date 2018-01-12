import eel
import os
import battlecode_cli as cli
import threading
import sys
import json
import signal
import psutil
import player_plain

target_dir = os.path.abspath(os.path.dirname(__file__))
print('Moving into', target_dir)
os.chdir(target_dir)

options = {'host':'0.0.0.0', 'port':6147, 'mode':'default'}

if sys.platform == 'win32':
    options['host'] = 'localhost'

print('Starting eel')

eel.init('web')

game = None

def start_game(return_args):
    global WINNER
    WINNER = 0

    return_args['map'] = cli.get_map(os.path.abspath(os.path.join('..', 'battlecode-maps', return_args['map'])))
    if 'NODOCKER' in os.environ:
        return_args['docker'] = False
        return_args['dir_p1'] = os.path.abspath(os.path.join('..', return_args['dir_p1']))
        return_args['dir_p2'] = os.path.abspath(os.path.join('..', return_args['dir_p2']))
    else:
        return_args['docker'] = True
        return_args['dir_p1'] = os.path.abspath(os.path.join('/player', return_args['dir_p1']))
        return_args['dir_p2'] = os.path.abspath(os.path.join('/player', return_args['dir_p2']))
    return_args['terminal_viewer'] = False
    return_args['extra_delay'] = 0

    global game
    (game, dockers, sock_file) = cli.create_game(return_args)

    winner = None
    try:
        print("Running game...")
        winner = cli.run_game(game, dockers, return_args, sock_file)
    finally:
        cli.cleanup(dockers, return_args, sock_file)
    lock.release()

    if winner == 'player1':
        eel.trigger_end_game(1)()
    elif winner == ' player2':
        eel.trigger_end_game(2)()
    else:
        eel.trigger_end_game(0)()


@eel.expose
def get_viewer_data(turn):
    if game != None and len(game.manager_viewer_messages) >= 1:
        if turn >= len(game.manager_viewer_messages) or turn == -1:
            turn = len(game.manager_viewer_messages) - 1

        message = json.loads(game.manager_viewer_messages[turn])
        message['turn'] = turn
        return message
    else :
        return {'width':0, 'height': 0, 'earth' : [], 'mars': [], 'turn':0}

@eel.expose
def run_game(return_args):
    if not lock.acquire(blocking=False):
        return "Fail"

    t1 = threading.Thread(target=start_game,args=(return_args,))
    t1.start()
    return "success"

@eel.expose
def get_maps():
    if 'NODOCKER' in os.environ:
        map_dir = os.path.abspath('../battlecode-maps')
    else:
        map_dir = '/battlecode/battlecode-maps'

    maps = [o for o in os.listdir(map_dir)
                        if 'bc18map' in o]

    maps.append('testmap.bc18map')
    return maps

@eel.expose
def get_player_dirs():
    if 'NODOCKER' in os.environ:
        player_dir = os.path.abspath('..')
    else:
        player_dir = '/player'
    players = []
    for o in os.listdir(player_dir):
        if o.startswith('.') or o in ('battlecode', 'battlecode-manager'):
            continue
        full_path = os.path.join(player_dir, o)
        if not os.path.isdir(full_path):
            continue
        if os.path.exists(os.path.join(full_path, 'run.sh')):
            players.append(o)
    return players

# if 0 not ended, if 1 red, 2 blue
@eel.expose
def get_player_logs():
    if game != None:
        if all('logger' in player for player in game.players):
            logs = [player['logger'].logs.getvalue() for player in game.players]
            return logs
        else:
            return ["", "", "", ""]
    return ["NULL", "NULL", "NULL", "NULL"]

@eel.expose
def end_game():
    global game
    if game is not None:
        game.winner = 'player3'
        game.disconnected = True
        game.game_over = True
    return ""

def reap_children(timeout=3):
    "Tries hard to terminate and ultimately kill all the children of this process."
    def on_terminate(proc):
        print("process {} terminated with exit code {}".format(proc, proc.returncode))

    print("Killing manager children...")

    procs = psutil.Process().children(recursive=True)
    # send SIGTERM
    for p in procs:
        print("Killing ", p.pid)
        p.terminate()
    gone, alive = psutil.wait_procs(procs, timeout=timeout, callback=on_terminate)
    if alive:
        # send SIGKILL
        for p in alive:
            print("process {} survived SIGTERM; trying SIGKILL" % p.pid)
            p.kill()
        gone, alive = psutil.wait_procs(alive, timeout=timeout, callback=on_terminate)
        if alive:
            # give up
            for p in alive:
                print("process {} survived SIGKILL; giving up" % p.pid)

@eel.expose
def stop_manager():
    print("Shutting manager down.")
    player_plain.reap(psutil.Process())
    procs = psutil.Process().kill()


print("=== Ready! ===")
print("To play games open http://localhost:6147/run.html in your browser on Mac/Linux/WindowsPro, or http://192.168.99.100:6147/run.html on Windows10Home.")
lock = threading.Lock()

eel.start('run.html', options=options, block=False)

while True:
    eel.sleep(1.0)
