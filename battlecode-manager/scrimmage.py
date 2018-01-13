from werkzeug.wrappers import Request, Response
import psycopg2
import json
import os
import battlecode_cli as cli


##### SCRIMMAGE SERVER 2K18 #######
##
##     We put a JSON post request to the scrimmage server.
##        {"password":"secret","red_key":"redawskey","blue_key":"blueawskey","map":"string","mapfile":{json of mapfile}}
##        Pass either map or mapfile, but not both
##
##     The scrimmage server returns the JSON {'game_id':random_game_id} to the POST request
##
##     If a game is currently not running, the scrimmage server creates an entry in the DB with the ID, red_key, blue_key,
##     map, mapfile, and current time, with a status field set to 1 (running).  The game is started.
##
##     If a game is currently running, an entry is once again created, but this time with status set to 0 (queued).
##
##     When the match is over, the scrimmage server sets that DB row status to 2 if player 1 wins, or 3 if player 2 wins,
##     and uploads the matchfile to s3.  The DB entry is updated with the matchfile key and logs for red/blue_earth/mars.
##
##     When not running matches, the scrimmage server polls the database for matches with status 1 started greater than N
##     seconds ago, or any matches with status 0 (queued), and runs them, resetting the start time.
##     N is the maximum time to run a match * 1.5.
##
##     At any time we can GET a status JSON {'games_run':[listofidseverrun],'busy':true/false}
##

pg = None
cur = None
BUSY = False
s3_bucket = None
GAMES_RUN = []


def random_key(length):
    return ''.join([random.choice(string.ascii_letters + string.digits + string.digits) for _ in range(length)])

def end_game(data,winner,match_file,logs):
    print(winner)
    print(logs)

def match_thread(data):
    BUSY = True
    GAMES_RUN.append(data['id'])

    data['s3_bucket'] = s3_bucket

    data['map'] = cli.get_map(os.path.abspath(os.path.join('..', 'battlecode-maps', data['map'])))
    data['docker'] = True
    data['terminal_viewer'] = False
    data['extra_delay'] = 0

    (game, dockers, sock_file) = cli.create_scrimmage_game(data)
    winner = None
    match_file = None
    try:
        print("Running game...")
        winner, match_file = cli.run_game(game, dockers, return_args, sock_file,scrimmage=True)
    finally:
        cli.cleanup(dockers, return_args, sock_file)

    logs = None
    if all('logger' in player for player in game.players):
        logs = [player['logger'].logs.getvalue() for player in game.players]

    end_game(data,winner,match_file,logs)

def run_match(data):
    t1 = threading.Thread(target=match_thread,args=(data,))
    t1.start()

    return t1

@Request.application
def application(request):
    if request.method == 'GET':
        return Response(json.dumps({'games_run':GAMES_RUN,'busy':BUSY}))
    elif request.method == 'POST':
        data = json.loads(request.data)

        if not 'password' in data:
            return Response(json.dumps({'error':'No password provided.'}),401)
        elif data['password'] != os.environ['PASSWORD']:
            return Response(json.dumps({'error':'Incorrect password.'}),401)

        if not ('red_key' in data and 'blue_key' in data and 'map' in data):
            return Response(json.dumps({'error':'Not all fields provided.'}),400)

        cur.execute("INSERT INTO " + os.environ["TABLE_NAME"] + " (red_key, blue_key, map, status, start, replay) VALUES (%s, %s, %s," + str(0 if BUSY else 1) + ", now(), '') RETURNING id", (data['red_key'],data['blue_key'],data['map']))

        pg.commit()
        game_id = cur.fetchone()[0]
        if not BUSY:
            data['id'] = game_id
            run_match(data)

        return Response(json.dumps({'game_id':game_id}))

if __name__ == "__main__":
    from werkzeug.serving import run_simple
    try:
        pg = psycopg2.connect("dbname='battlecode' user='battlecode' host='" + os.environ["DB_HOST"] + "' password='" + os.environ["DB_PASS"] + "'")
        cur = pg.cursor()
        print("Connected to postgres.")
    except:
        print("Could not connect to postgres.")

    run_simple('0.0.0.0', 410, application, use_reloader=True)
