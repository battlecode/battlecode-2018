'''
This runs stuff
'''
import os
import docker
import packaging
import packaging.version
import packaging.specifiers
import packaging.requirements

def start_docker(players):
    '''
    Start the docker file
    '''
    docker_client = docker.from_env()
    volumes = {str(players):{'bind':'/player', 'mode':'rw'}}
    ports = {6147:6147, 16147:16147}

    command = "sh start_docker.sh"
    try:
        docker_client.containers.run('battlecode/battlecode-2018', privileged=True,
                                     detach=False, stdout=True, stderr=True,
                                     tty=True, stdin_open=True,
                                     volumes=volumes, ports=ports)
    except ConnectionError as e:
        print("Please run this as sudo")
    except Exception as e:
        print("There was an error " + str(e))

    print("done")


start_docker(os.getcwd())
