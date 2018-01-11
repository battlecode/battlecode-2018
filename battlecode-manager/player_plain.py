import os
import psutil
import subprocess
import threading
import sys
from threading import Timer
import select

from player_abstract import AbstractPlayer


class PlainPlayer(AbstractPlayer):
    def __init__(self, socket_file, working_dir, local_dir=None,
                 player_key="", player_mem_limit=256, player_cpu=20):

        super().__init__(socket_file, working_dir, local_dir, None, None, player_key, player_mem_limit, player_cpu)

        self.paused = False
        self.streaming = False
        self.process = None

    def stream_logs(self, stdout=True, stderr=True, line_action=lambda line: print(line.decode())):
        assert not self.streaming
        self.streaming = True
        if stdout:
            threading.Thread(target=self._stream_logs, args=(self.process.stdout, line_action)).start()
        if stderr:
            threading.Thread(target=self._stream_logs, args=(self.process.stderr, line_action)).start()

    def _stream_logs(self, stream, line_action):
        for line in stream:
            line_action(line)

    def start(self):
        if sys.platform == 'win32':
            args = [os.path.join(self.working_dir, 'run.bat')]
            # things break otherwise
            env = dict(os.environ)
        else:
            args = ['sh', os.path.join(self.working_dir, 'run.sh')]
            # Path needs to be passed through, otherwise some compilers (e.g gcc) can get confused and not find things
            env = {'PATH': os.environ['PATH']}

        env['PLAYER_KEY'] = str(self.player_key)
        env['RUST_BACKTRACE'] = '1'
        env['BC_PLATFORM'] = self._detect_platform()

        if isinstance(self.socket_file, tuple):
            # tcp port
            env['TCP_PORT'] = str(self.socket_file[1])
        else:
            env['SOCKET_FILE'] = self.socket_file

        cwd = self.working_dir
        self.process = psutil.Popen(args, env=env, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=-1)

    def pause(self):
        print("Pausing")
        self.paused = True
        self.process.suspend()

    def unpause(self, timeout=None):
        if self.paused:
            self.process.resume()
            Timer(timeout, self.pause).start()
        else:
            raise RuntimeError('You attempted to unpause a player that was not paused.')

    def destroy(self):
        if self.process is not None:
            self.process.kill()
            self.process = None
        super().destroy()
