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
##     The scrimmage server returns the JSON {'game_id':random_game_id} to the POST request if it is not
##     running a game, or {'error':'busy'} if it is.  If it is busy, it does not do the following.
##
##     The scrimmage server creates an entry in the DB with the ID, red_key, blue_key, map, mapfile,
##     and current time, with a status field set to 0 (running).
##
##     When the match is over, the scrimmage server sets that DB row status to 1 if player 1 wins, or 2 if player 2 wins.
##
##     When not running matches, the scrimmage server polls the database for matches started greater than N seconds ago, and runs them,
##     resetting the start time.  N is the maximum time to run a match * 1.5.
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

def run_match(data):
    data['s3_bucket'] = s3_bucket
    (game, dockers, sock_file) = cli.create_scrimmage_game(data)

    pass

@Request.application
def application(request):
    if request.method == 'GET':
        return Response('Hello World!')
    elif request.method == 'POST':
        data = json.loads(request.data)

        if not 'password' in data:
            return Response(json.dumps({'error':'No password provided.'}),401)
        elif data['password'] != os.environ['PASSWORD']:
            return Response(json.dumps({'error':'Incorrect password.'}),401)

        if not ('red_key' in data and 'blue_key' in data and 'map' in data):
            return Response(json.dumps({'error':'Not all fields provided.'}),400)


        if not BUSY:
            cur.execute("INSERT INTO " + os.environ["TABLE_NAME"] + " (red_key, blue_key, map, start, replay) VALUES (%s, %s, %s, now(), '') RETURNING id", (data['red_key'],data['blue_key'],data['map']))
            pg.commit()

            GAMES_RUN.append(cur.fetchone()[0])
            BUSY = True
            run_match(data)

        return Response('Goodbye World!')

if __name__ == "__main__":
    from werkzeug.serving import run_simple
    try:
        pg = psycopg2.connect("dbname='battlecode' user='battlecode' host='" + os.environ["DB_HOST"] + "' password='" + os.environ["DB_PASS"] + "'")
        cur = pg.cursor()
        print("Connected to postgres.")
    except:
        print("Could not connect to postgres.")

    run_simple('0.0.0.0', 410, application, use_reloader=True)
