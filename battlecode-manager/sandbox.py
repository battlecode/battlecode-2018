from pathlib import Path
from threading import Timer
import threading
from tqdm import tqdm
import os, time, socket, fcntl, struct, string, random, io, zipfile
from shutil import copytree
import psutil, subprocess
try:
    import boto3, docker
except:
    pass

def delete_folder(path):
    try:
        for sub in path.iterdir():
            if sub.is_dir():
                delete_folder(sub)
            else:
                sub.unlink()
        path.rmdir()
    except Exception as e:
        pass

def random_key(length):
    return ''.join([random.choice(string.ascii_letters + string.digits) for _ in range(length)])

class NoSandbox:
    def __init__(self, socket_file, local_dir=None, s3_bucket=None, s3_key=None,
                player_key="", working_dir="working_dir/",
                player_mem_limit=256, player_cpu=20):
        self.player_mem_limit = str(player_mem_limit)+'mb'
        self.player_key = player_key
        self.socket_file = socket_file

        working_dir = os.path.abspath(working_dir);

        bcdir = os.path.join(working_dir, 'battlecode')
        # todo: copy battlecode to working_dir
        if not os.path.exists(bcdir):
            if not os.path.exists(working_dir):
                print("Creating working directory at", working_dir)
                os.makedirs(working_dir)
            prepath = os.path.abspath('../battlecode')
            print("Copying battlecode resources from {} to {}".format(prepath, working_dir))
            copytree(prepath, bcdir)
            print("Working dir ready!")

        self.working_dir = os.path.join(working_dir, random_key(20))

        if s3_bucket:
            raise Exception("Cannot run code from s3 without sandbox, dummy!")
        elif local_dir:
            print("Copying files from {} to {}".format(os.path.abspath(local_dir), self.working_dir))
            copytree(os.path.abspath(local_dir), self.working_dir)
        else:
            raise ValueError("Must provide either S3 key and bucket or local directory for code.")
            return

        self.paused = False
        self.streaming = False

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
        # TODO: windows chec
        args = ['sh', os.path.join(self.working_dir, 'run.sh')]
        env = {'PLAYER_KEY': str(self.player_key), 'SOCKET_FILE': self.socket_file, 'RUST_BACKTRACE': '1',
            'PYTHONPATH': os.environ['PYTHONPATH']}
        cwd = self.working_dir
        self.process = psutil.Popen(args, env=env, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    def pause(self):
        self.paused = True
        self.process.suspend()

    def unpause(self,timeout=None):
        if self.paused:
            self.process.resume()
            Timer(timeout, self.pause).start()
        else:
            raise RuntimeError('You attempted to unpause a player that was not paused.')

    def destroy(self):
        if hasattr(self, 'process'):
            if self.process.is_running():
                self.process.kill()
        delete_folder(self.working_dir)

    def __del__(self):
        self.destroy()

if 'NODOCKER' not in os.environ:
    def _stream_logs(container, stdout, stderr, line_action):
        for line in container.logs(stdout=stdout, stderr=stderr, stream=True):
            line_action(line)

    docker_client = docker.from_env()

    class Sandbox:

        def dos2unix(self):
            pathlist = Path(str(self.working_dir.absolute())).glob("**/*.py")
            for path in pathlist:
                with open(str(path),'r') as f:
                    x = f.read()
                with open(str(path),'w') as f:
                    f.write(x.replace('\r\n', '\n'))

            pathlist = Path(str(self.working_dir.absolute())).glob("**/*.sh")
            for path in pathlist:
                with open(str(path),'r') as f:
                    x = f.read()
                with open(str(path),'w') as f:
                    f.write(x.replace('\r\n', '\n'))

        def __init__(self, socket_file, local_dir=None, s3_bucket=None, s3_key=None,
                    player_key="", working_dir="working_dir/",
                    player_mem_limit=256, player_cpu=20):
            self.player_mem_limit = str(player_mem_limit)+'mb'
            self.player_key = player_key
            self.docker = docker_client
            self.socket_file = socket_file
            if working_dir[-1] != "/":
                working_dir += "/"

            self.working_dir = Path(working_dir + random_key(20) + "/")
            self.working_dir.mkdir(parents=True,exist_ok=True)

            if s3_bucket:
                self.extract_code(s3_bucket, s3_key)
            elif local_dir:
                copytree(local_dir, str(self.working_dir.absolute()))
            else:
                raise ValueError("Must provide either S3 key and bucket or local directory for code.")
                return

            self.dos2unix()


        def stream_logs(self, stdout=True, stderr=True, line_action=lambda line: print(line.decode())):
            threading.Thread(target=_stream_logs, args=(self.container, stdout, stderr, line_action)).start()

        def extract_code(self, bucket, key):
            obj = bucket.Object(key)
            with io.BytesIO(obj.get()["Body"].read()) as tf:
                tf.seek(0)
                with zipfile.ZipFile(tf, mode='r') as zipf:
                    zipf.extractall(path=str(self.working_dir.absolute()))

        def start(self):
            volumes = {str(self.working_dir.absolute()):{'bind':'/code','mode':'rw'},self.socket_file:{'bind':'/tmp/battlecode-socket','mode':'rw'}}

            working_dir = '/code'
            command = 'sh run.sh'
            env = {'PLAYER_KEY':self.player_key,'SOCKET_FILE':'/tmp/battlecode-socket','RUST_BACKTRACE':1}

            self.container = self.docker.containers.run('battlebaby', command,
                    privileged=False, detach=True, stdout=True, stderr=True,
                    volumes=volumes, working_dir=working_dir, environment=env,
                    mem_limit=self.player_mem_limit,memswap_limit=self.player_mem_limit,
                    network_disabled=True)

        def pause(self):
            if self.container.status == 'running':
                self.container.pause()
            else:
                raise RuntimeError('You attempted to pause a non-running container.')

        def unpause(self,timeout=None):
            if self.container.status == 'paused':
                self.container.unpause()
                Timer(timeout, self.pause).start()
            else:
                raise RuntimeError('You attempted to unpause a container that was not paused.')

        def destroy(self):
            try:
                self.container.remove(force=True)
            except Exception as e:
                pass

            delete_folder(self.working_dir)

        def docker_stats(self, stream=False):
            return self.container.stats(decode=True, stream=stream)

        def __del__(self):
            self.destroy()

else:
    class Sandbox:
        def __init__(self, *args, **kwargs):
            raise Exception("No sandboxing in NODOCKER mode")
