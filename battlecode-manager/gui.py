import eel
import os
import battlecode_cli as cli
import threading

options = {'host':'0.0.0.0', 'port':6147, 'mode':'default'}

eel.init('web')

WINNER = 0
game = None

def start_game(return_args):
    global WINNER
    WINNER = 0

    return_args['map'] = cli.get_map(return_args['map'])
    return_args['dir_p2'] = '/player/' + return_args['dir_p2']
    return_args['dir_p1'] = '/player/' + return_args['dir_p1']
    print(return_args)

    global game
    (game, dockers, sock_file) = cli.create_game(return_args)
    print(sock_file)

    try:
        print("running game")
        winner  = cli.run_game(game, dockers, return_args, sock_file)
    finally:
        cli.cleanup(dockers, return_args, sock_file)
    lock.release()

    WINNER = 1 if winner == 'player1' else 2

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
    maps.append('test_map')
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
            logs.append(0)
            return logs
        else:
            return ["", "", "", "",0]
    return ["NULL", "NULL", "NULL", "NULL",0]

@eel.expose
def end_game():
    return ""

print("To play games open http://localhost:6147/run.html in your browser")
lock = threading.Lock()
eel.start('run.html', options=options,block=False)

while True:
    eel.sleep(1.0)
