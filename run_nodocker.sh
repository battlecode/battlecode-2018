#!/bin/sh
echo "=== STARTING THE MANAGER (no docker) ==="
echo "=== ensuring dependencies ==="
echo "$ pip3 install --user cffi eel tqdm werkzeug ujson psutil"
pip3 install --user cffi eel tqdm werkzeug ujson psutil
RESULT=$?
if [ $RESULT -ne 0 ]; then
    echo "Warning: pip install failed!"
    echo "I'll keep going, but maybe try to fix whatever error you just got."
fi
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd $DIR/battlecode-manager
export PYTHONPATH="../bindings/python/:$PYTHONPATH"
export NODOCKER=1
python3 gui.py