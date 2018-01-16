#!/bin/bash 
# Some slightly wacky stuff that allows us to cache the build artifacts used by the Linux dockerfile build.
# Essentially, instead of running the last few build steps in a Dockerfile, we run them in a docker image, then use "docker checkpoint" to export that image.
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
# note: we run in the above directory
cd $DIR/..

step() {
    tput setaf 5
    echo $ $@
    tput sgr0
    $@
}

# build the initial image, just containing deps
step docker build -t battlebaby -f scripts/SandboxDockerfile .

# run build in the image
tput setaf 5
echo == Starting build in docker image ==
tput sgr0

step mkdir -p .cache

BINDS="
-v $PWD:/battlecode_src
-v $PWD/.cache/cargo-sandbox-registry:/root/.cargo/registry
-v $PWD/.cache/cargo-sandbox-git:/root/.cargo/git
"

tput setaf 5
echo $ docker create $BINDS battlebaby sh -c '/*build script*/'
tput sgr0

ID=$(docker create $BINDS battlebaby sh -c '
set -e
cd /battlecode_src

step() {
    echo battlebaby:/battlecode_src$ $@
    $@
}

step export CARGO_TARGET_DIR=$PWD/.cache/target-sandbox

step make clean
step make release
step apk del rust cargo swig .pypy-rundeps --purge

echo == Moving results into place ==

step mkdir -p docker-artifacts/
step "rm -rf docker-artifacts/linux-battlecode-musl || true"
step cp -R battlecode docker-artifacts/linux-battlecode-musl
step mv /battlecode_src/battlecode /battlecode

# backwards compatibility w/ original image layout
step ln -s -T /battlecode/c /battlecode-c
step ln -s -T /battlecode/c/lib/libbattlecode-linux.a /battlecode-c/lib/libbattlecode.a
step ln -s -T /battlecode/java /battlecode-java

# needed to keep java from trying to compile these
step rm /battlecode/java/bc/*.java

step cp scripts/player_startup.sh /player_startup.sh
step cp scripts/suspender.py /suspender.py
')
step docker start $ID -a -i

tput setaf 5
echo $ docker commit -a "Teh Devs battlecode@mit.edu" -m "Final build step" -c "ENV PYTHONPATH=/battlecode/python" -c 'CMD ["/bin/ash"]' $ID battlebaby-fat
tput sgr0
docker commit -a "Teh Devs battlecode@mit.edu" -m "Final build step" -c 'CMD ["/bin/ash"]' $ID battlebaby-fat
# cmd is just there for debugging, we don't use it in prod
step "docker-squash --version || echo 'please pip3 install docker-squash' && exit 1"

step docker-squash battlebaby-fat -t battlebaby

step mkdir -p docker-artifacts
step docker save battlebaby -o docker-artifacts/battlebaby.tar
