#!/bin/sh
export SANDBOX='gcr.io/battlecode18/sandbox'
export PLAYER_MEM_LIMIT='256m'
export PLAYER_CPU_PERCENT=20
export P1='../players/examplefuncsplayer'
export VIEWER=False
export MAP='default'
export P2='../players/examplefuncsplayer' 
docker pull gcr.io/battlecode18/sandbox
python3 gui.py
