#!/bin/bash 
# Some slightly wacky stuff that allows us to cache the build artifacts used by the Linux dockerfile build.
# Essentially, instead of running the last few build steps in a Dockerfile, we run them in a docker image, then use "docker checkpoint" to export that image.

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
# note: we run in the above directory
cd $DIR/..

set -e

step() {
    tput setaf 5
    echo $ $@
    tput sgr0
    $@
}

# build the initial image, just containing deps
step docker build -t battledaddy -f scripts/ManagerDockerfile .

# run build in the image
tput setaf 5
echo == Starting build in docker image ==
tput sgr0

step mkdir -p .cache

BINDS="
-v $PWD:/battlecode_src
"

tput setaf 5
echo $ docker create --privileged $BINDS battledaddy sh -c '/*build script*/'
tput sgr0

ID=$(docker create --privileged $BINDS battledaddy sh -c '
set -e

cd /battlecode_src
set -e

step() {
    echo battledaddy:/battlecode_src$ $@
    $@
}
step mkdir -p /battlecode
step cp -r ./docker-artifacts/linux-battlecode-musl /battlecode/battlecode
# this is necessary because we dont need to copy this file into the target dir & it breaks shutil.copytree
rm /battlecode/battlecode/c/lib/libbattlecode.a 2&>/dev/null || true
step cp -r ./battlecode-manager /battlecode/battlecode-manager
step cp -r ./battlecode-maps /battlecode/battlecode-maps
step mkdir -p /images
step cp -r ./docker-artifacts/battlebaby.tar /images
step cp ./scripts/manager_startup.sh /manager_startup.sh
')
echo "== ID: $ID =="
step docker start $ID -a -i

tput setaf 5
echo $ docker commit -a "Teh Devs battlecode@mit.edu" $ID -m "Final build step" -c 'CMD ["sh", "/manager_startup.sh"]' $ID battledaddy
tput sgr0

docker commit -a "Teh Devs battlecode@mit.edu" -m "Final build step" -c 'CMD ["sh", "/manager_startup.sh"]' $ID battledaddy
