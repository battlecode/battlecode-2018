'''
This runs stuff
'''
import os
import time
import docker
import packaging
import packaging.version
import packaging.specifiers
import packaging.requirements
import atexit

stuff=None

def start_docker(players):
    '''
    Start the docker file
    '''
    docker_client = docker.from_env()
    volumes = {str(players):{'bind':'/player', 'mode':'rw'}}
    ports = {6147:6147, 16147:16147}

    try:
        global stuff
        stuff = docker_client.containers.run('battlecode/battlecode-2018', privileged=True,
                                     detach=True, stdout=True, stderr=True,
                                     tty=True, stdin_open=True,
                                     volumes=volumes, ports=ports)
    except ConnectionError as e:
        print("Please run this as sudo")
    except Exception as e:
        if stuff != None:
            stuff.remove(force=True)
        print("There was an error " + str(e))




def exit_handler():
    stuff.remove(force=True)
    print("Exiting nicely...")

atexit.register(exit_handler)

try:
    start_docker(os.getcwd())
    time.sleep(15)
    print("Docker running at https://localhost:6147/run.html on Mac/Linux/WindowsPro, or http://192.168.99.100:6147/run.html on Windows10Home.")
    while True:
        time.sleep(1)
except KeyboardInterrupt:
    if stuff != None:
        print(stuff)
        stuff.remove(force=True)
    print("Killing")
