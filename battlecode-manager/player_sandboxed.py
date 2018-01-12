import threading
from threading import Timer

from player_abstract import AbstractPlayer


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
        volumes = {
            self.working_dir: {'bind': '/code', 'mode': 'rw'},
            self.socket_file: {'bind': '/tmp/battlecode-socket', 'mode': 'rw'}
        }

        working_dir = '/'
        command = 'sh player_start.sh'
        env = {'PLAYER_KEY': self.player_key, 'SOCKET_FILE': '/tmp/battlecode-socket', 'RUST_BACKTRACE': 1,
               'BC_PLATFORM': self._detect_platform()}

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

        #cap_drop=['chown, dac_override, fowner, fsetid, kill, setgid, setuid, setpcap, net_bind_service, net_raw, sys_chroot, mknod, audit_write, setfcap'],cpu_period=100000,cpu_quota=self.player_cpu_fraction*100000,


    def pause(self):
        # 6147 is the UID of player
        # you can't escape ;;;)
        self.container.exec('/bin/pkill -STOP -U 6147')
        # we have to use /bin/pkill from procps, not /usr/bin/pkill from busybox
        # because /usr/bin/pkill doesn't have -U

        # we don't use this because it's slow:
        # self.container.pause()

    def unpause(self, timeout=None):
        self.container.exec('/bin/pkill -CONT -U 6147')

        # we don't use this because it's slow:
        # self.container.unpause()

    def destroy(self):
        try:
            self.container.remove(force=True)
        except Exception as e:
            pass

        super().destroy()

    def docker_stats(self, stream=False):
        return self.container.stats(decode=True, stream=stream)

    def __del__(self):
        self.destroy()
