echo "=== STARTING THE MANAGER ==="
echo "=== random garbage incoming: ==="
nohup dockerd-entrypoint.sh &
sleep 3
docker load -i /images/battlebaby.tar
echo "=== random garbage complete! ==="
python3 gui.py