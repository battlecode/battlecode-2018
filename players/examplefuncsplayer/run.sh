#!/bin/sh
echo 'running'
pip3 install pympler
ls
pwd
ls -alt /tmp
python3 run.py
EXIT=$?
echo exit code $EXIT
echo 'finito'
