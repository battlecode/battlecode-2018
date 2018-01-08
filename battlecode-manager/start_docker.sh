nohup dockerd-entrypoint.sh &

sleep 3

docker pull $SANDBOX
python3 battlecode_cli.py
