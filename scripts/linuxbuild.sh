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
step docker build -t linuxbuild -f scripts/LinuxBuildDockerfile .

# run build in the image
tput setaf 5
echo == Starting build in docker image ==
tput sgr0

step mkdir -p .cache

BINDS="
-v $PWD:/battlecode 
-v $PWD/.cache/cargo-linux-registry:/root/.cargo/registry
-v $PWD/.cache/cargo-linux-git:/root/.cargo/git
"

echo binds: $BINDS

ID=$(docker create --rm $BINDS linuxbuild sh -c '
export TERM=xterm-256color
. ~/.cargo/env
cd /battlecode

step() {
    tput setaf 2
    echo linuxbuild:/battlecode$ $@
    tput sgr0
    $@
}

step export CARGO_TARGET_DIR=$PWD/.cache/target-linux

step make clean
step make release

tput setaf 2
echo == Moving results to docker-artifacts/battlecode-linux ==
tput sgr0

step mkdir -p docker-artifacts
step "rm -rf docker-artifacts/battlecode-linux || true"
step cp -R battlecode docker-artifacts/battlecode-linux
')
step docker start $ID -a -i

tput setaf 5
echo "Created docker-artifacts/battlecode-linux."
tput sgr0
