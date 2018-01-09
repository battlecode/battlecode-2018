print('atest')
import os
import signal
import time

def sig_handler(signum, frame):
    print("segfault")
signal.signal(signal.SIGSEGV, sig_handler)
def sig_handler(signum, frame):
    print("sigill")
signal.signal(signal.SIGILL, sig_handler)

time.sleep(2)
import battlecode as bc
time.sleep(2)

print('btest')
time.sleep(2)
for controller in bc.game_turns():
    time.sleep(2)
    print('test')
    print(controller.round)

time.sleep(2)
print('afinitio')
