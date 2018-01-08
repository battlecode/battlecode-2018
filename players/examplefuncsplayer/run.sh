#!/bin/sh
echo 'running'
ls
pwd
ls -alt /tmp
python3 run.py
EXIT=$?
echo exit code $EXIT
echo 'finito'
