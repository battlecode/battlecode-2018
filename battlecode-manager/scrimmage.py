from werkzeug.wrappers import Request, Response
import psycopg2
import json
import os
import battlecode_cli as cli
import threading
import boto3
from time import sleep
import socket
import time

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
DB_LOCK = False
GAMES_RUN = []

s3 = boto3.resource('s3')
bucket = s3.Bucket(os.environ['BUCKET_NAME'])

def random_key(length):
    return ''.join([random.choice(string.ascii_letters + string.digits + string.digits) for _ in range(length)])

class ProxyUploader():
    def __init__(self):
        if 'SCRIMMAGE_PROXY_URL' in os.environ and 'SCRIMMAGE_PROXY_SECRET' in os.environ:
            self.url = os.environ['SCRIMMAGE_PROXY_URL']
            self.secret = os.environ['SCRIMMAGE_PROXY_SECRET']
            self.thread = threading.Thread(self.run_forever, args=())
            self.thread.start()
        if 'SCRIMMAGE_UPDATE_EVERY' in os.environ:
            self.update_every = os.environ['SCRIMMAGE_UPDATE_EVERY']
        else:
            self.update_every = 1
        self.red_id = 0
        self.blue_id = 0
        self.red_name = 0
        self.blue_name = 0
        self.game_id = 0
        self.game = None
        self.id = random_key(20)
        self.start = time.time()
        self.games_run = 0

    def run_forever(self):
        import json
        while True:
            try:
                self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
                self.socket.settimeout(30)
                self.socket.connect(self.url, 56147)
                self.f = self.socket.makefile()
                while True:
                    msg = {
                        "id": self.id,
                        "secret": self.secret,
                        "uptime": int((time.time() - self.start) * 1000),
                        "games_run": self.games_run
                    }
                    if self.game is not None:
                        game = self.game.state_report()
                        game['id'] = self.game_id
                        game['red']['id'] = self.red_id
                        game['blue']['id'] = self.blue_id
                        msg['game'] = game

                    self.f.write(json.dumps(msg))
                    self.f.flush()
                    m = next(self.f)
                    assert m.strip() == 'ok', 'wrong resp: {}'.format(m.strip())
                    time.sleep(self.update_every)
            except Exception as e:
                print('some sort of failure', e)
            time.sleep(30)

UPLOADER = ProxyUploader()

def end_game(data,winner,match_file,logs):
    global BUSY
    global DB_LOCK

    BUSY = False

    status = -1
    if winner == 'player1':
        status = 'redwon'
    elif winner == 'player2':
        status = 'bluewon'

    replay_key = 'replays/' + str(data['id']) + '.bc18';
    replay = s3.Object(os.environ['BUCKET_NAME'], replay_key)
    replay.put(Body=json.dumps(match_file).encode())

    red_log_key = 'logs/' + str(data['id']) + '_0.bc18log'
    blue_log_key = 'logs/' + str(data['id']) + '_1.bc18log'

    red_log = s3.Object(os.environ['BUCKET_NAME'], red_log_key)
    red_log.put(Body=json.dumps({'earth':logs[0],'mars':logs[2]}).encode())

    blue_log = s3.Object(os.environ['BUCKET_NAME'], blue_log_key)
    blue_log.put(Body=json.dumps({'earth':logs[1],'mars':logs[3]}).encode())

    while DB_LOCK == True:
        sleep(0.1)
    DB_LOCK = True
    cur.execute("UPDATE " + os.environ["TABLE_NAME"] + " SET (status, replay, red_logs, blue_logs)=(%s,%s,%s,%s)  WHERE id=%s", (status,replay_key,red_log_key,blue_log_key,data['id']))
    pg.commit()
    DB_LOCK = False

    print("Finsihed game " + str(data['id']))

def match_thread(data):
    global BUSY
    BUSY = True
    GAMES_RUN.append(data['id'])

    data['s3_bucket'] = bucket

    data['player_memory'] = 256
    data['player_cpu'] = 20
    data['map_name'] = data['map']

    data['map'] = cli.get_map(os.path.abspath(os.path.join('..', 'battlecode-maps', data['map'])))
    data['docker'] = True
    data['terminal_viewer'] = False
    data['use_viewer'] = False

    data['extra_delay'] = 0

    (game, dockers, sock_file) = cli.create_scrimmage_game(data)
    winner = None
    match_file = None
    try:
        print("Running match " + str(data['id']))
        winner, match_file = cli.run_game(game, dockers, data, sock_file,scrimmage=True)
    finally:
        cli.cleanup(dockers, data, sock_file)

    logs = None
    if all('logger' in player for player in game.players):
        logs = [player['logger'].logs.getvalue() for player in game.players]

    end_game(data,winner,match_file,logs)

def run_match(data):
    t1 = threading.Thread(target=match_thread,args=(data,))
    t1.start()

    return t1

def poll_thread():
    global DB_LOCK
    global BUSY

    while True:
        sleep(1)
        if BUSY:
            continue

        while DB_LOCK == True:
            sleep(0.1)
        DB_LOCK = True

        cur.execute("SELECT (id, red_key, blue_key, map, red_team, blue_team) FROM " + os.environ["TABLE_NAME"] + " WHERE status='queued' or (status='running' and start < (NOW() - INTERVAL '5 min')) ORDER BY start ASC")

        row = cur.fetchone()

        if row is not None:
            if len(row) == 1:
                row = row[0][1:-1].split(",")
                row[0] = int(row[0])

            data = {'id':row[0],'red_key':row[1],'blue_key':row[2],'map':row[3]}

            if not BUSY:
                BUSY = True
                cur.execute("UPDATE " + os.environ['TABLE_NAME'] + " SET status='running', start=NOW() WHERE id=%s",(data['id'],))
                pg.commit()

                try:
                    PROXY_UPLOADER.game_id = row[0]
                    PROXY_UPLOADER.red_id = row[4]
                    PROXY_UPLOADER.blue_id = row[5]
                except Exception as e:
                    print("error setting team data:", e)

                run_match(data)

        DB_LOCK = False

@Request.application
def application(request):
    global DB_LOCK
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

        while DB_LOCK == True:
            sleep(0.1)
        DB_LOCK = True
        cur.execute("INSERT INTO " + os.environ["TABLE_NAME"] + " (red_key, blue_key, map, status, start) VALUES (%s, %s, %s," + ('queued' if BUSY else 'running') + ", now()) RETURNING id", (data['red_key'],data['blue_key'],data['map']))

        pg.commit()
        game_id = cur.fetchone()[0]
        DB_LOCK = False
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

    threading.Thread(target=poll_thread).start()
    run_simple('0.0.0.0', 410, application, use_reloader=True)
