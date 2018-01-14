# Runs inside the docker container and suspends / restarts the player.
# Speaks a very simple line protocol to player_sandbox.py

import socket
import os
import subprocess

key = os.environ['PLAYER_KEY']

def log(*args, **kwargs):
    #print("suspender", key, ':', *args, **kwargs)
    pass

log('connecting to /tmp/battlecode-suspender')
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect('/tmp/battlecode-suspender')
sockf = sock.makefile('rw', 64)
sockf.write(key + '\n')
sockf.flush()
for line in sockf:
    line = line.strip()
    log('received command', repr(line))
    if line == 'suspend':
        try:
            subprocess.check_output(
                # we use /bin/pkill from procps because default pkill doesn't have -U
                # the "player" user has uid 6147, see SandboxDockerfile
                ['/bin/pkill', '-U', '6147', '-STOP'],
                stderr=subprocess.STDOUT
            )
            e = os.system('')
            log('suspended')
            sockf.write('ack\n')
            sockf.flush()
        except Exception as e:
            log('failed suspension', e)
            sockf.write('failed '+str(e)+'\n')
            sockf.flush()
    elif line == 'resume':
        try:
            subprocess.check_output(
                ['/bin/pkill', '-U', '6147', '-CONT'],
                stderr=subprocess.STDOUT
            )
            log('resumed')
            sockf.write('ack\n')
            sockf.flush()
        except Exception as e:
            log('failed resumption', e)
            sockf.write('failed '+str(e)+'\n')
            sockf.flush()
    else:
        log('unknown command, skipping')
        sockf.write('failed\n')
        sockf.flush()

# no need to exit, our container will be killed by the manager

