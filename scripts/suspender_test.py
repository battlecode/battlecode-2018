# test the suspender.py script.

import socket
import os

key = 12345

def log(*args, **kwargs):
    print("t:", *args, **kwargs)
    # pass

log('starting on to /tmp/battlecode-suspender')
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
os.system('rm /tmp/battlecode-suspender')
sock.bind('/tmp/battlecode-suspender')

# Bind the socket to the port

# Listen for incoming connections
sock.listen(1)

while True:
    # Wait for a connection
    log('waiting')
    connection, client_address = sock.accept()
    try:
        log('connection from', client_address)

        f = connection.makefile('rw', 64)

        def do(s):
            f.write(s + '\n')
            f.flush()
            print(next(f).strip())

        login = next(f).strip()

        do('suspend')
        do('resume')
        do('resume')
        do('broken')
        do('suspend')
        do('resume')
        do('suspend')
        do('suspend')
        do('suspend')
        

    finally:
        print('done')
        # Clean up the connection
        connection.close()
