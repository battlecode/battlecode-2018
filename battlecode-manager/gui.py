import eel
import os
import battlecode_cli as cli

options = {'host':'0.0.0.0', 'port':6147, 'mode':'default'}

eel.init('web')
@eel.expose
def start_game(use_viewer, dir_p1, dir_p2, map_file):
    return_args = {}
    return_args['use_viewer'] = bool(use_viewer)
    return_args['dir_p1'] = os.path.abspath(dir_p1)+"/"
    return_args['dir_p2'] = os.path.abspath(dir_p2)+"/"
    return_args['map'] = cli.get_map(os.path.abspath(map_file))
    print(return_args)

    (game, dockers, sock_file) = cli.create_game(return_args)

    try:
        print("running game")
        cli.run_game(game, dockers, return_args, sock_file)
    finally:
        cli.cleanup(dockers, return_args, sock_file)


print("To play games open http://localhost:6147/play.html in your browser")
eel.start('main.html', options=options)
