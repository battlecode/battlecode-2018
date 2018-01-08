ls /

nohup dockerd-entrypoint.sh &

sleep 3

#docker load -i /sandbox
docker pull $SANDBOX


if [ $IMAGE_UPDATER ]
then
    python3 sandbox_update.py
else
    python3 battlecode_cli.py
fi
