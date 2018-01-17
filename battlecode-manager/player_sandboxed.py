import threading
from threading import Timer

from player_abstract import AbstractPlayer

import random
import socket
import server

def _stream_logs(container, stdout, stderr, line_action):
    for line in container.logs(stdout=stdout, stderr=stderr, stream=True):
        line_action(line)

class SandboxedPlayer(AbstractPlayer):
    def __init__(self, socket_file, working_dir, docker_client, local_dir=None, s3_bucket=None, s3_key=None,
                 player_key="", player_mem_limit=256, player_cpu=20):

        super().__init__(socket_file, working_dir, local_dir, s3_bucket, s3_key, player_key, player_mem_limit, player_cpu)
        self.docker = docker_client

    def stream_logs(self, stdout=True, stderr=True, line_action=lambda line: print(line.decode())):
        threading.Thread(target=_stream_logs, args=(self.container, stdout, stderr, line_action)).start()

    def start(self):
        # won't collide ;)
        self.socket_name = '/tmp/battlecode-suspender-{}'.format(random.randint(0, 10**50))
        self.suspender_socket = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.suspender_socket.bind(self.socket_name)
        self.suspender_socket.settimeout(server.BUILD_TIMEOUT) # seconds
        self.suspender_socket.listen(1)

        volumes = {
            self.working_dir: {'bind': '/code', 'mode': 'rw'},
            self.socket_file: {'bind': '/tmp/battlecode-socket', 'mode': 'rw'},
            self.socket_name: {'bind': '/tmp/battlecode-suspender', 'mode': 'rw'}
        }

        working_dir = '/'
        command = 'sh /player_startup.sh'
        env = {
               'PLAYER_KEY': self.player_key,
               'SOCKET_FILE': '/tmp/battlecode-socket',
               'RUST_BACKTRACE': 1,
               'BC_PLATFORM': self._detect_platform()
       }

        self.container = self.docker.containers.run(
            'battlebaby',
            command,
            privileged=False,
            detach=True,
            stdout=True,
            stderr=True,
            volumes=volumes,
            working_dir=working_dir,
            environment=env,
            mem_limit=self.player_mem_limit,
            memswap_limit=self.player_mem_limit,
            auto_remove = True,
            network_disabled=True
        )
        self.suspender_connection = None

    def guess_language(self):
        procs = self.container.top()['Processes']
        for p in procs:
            name = p[3]
            if "java" in name:
                return "jvm"
            elif "python" in name:
                return "python"
            elif "pypy" in name:
                return "pypy"
            elif "mono" in name:
                return "mono"
        return "c"

    def suspinit(self):
        if self.suspender_connection == None:
            try:
                # wait for suspender script to connect from player host
                connection, _ = self.suspender_socket.accept()
                self.suspender_connection = connection
                self.suspender_file = self.suspender_connection.makefile('rw', 64)
                login = next(self.suspender_file)
                assert int(login.strip()) == self.player_key, 'mismatched suspension login: {} != {}'.format(repr(login.strip()), repr(self.player_key))
            except Exception as e:
                print('suspender timed out', e)

    def pause(self):
        self.suspinit()
        # see suspender.py
        # we don't go through docker.suspend or docker.exec because they're too slow (100ms)
        try:
            self.suspender_file.write('suspend\n')
            self.suspender_file.flush()
            response = next(self.suspender_file)
            assert response.strip() == 'ack', response.strip() + ' != ack'
        except Exception as e:
            print("SUSPENSION FAILED!!! SUSPICIOUS:", e)

    def unpause(self, timeout=None):
        self.suspinit()
        # see suspender.py
        # we don't go through docker.suspend or docker.exec because they're too slow (100ms)
        try:
            self.suspender_file.write('resume\n')
            self.suspender_file.flush()
            response = next(self.suspender_file)
            assert response.strip() == 'ack', response.strip() + ' != ack'
        except Exception as e:
            print("resumption failed:", e)

    def destroy(self):
        try:
            self.container.remove(force=True)
        except Exception as e:
            pass

        try:
            self.suspender_socket.close()
        except Exception as e:
            print('suspender close err:', e)
        super().destroy()

    def docker_stats(self, stream=False):
        return self.container.stats(decode=True, stream=stream)

    def __del__(self):
        self.destroy()
