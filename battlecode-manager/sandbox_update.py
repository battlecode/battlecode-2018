import docker
import gzip

docker_client = docker.from_env()

img = docker_client.images.pull('gcr.io/battlecode18/sandbox',tag='latest')
resp = img.save()

with gzip.open('sandbox.tar.gz','wb') as f:
    f.write(resp.data)

storage_client = storage.Client()
bucket = storage_client.get_bucket(bucket_name)
blob = bucket.blob('sandbox.tar.gz')
blob.upload_from_filename('sandbox.tar.gz')
