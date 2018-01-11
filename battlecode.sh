#!/bin/bash
mtput() {
    if command -v tput > /dev/null; then
        tput $@
    fi
}

# Use tput to show different colors in the terminal
mtput setaf 5
echo "$ pip3 install --user cffi eel tqdm werkzeug ujson psutil"
mtput sgr0
pip3 install -q --user cffi eel tqdm werkzeug ujson psutil

RESULT=$?
if [ $RESULT -ne 0 ]; then
    echo "Warning: pip install failed!"
    echo "I'll keep going, but maybe try to fix whatever error you just got."
fi
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
export PYTHONPATH="$DIR/bindings/python/:$PYTHONPATH"
python3 $DIR/battlecode-manager/simple_cli.py "$@"