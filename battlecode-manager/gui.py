import eel
import os
import battlecode_cli as cli
import threading

options = {'host':'0.0.0.0', 'port':6147, 'mode':'default'}

eel.init('web')
@eel.expose
def run_game(return_args):
    lock.acquire()
    return_args['map'] = cli.get_map(return_args['map'])
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
    return winner



@eel.expose
def get_maps():
    player_dir = '/player'
    return [o for o in os.listdir(player_dir)
                        if o.contains('bc18map')]


@eel.expose
def get_player_dirs():

    player_dir = '/player'
    return [o for o in os.listdir(player_dir)
                if os.path.isdir(os.path.join(player_dir,o))]

@eel.expose
def get_player_logs():
    if game != None:
        return [player['logger'].logs.getvalue() for player in game.players]
    return ["NULL", "NULL", "NULL", "NULL"]

def end_game():
    return ""

print("To play games open http://localhost:6147/run.html in your browser")
lock = threading.Lock()
eel.start('run.html', options=options)
