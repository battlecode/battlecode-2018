import eel
import os
import battlecode_cli as cli
import threading
import sys
import json
import signal
import psutil
import player_plain
import battlecode as bc
import zipfile
import requests
import base64
import shutil


target_dir = os.path.abspath(os.path.dirname(__file__))
print('Moving into', target_dir)
os.chdir(target_dir)

options = {'host':'0.0.0.0', 'port':6147, 'mode':'default'}

if sys.platform == 'win32':
    options['host'] = 'localhost'

print('Starting eel')

eel.init('web')

CLIENT_ID = 'YmF0dGxlY29kZXdlYmFwcDpKQlVZOVZFNjkyNDNCWUM5MDI0Mzg3SEdWWTNBUUZL'
game = None


def get_token(username, password):
    headers = {}
    headers['authorization'] = "Basic " + CLIENT_ID
    data = {}
    data['grant_type'] = 'password'
    data['username'] = username
    data['password'] = password
    data['client_id'] = CLIENT_ID
    req = requests.post("http://www.battlecode.org/oauth/token", headers=headers, data=data)
    print(req.text)
    return req


@eel.expose
def upload_scrim_server(return_args):
    cwd = os.getcwd()
    if 'NODOCKER' in os.environ:
        os.chdir('..')
    else:
        os.chdir('/player')
    os.chdir(return_args['file_name'])
    zip_file_name = os.path.abspath(os.path.join('../',
        return_args['file_name']))

    shutil.make_archive(zip_file_name, 'zip', '.')
    if not zip_file_name.endswith('.zip'):
        zip_file_name += '.zip'

    os.chdir(cwd)
    username = return_args['username']
    password = return_args['password']
    req = get_token(username, password)
    if req.status_code != 200:
        print("Error authenticating.")
        return "Error authenticating."

    token = json.loads(req.text)['access_token']
    headers = {}
    headers['Authorization'] = 'Bearer ' + token
    data = {}
    data['label'] = return_args['player']
    with open(zip_file_name, 'rb') as image_file:
        encoded_string = base64.b64encode(image_file.read())
    data['src'] = encoded_string
    res =  requests.post("https://battlecode.org/apis/submissions", headers=headers, data=data)
    return "success"



@eel.expose
def save_logs(file_name):
    if 'NODOCKER':
        file_name = os.path.abspath(os.path.join('..', file_name))
    else:
        file_name = os.path.abspath(os.path.join('/player/', file_name))

    output_string = ""
    if game != None:
        if all('logger' in player for player in game.players):
            for i in range(len(game.players)):
                player = game.players[i]
                log_header = "\n\n\n\n\n\n======================================\n"
                if i % 2 == 0:
                    log_header += "Red "
                else:
                    log_header += "Blue "
                if i < 2:
                    log_header += "Earth"
                else:
                    log_header += "Mars"
                log_header += "\n\n"
                logs = log_header + player['logger'].logs.getvalue()
                output_string += logs
    else:
        # This should never run. Game needs to be started to call this modal
        return ""

    try:
        with open(file_name, 'w') as f:
            f.write(output_string)
        return ""

    except Exception as e:
        print("There was an error dumping the logs")
        print(e)
        return str(e)

def start_game(return_args):
    global WINNER
    WINNER = 0
    return_args['map_name'] = return_args['map']

    # check mountpoint for maps first
    c2 = os.path.abspath(os.path.join('/player/battlecode-maps', return_args['map']))
    if 'NODOCKER' not in os.environ and os.path.exists(c2):
        return_args['map'] = cli.get_map(c2)
    else:
        c1 = os.path.abspath(os.path.join('..', 'battlecode-maps', return_args['map']))
        if os.path.exists(c1):
            return_args['map'] = cli.get_map(c1)
        else:
            if 'testmap' not in return_args['map']:
                print("Can't find map {} in {}, falling back to test map..",
                    return_args['map'],
                    os.path.abspath(os.path.join('..', 'battlecode-maps'))
                )
            if 'NODOCKER' not in os.environ:
                print('(Also looked in /player/battlecode-maps, which should be mounted to the battlecode-maps directory of your scaffold)')
            return_args['map'] = bc.GameMap.test_map()

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

    print("Ready to run next game.")


@eel.expose
def get_viewer_data(turn):
    turn = int(turn)
    if game != None and len(game.manager_viewer_messages) >= 1:
        if turn >= len(game.manager_viewer_messages) or turn == -1:
            turn = len(game.manager_viewer_messages) - 1

        message = json.loads(game.manager_viewer_messages[turn])
        message['turn'] = turn
        return message
    else:
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
                        if 'bc18map' in o or 'bc18t' in o]

    maps.append('testmap.bc18map')
    if 'NODOCKER' not in os.environ:
        try:
            for o in os.listdir('/player/battlecode-maps'):
                if o not in maps:
                    maps.append(o)
        except:
            pass

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
        pass
        # print("process {} terminated with exit code {}".format(proc, proc.returncode))

    procs = psutil.Process().children(recursive=True)
    # send SIGTERM
    for p in procs:
        p.terminate()
    gone, alive = psutil.wait_procs(procs, timeout=timeout, callback=on_terminate)
    if alive:
        # send SIGKILL
        for p in alive:
            # print("process {} survived SIGTERM; trying SIGKILL" % p.pid)
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
