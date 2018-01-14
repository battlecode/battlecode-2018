#!/bin/sh
# this script runs in docker to start up players.
# it is placed in the root directory.

# player code will be mounted at /code.
# A "player" user already exists, we just have to chown.
chown -R player:player /code
# The above may have no effect in Docker for Windows (https://github.com/docker/for-win/issues/39)
# so, we'll chmod for good measure.
chmod -R a+rw /code

# additionally, let players access the messaging socket
chmod a+rw /tmp/battlecode-socket

# start the player suspender script
python3 /suspender.py &

# now run their script!
cd /code && su player -s /bin/sh -m -c 'sh run.sh'
