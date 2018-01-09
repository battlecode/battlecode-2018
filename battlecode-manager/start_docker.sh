ls /

nohup dockerd-entrypoint.sh &

sleep 3

#docker load -i /sandbox
docker pull $SANDBOX


echo "Got Here?"
if [ $IMAGE_UPDATER ]
then
    python3 sandbox_update.py
else
    echo "Here?"
    ls
    python3 gui.py
    echo "Here?"
fi
