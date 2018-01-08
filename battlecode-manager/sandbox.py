from pathlib import Path
from threading import Timer
from tqdm import tqdm
import os, time, socket, fcntl, struct, string, random, io, zipfile, boto3, docker
from distutils.dir_util import copy_tree

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

class Sandbox:
    def initialize():
        global docker_client
        docker_client = docker.from_env()

    def __init__(self, socket_file, local_dir=None, s3_bucket=None, s3_key=None, player_key="", working_dir="working_dir/"):
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
            copy_tree(local_dir, str(self.working_dir.absolute()))
        else:
            raise ValueError("Must provide either S3 key and bucket or local directory for code.")

    def extract_code(self, bucket, key):
        obj = bucket.Object(key)
        with io.BytesIO(obj.get()["Body"].read()) as tf:
            tf.seek(0)
            with zipfile.ZipFile(tf, mode='r') as zipf:
                zipf.extractall(path=str(self.working_dir.absolute()))

    def start(self):
        volumes = {str(self.working_dir.absolute()):{'bind':'/code','mode':'ro'},self.socket_file:{'bind':'/tmp/battlecode-socket'}}

        working_dir = '/code'
        command = 'sh run.sh'
        env = {'PLAYER_KEY':self.player_key,'SOCKET_FILE':self.socket_file}

        self.container = self.docker.containers.run('gcr.io/battlecode18/sandbox',command,privileged=False,detach=True,cpu_percent=int(os.environ['PLAYER_CPU_PERCENT']),mem_limit=os.environ['PLAYER_MEM_LIMIT'],memswap_limit=os.environ['PLAYER_MEM_LIMIT'],stdout=True,stderr=True,volumes=volumes,working_dir=working_dir,environment=env)

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

    def get_logs(self, stdout=True, stderr=True, timestamps=True, stream=False):
        return self.container.logs(stdout=stdout,stderr=stderr,timestamps=timestamps,stream=stream)

    def destroy(self):
        logs = self.container.logs(stdout=True,stderr=True,timestamps=True,stream=False)
        try:
            self.container.remove(force=True)
        except Exception as e:
            pass

        delete_folder(self.working_dir)
        return logs

    def stats(self, stream=False):
        return self.container.stats(decode=True, stream=stream)

    def __del__(self):
        self.destroy()
