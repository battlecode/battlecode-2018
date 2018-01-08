ls /

nohup dockerd-entrypoint.sh &

sleep 3

docker load -i /sandbox
docker pull gcr.io/battlecode18/sandbox

python3 battlecode_cli.py
