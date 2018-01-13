#!/bin/bash

docker stop $(docker ps -q)
docker container rm $(docker container ls -aq)
docker volume rm $(docker volume ls -q)$
docker volume prune

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

docker run -it --privileged -p 16147:16147 -p 6147:6147 -v $DIR:/player --rm battlecode/battlecode-2018
