from trueskill import Rating, quality_1vs1, rate_1vs1
import psycopg2
import json
from time import sleep
import threading
import random
import os
from math import exp

pg = psycopg2.connect("dbname='battlecode' user='battlecode' host='" + os.environ["DB_HOST"] + "' password='" + os.environ["DB_PASS"] + "'")
cur = pg.cursor()
print("Connected to postgres.")

MATCH_PERIOD = 0.75
QUEUED_MATCHES = {}
DB_LOCK = False
QUEUE_RANGE = 50
MAPS = ['socket.bc18map','bananas.bc18t','julia.bc18t']

def update_loop():
    global DB_LOCK
    while True:
        sleep(1)
        if len(QUEUED_MATCHES.keys()) == 0:
            continue

        while DB_LOCK:
            sleep(0.1)
        DB_LOCK = True
        keys = [int(key) for key in QUEUED_MATCHES.keys()]

        cur.execute("SELECT (id, status) FROM match_kube WHERE (status='redwon' or status='bluewon') and id=ANY(%s)", (keys,))
        games = cur.fetchall()
        DB_LOCK = False

        for game in games:
            game = game[0][1:-1].split(",")
            game[0] = int(game[0])

            print("Updating game " + str(game[0]) + ".")

            red_player = QUEUED_MATCHES[game[0]]['red']
            blue_player = QUEUED_MATCHES[game[0]]['blue']

            rs = Rating(mu=red_player['mu'],sigma=red_player['sigma'])
            bs = Rating(mu=blue_player['mu'],sigma=blue_player['sigma'])

            rn, bn = rate_1vs1(rs, bs) if game[1] == 'redwon' else rate_1vs1(bs, rs)

            print("New red mu: " + str(rn.mu) + ", sigma: " + str(rn.sigma))
            print("New blue mu: " + str(bn.mu) + ", sigma: " + str(bn.sigma) + "\n")

            while DB_LOCK:
                sleep(0.1)
            DB_LOCK = True
            cur.execute("UPDATE battlecode_teams SET (mu,sigma)=(%s,%s) WHERE id=%s",(rn.mu,rn.sigma,red_player['id']))
            cur.execute("UPDATE battlecode_teams SET (mu,sigma)=(%s,%s) WHERE id=%s",(bn.mu,bn.sigma,blue_player['id']))
            pg.commit()
            DB_LOCK = False

            del QUEUED_MATCHES[game[0]]

def softmax(x, T=1):
    s = sum([exp(n/T) for n in x])
    return [exp(n/T)/s for n in x]

threading.Thread(target=update_loop).start()

while True:
    sleep(1)
    while DB_LOCK:
        sleep(0.1)
    DB_LOCK = True

    if len(QUEUED_MATCHES.keys()) > 500:
        continue

    cur.execute("SELECT (battlecode_teams.id, battlecode_teams.mu, battlecode_teams.sigma, s2.source_code) FROM battlecode_teams INNER JOIN (SELECT source_code, team FROM scrimmage_submissions WHERE id IN (SELECT MAX(id) from scrimmage_submissions GROUP BY team)) as s2 ON battlecode_teams.id = s2.team ORDER BY battlecode_teams.mu DESC")
    users = cur.fetchall()

    DB_LOCK = False

    matches = []
    for index, user in enumerate(users):
        upper_bound = index-QUEUE_RANGE if index > QUEUE_RANGE else 0
        lower_bound = index+QUEUE_RANGE if index < len(users)-QUEUE_RANGE else len(users) #not inclusive

        options = users[upper_bound:lower_bound]
        qualities = []
        softmaxes = []
        user = user[0][1:-1].split(",")
        user[0] = int(user[0])
        user[1] = float(user[1])
        user[2] = float(user[2])
        for option in options:
            option = option[0][1:-1].split(",")
            option[0] = int(option[0])
            option[1] = float(option[1])
            option[2] = float(option[2])
            if option != user:
                qualities.append({'red':{'id':user[0],'mu':user[1],'sigma':user[2],'s3':user[3]},
                                  'blue':{'id':option[0],'mu':option[1],'sigma':option[2],'s3':option[3]}})
                qualities[-1]['qual'] = quality_1vs1(Rating(mu=user[1],sigma=user[2]),
                                                     Rating(mu=option[1],sigma=option[2]))
                softmaxes.append(qualities[-1]['qual'])

        if len(softmaxes) == 0:
            print("Not enough users to queue matches.")
            continue

        softmaxes = softmax(softmaxes)
        sample, action = random.random(), -1
        while sample >= 0:
            action += 1
            sample -= softmaxes[action]
        matches.append(qualities[action])

    print("Queuing " + str(len(matches)) + " matches.")
    while DB_LOCK:
        sleep(0.1)
    DB_LOCK = True
    for match in matches:
        random_map = random.choice(MAPS)
        cur.execute("INSERT INTO match_kube (red_key, blue_key, map, status, red_team, blue_team) VALUES (%s,%s,%s,'queued',%s,%s) RETURNING id",(match['red']['s3'],match['blue']['s3'],random_map,match['red']['id'],match['blue']['id']))
        pg.commit()
        QUEUED_MATCHES[cur.fetchone()[0]] = match
    DB_LOCK = False

    sleep(int(MATCH_PERIOD*60))
