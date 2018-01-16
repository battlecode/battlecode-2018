#!/bin/sh
# This is the script that runs INSIDE DOCKER to start the manager.
# It won't work outside docker.
echo "=== STARTING THE MANAGER ==="
echo "=== random garbage incoming: ==="
nohup dockerd-entrypoint.sh &
sleep 3
docker load -i /images/battlebaby.tar
export RUST_BACKTRACE=1
echo "=== random garbage complete! ==="

cd /battlecode/battlecode-manager
if [ -z "$SCRIMMAGE" ]; then
    python3 gui.py
else
    python3 scrimmage.py
fi
