from pathlib import Path
import os
import string
import random
import io
import zipfile
from shutil import copytree, rmtree
import sys


def random_key(length):
    return ''.join([random.choice(string.ascii_letters + string.digits) for _ in range(length)])


def extract_s3_bucket(bucket, key, destination_directory):
    obj = bucket.Object(key)
    try:
        with io.BytesIO(obj.get()["Body"].read()) as tf:
            tf.seek(0)
            with zipfile.ZipFile(tf, mode='r') as zipf:
                zipf.extractall(path=destination_directory)
        return True
    except Exception as e:
        print("Invalid s3 key.")
        return False


def dos2unix(directory):
    ''' Converts all .py and .sh files in the given directory (recursively) to use unix line endings '''
    pathlist = list(Path(directory).glob("**/*.py"))
    pathlist += list(Path(directory).glob("**/*.sh"))

    for path in pathlist:
        try:
            with open(str(path), 'r') as f:
                x = f.read()
        except UnicodeDecodeError as e:
            try:
                print("Trying Latin encoding...")
                with open(str(path), 'r', encoding='ISO-8859-1') as f:
                    x = f.read()
            except Exception as e2:
                print(e2)
                x = 'echo "Unable to read file (please encode as unicode)."'

        with open(str(path), 'w') as f:
            f.write(x.replace('\r\n', '\n'))


class AbstractPlayer:
    def __init__(self, socket_file, working_dir, local_dir, s3_bucket, s3_key,
                 player_key, player_mem_limit, player_cpu):
        self.player_mem_limit = str(player_mem_limit) + 'mb'
        self.player_cpu_fraction = player_cpu/100.0
        self.player_key = player_key
        self.socket_file = socket_file

        # Note that working dir is an absolute path
        self.working_dir = os.path.abspath(os.path.join(working_dir, random_key(20)))

        if not os.path.exists(working_dir):
            os.makedirs(working_dir)

        if s3_bucket:
            if not extract_s3_bucket(s3_bucket, s3_key, self.working_dir):
                raise ValueError("Incorrect s3 key provided.")
        elif local_dir:
            # print("Copying files from {} to {}".format(os.path.abspath(local_dir), self.working_dir))
            try:
                copytree(os.path.abspath(local_dir), self.working_dir)
            except Exception as e:
                print("Failed to copy files from {} to {}\nMake sure you don't have any broken symlinks in your player directory for example" +
                      " and that your dog didn't eat your hard drive.".format(os.path.abspath(local_dir), self.working_dir))
                raise
        else:
            raise ValueError("Must provide either S3 key and bucket or local directory for code.")

        dos2unix(self.working_dir)

    def _detect_platform(self):
        if sys.platform.startswith("linux"):
            return "LINUX"
        elif sys.platform == 'win32':
            return "WIN32"
        elif sys.platform == 'darwin':
            return 'DARWIN'
        else:
            raise Exception("Unknown os: " + sys.platform)

    def guess_language(self):
        return "?"

    def start(self):
        pass

    def stream_logs(self, stdout=True, stderr=True, line_action=lambda line: print(line.decode())):
        pass

    def pause(self):
        pass

    def unpause(self, timeout=None):
        pass

    def destroy(self):
        # Need to check for existance because this method may be called multiple times (e.g from __del__)
        if os.path.isdir(self.working_dir):
            rmtree(self.working_dir)

    def __del__(self):
        self.destroy()
