#!/bin/bash
mtput() {
    if command -v tput > /dev/null; then
        tput $@
    fi
}

echo "=== STARTING THE MANAGER (no docker) ==="
echo "=== ensuring dependencies ==="
mtput setaf 5
echo "$ pip3 install --user cffi eel tqdm werkzeug ujson psutil"
mtput sgr0
pip3 install --user cffi eel tqdm werkzeug ujson psutil
RESULT=$?
if [ $RESULT -ne 0 ]; then
    echo "Warning: pip install failed!"
    echo "I'll keep going, but maybe try to fix whatever error you just got."
fi
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
export PYTHONPATH="$DIR/battlecode/python:$PYTHONPATH"
export NODOCKER=1
mtput setaf 5
echo "$ python3 $DIR/battlecode-manager/gui.py"
mtput sgr0
python3 $DIR/battlecode-manager/gui.py