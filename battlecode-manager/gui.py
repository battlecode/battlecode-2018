import eel
import os
import battlecode_cli as cli
import threading
import sys
import json

options = {'host':'0.0.0.0', 'port':6147, 'mode':'default'}

eel.init('web')

game = None

def start_game(return_args):
    global WINNER
    WINNER = 0

    return_args['map'] = cli.get_map(return_args['map'])
    return_args['dir_p2'] = '/player/' + return_args['dir_p2']
    return_args['dir_p1'] = '/player/' + return_args['dir_p1']

    global game
    (game, dockers, sock_file) = cli.create_game(return_args)

    winner = None
    try:
        print("running game")
        winner  = cli.run_game(game, dockers, return_args, sock_file)
    finally:
        cli.cleanup(dockers, return_args, sock_file)
    lock.release()
    print("release lock")

    if winner == 'player1':
        eel.trigger_end_game(1)()
    elif winner == ' player2':
        eel.trigger_end_game(2)()
    else:
        eel.trigger_end_game(0)()


@eel.expose
def get_viewer_data():
    if game != None:
        return json.loads(game.manager.manager_viewer_message())
    else:
        return {'width':0, 'height': 0, 'earth' : [], 'mars': []}


@eel.expose
def run_game(return_args):
    if not lock.acquire(blocking=False):
        return "Fail"

    t1 = threading.Thread(target=start_game,args=(return_args,))
    t1.start()
    return "success"

@eel.expose
def get_maps():
    player_dir = '/battlecode/battlecode-maps'
    maps = [o for o in os.listdir(player_dir)
                        if 'bc18map' in o]
    maps.extend([o for o in os.listdir('/player')
                        if 'bc18map' in o])

    maps.append('testmap.bc18map')
    return maps

@eel.expose
def get_player_dirs():

    player_dir = '/player'
    return [o for o in os.listdir(player_dir)
                if os.path.isdir(os.path.join(player_dir,o))]

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

@eel.expose
def stop_manager():
    os.system('killall python3')
    sys.exit(0)

print("To play games open http://localhost:6147/run.html in your browser on Mac/Linux/WindowsPro, or http://192.168.99.100:6147/run.html on Windows10Home.")
lock = threading.Lock()
eel.start('run.html', options=options, block=False)

while True:
    eel.sleep(1.0)
