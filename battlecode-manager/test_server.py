#!/bin/python3
""" Creates simple UNIX Stream server for testing
"""
import socketserver
import argparse
import threading
import time
import os, docker
from tqdm import tqdm
from sandbox import Sandbox
from werkzeug.wrappers import Request, Response
from multiprocessing import Process

OUTPUT_FILE = None
NUM_CLIENTS = 0

class ReceiveHandler(socketserver.BaseRequestHandler):
    '''
    This class overrides the default handling method in socketServer, so it
    calls what we want
    '''

    def handle(self):
        '''
        This does all the processing of the data we receive and we spend our
        time in this function.
        '''
        # Wrap socket as file to get newline delimited data
        self._socket = self.request.makefile('rwb', 2)
        # Start timeing
        this_times = []
        # First time
        this_times.append(time.perf_counter())
        message = "Received initial"
        # Send reply
        self._socket.write(message.encode('utf-8'))
        # Wait for reply from client
        data = next(self._socket)

        this_times.append(time.perf_counter())
        # Process client reply
        client_times = data.decode().split(" ")
        this_times.append(float(str(client_times[0])))
        this_times.append(float(str(client_times[1])))
        print('Time diff:' + str(this_times[2] - this_times[0]))
        print('Time diff:' + str(this_times[1] - this_times[3]))
        for i in this_times:
            OUTPUT_FILE.write(str(i) + " ")
        OUTPUT_FILE.write('\n')
        global NUM_CLIENTS # pylint: disable=W0603
        OUTPUT_FILE.flush()
        NUM_CLIENTS -= 1

sandboxes = []

def main():
    Sandbox.initialize()

    socket_file = '/tmp/battlecode-socket'
    output_file = './output.txt'
    num_clients = 2

    try:
        os.unlink(socket_file)
    except OSError:
        if os.path.exists(socket_file):
            print("File exists at desired connection")
            raise
    server = socketserver.ThreadingUnixStreamServer(socket_file, ReceiveHandler)

    global OUTPUT_FILE # pylint: disable=W0603
    OUTPUT_FILE = open(output_file, 'w')

    global NUM_CLIENTS # pylint: disable=W0603
    NUM_CLIENTS = num_clients
    server_thread = threading.Thread(target=server.serve_forever, daemon=True)
    server_thread.start()

    global sandboxes
    for _ in tqdm(range(2)):
        sandboxes.append(Sandbox(socket_file,local_dir='test_bot'))

    for sandbox in sandboxes:
        sandbox.start()

    try:
        while NUM_CLIENTS > 0:
            time.sleep(0.5)
            continue
        print("Exiting Gracefully")
        server.server_close()
        for sandbox in sandboxes:
            sandbox.destroy()

    except KeyboardInterrupt:
        print("Exiting Gracefully")
        server.server_close()
        for sandbox in sandboxes:
            sandbox.destroy()

@Request.application
def application(request):
    # Either render the viewer if local, or alternatively return manager stats as json.
    return Response('Hello World!')

if __name__ == "__main__":
    from werkzeug.serving import run_simple

    status_thread = Process(target=run_simple, args=('0.0.0.0', 80, application))
    status_thread.start()

    main()
