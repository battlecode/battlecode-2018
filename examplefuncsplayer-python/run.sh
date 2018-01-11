#!/bin/sh
export "PYTHONPATH=../battlecode/python:$PYTHONPATH"
python3 run.py

# If you set the following flag, the engine won't run type asserts.
# It may be slightly faster but you'll get more confusing error messages.
# Maybe leave it in place until the tournament.
#python3 -O run.py
