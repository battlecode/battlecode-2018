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
    volumes = {str(players):{'bind':'/player1', 'mode':'ro'}}

    command = "sh start_docker.sh"
    env = {'SANDBOX':'gcr.io/battlecode18/sandbox'}
    try:
        docker_client.containers.run('battledaddy', command, privileged=True,
                                     detach=False, stdout=True, stderr=True,
                                     volumes=volumes, environment=env)
    except ConnectionError as e:
        print("Please run this as sudo")
    except Exception as e:
        print("There was an error " + str(e))

    print("done")



start_docker(os.getcwd())
