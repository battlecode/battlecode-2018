import boto3
import docker
import gzip

docker_client = docker.from_env()

img = docker_client.images.pull('gcr.io/battlecode18/sandbox',tag='latest')
resp = img.save()

with gzip.open('sandbox.tar.gz','wb') as f:
    f.write(resp.data)

s3 = boto3.resource('s3')
