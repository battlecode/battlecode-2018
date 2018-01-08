print('atest')
import os
import signal

def sig_handler(signum, frame):
    print("segfault")
signal.signal(signal.SIGSEGV, sig_handler)
def sig_handler(signum, frame):
    print("sigill")
signal.signal(signal.SIGILL, sig_handler)

import battlecode as bc

print('btest')
for controller in bc.game_turns():
    print('test')
    print(controller.round)

print('afinitio')
