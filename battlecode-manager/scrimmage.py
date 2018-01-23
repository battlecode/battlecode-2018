import psycopg2
import json
import os
import battlecode_cli as cli
import threading
import boto3
from time import sleep
import socket
import time
import nonsense
import random
import proxyuploader
import gzip
import string

pg = None
cur = None
BUSY = False
DB_LOCK = False
GAMES_RUN = []

s3 = boto3.resource('s3')
bucket = s3.Bucket(os.environ['BUCKET_NAME'])
key_prefix = 'tournament/' + os.environ['TOURNAMENT'] + '/' if 'TOURNAMENT' in os.environ else ''

def random_key(length):
    return ''.join([random.choice(string.ascii_letters + string.digits + string.digits) for _ in range(length)])

PROXY_UPLOADER = proxyuploader.ProxyUploader()

def end_game(data,winner,match_file,logs):
    global BUSY
    global DB_LOCK

    BUSY = False

    status = -1
    if winner == 'player1':
        status = 'redwon'
    elif winner == 'player2':
        status = 'bluewon'

    hidden_key = random_key(20)
    replay_key = key_prefix + 'replays/' + hidden_key + '.bc18z'
    red_log_key = key_prefix + 'logs/' + hidden_key + '_0.bc18log'
    blue_log_key = key_prefix + 'logs/' + hidden_key + '_1.bc18log'

    gzipped_replay = gzip.compress(json.dumps(match_file).encode())

    bucket.put_object(Key=replay_key,Body=gzipped_replay,ACL='public-read')
    bucket.put_object(Key=red_log_key,Body=json.dumps({'earth':logs[0],'mars':logs[2]}).encode(),ACL='public-read')
    bucket.put_object(Key=blue_log_key,Body=json.dumps({'earth':logs[1],'mars':logs[3]}).encode(),ACL='public-read')

    while DB_LOCK == True:
        sleep(0.1)
    DB_LOCK = True
    cur.execute("UPDATE " + os.environ["TABLE_NAME"] + " SET (status, replay, red_logs, blue_logs)=(%s,%s,%s,%s)  WHERE id=%s", (status,replay_key,red_log_key,blue_log_key,data['id']))
    pg.commit()
    DB_LOCK = False

    print("Finished game " + str(data['id']))

def match_thread(data):
    global BUSY
    global DB_LOCK
    BUSY = True
    GAMES_RUN.append(data['id'])

    data['s3_bucket'] = bucket

    data['player_memory'] = int(os.environ['PLAYER_MEMORY'])
    data['player_cpu'] = 20
    data['map_name'] = data['map']

    data['map'] = cli.get_map(os.path.abspath(os.path.join('..', 'battlecode-maps', data['map'])))
    data['docker'] = True
    data['terminal_viewer'] = False
    data['use_viewer'] = False

    data['extra_delay'] = 0

    try:
        (game, dockers, sock_file) = cli.create_scrimmage_game(data)
    except ValueError as e:
        print("Destroying the game, as it is invalid.  This should not happen.")
        while DB_LOCK == True:
            sleep(0.1)
        DB_LOCK = True
        cur.execute("UPDATE " + os.environ["TABLE_NAME"] + " SET status='rejected' WHERE id=%s", (data['id'],))
        pg.commit()
        DB_LOCK = False

        return


    PROXY_UPLOADER.game = game
    winner = None
    match_file = None
    try:
        print("Running match " + str(data['id']))
        winner, match_file = cli.run_game(game, dockers, data, sock_file,scrimmage=True)
    finally:
        cli.cleanup(dockers, data, sock_file)
    PROXY_UPLOADER.game = None
    PROXY_UPLOADER.games_run += 1

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

        cur.execute("SELECT (id, red_key, blue_key, map, red_team, blue_team) FROM " + os.environ["TABLE_NAME"] + " WHERE status='queued' or (status='running' and start < (NOW() - INTERVAL '8 min')) ORDER BY start ASC")

        row = cur.fetchone()

        if row is not None:
            if len(row) == 1:
                row = row[0][1:-1].split(",")
                row[0] = int(row[0])

            data = {'id':row[0],'red_key':row[1],'blue_key':row[2],'map':row[3]}

            if not BUSY:
                BUSY = True
                print('Running game ' + str(data))
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

if __name__ == "__main__":
    try:
        pg = psycopg2.connect("dbname='battlecode' user='battlecode' host='" + os.environ["DB_HOST"] + "' password='" + os.environ["DB_PASS"] + "'")
        cur = pg.cursor()
        print("Connected to postgres.")
    except:
        print("Could not connect to postgres.")

    poll_thread()
