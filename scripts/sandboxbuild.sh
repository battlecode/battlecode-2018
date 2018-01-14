#!/bin/bash 
# Some slightly wacky stuff that allows us to cache the build artifacts used by the Linux dockerfile build.
# Essentially, instead of running the last few build steps in a Dockerfile, we run them in a docker image, then use "docker checkpoint" to export that image.

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
step docker build -t Sandbox -f scripts/SandboxDockerfile .

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

echo binds: $BINDS

ID=$(docker create $BINDS Sandbox sh -c '
export TERM=xterm-256color
. ~/.cargo/env
cd /battlecode_src
set -e

step() {
    tput setaf 2
    echo sandboxbuild:/battlecode_src$ $@
    tput sgr0
    $@
}

step export CARGO_TARGET_DIR=$PWD/.cache/target-sandbox

step make clean
step make release
step apk del rust cargo swig .pypy-rundeps --purge

tput setaf 2
echo == Moving results into place ==
tput sgr0

step mv /battlecode_src/battlecode /battlecode

# backwards compatibility w/ original image layout
step ln -s -T /battlecode/c /battlecode-c
step ln -s -T /battlecode/c/lib/libbattlecode-linux.a /battlecode-c/lib/libbattlecode.a
step ln -s -T /battlecode/java /battlecode-java
')
step docker start $ID -a -i
step docker commit -a "Teh Devs battlecode@mit.edu" $ID -m "Final build step" -c "ENV PYTHONPATH=/battlecode/python" battlebaby-fat
