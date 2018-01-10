from werkzeug.wrappers import Request, Response
import psycopg2
import json
import os

##### SCRIMMAGE SERVER 2K18 #######
##
##     We put a JSON post request to the scrimmage server.
##        {"password":"secret","red_key":"redawskey","blue_key":"blueawskey","map":"string","mapfile":{json of mapfile}}
##        Pass either map or mapfile, but not both
##
##     The scrimmage server generates a string game ID, and creates an entry in the DB
##     with the ID, red_key, blue_key, map, mapfile, and current time, with a status field set to 0 (running).
##
##     The scrimmage server returns the JSON {'game_id':game_id} to the POST request.
##
##     When the match is over, the scrimmage server sets that DB row status to 1 if player 1 wins, or 2 if player 2 wins.
##
##     Every so often, the scrimmage server polls the database for matches started greater than N seconds ago, and runs them,
##     resetting the start time.  N is the maximum time to run a match * 1.5.
##


def random_key(length):
    return ''.join([random.choice(string.ascii_letters + string.digits + string.digits) for _ in range(length)])

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

        if not ('red_key' in data and 'blue_key' in data and ('map' in data or 'mapfile' in data)):
            return Response(json.dumps({'error':'Not all fields provided.'}),400)



        return Response('Goodbye World!')

if __name__ == "__main__":
    from werkzeug.serving import run_simple
    run_simple('0.0.0.0', 410, application, use_reloader=True)
