from pathlib import Path
import os
import string
import random
import io
import zipfile
from shutil import copytree
import sys


# TODO Any reason why shutil.rmtree is not used for this?
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


def extract_s3_bucket(bucket, key, destination_directory):
    obj = bucket.Object(key)
    with io.BytesIO(obj.get()["Body"].read()) as tf:
        tf.seek(0)
        with zipfile.ZipFile(tf, mode='r') as zipf:
            zipf.extractall(path=destination_directory)


def dos2unix(directory):
    ''' Converts all .py and .sh files in the given directory (recursively) to use unix line endings '''
    pathlist = list(Path(directory).glob("**/*.py"))
    pathlist += list(Path(directory).glob("**/*.sh"))

    for path in pathlist:
        with open(str(path), 'r') as f:
            x = f.read()
        with open(str(path), 'w') as f:
            f.write(x.replace('\r\n', '\n'))


class AbstractPlayer:
    def __init__(self, socket_file, working_dir, local_dir, s3_bucket, s3_key,
                 player_key, player_mem_limit, player_cpu):
        self.player_mem_limit = str(player_mem_limit) + 'mb'
        self.player_key = player_key
        self.socket_file = socket_file
        if working_dir[-1] != "/":
            working_dir += "/"

        self.working_dir = Path(working_dir + random_key(20) + "/")

        if not os.path.exists(working_dir):
            os.makedirs(working_dir)

        if s3_bucket:
            extract_s3_bucket(s3_bucket, s3_key, self.working_dir.absolute())
        elif local_dir:
            # print("Copying files from {} to {}".format(os.path.abspath(local_dir), self.working_dir))
            copytree(os.path.abspath(local_dir), str(self.working_dir.absolute()))
        else:
            raise ValueError("Must provide either S3 key and bucket or local directory for code.")

        dos2unix(str(self.working_dir.absolute()))

    def _detect_platform(self):
        if sys.platform.startswith("linux"):
            return "LINUX"
        elif sys.platform == 'win32':
            return "WIN32"
        elif sys.platform == 'darwin':
            return 'DARWIN'
        else:
            raise Exception("Unknown os: " + sys.platform)

    def start(self):
        pass

    def stream_logs(self, stdout=True, stderr=True, line_action=lambda line: print(line.decode())):
        pass

    def pause(self):
        pass

    def unpause(self, timeout=None):
        pass

    def destroy(self):
        delete_folder(self.working_dir)

    def __del__(self):
        self.destroy()
