#!/bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd $DIR/..

green() {
    tput setaf 2
}
blue() {
    tput setaf 4
}
red() {
    tput setaf 1
}
magenta() {
    tput setaf 5
}
plain() {
    tput sgr0
}

set -e
step() {
    green
    echo $ $@
    plain
    $@
}
step_ignore() {
    green
    echo $ $@
    plain
    if $@; then
        true
    fi
}
prompt() {
    blue
    printf "$@ [y/n] "
    plain
    while true; do
        read good
        if [ "$good" = "y" ]; then
            magenta
            echo "Okay, continuing."
            plain
            break
        elif [ "$good" = "n" ]; then
            red
            echo "Bailing out."
            plain
            exit 1
        else
            echo "Huh?" $good
        fi
    done
}

RELEASE=0.11.0

green
echo "=== Starting release $(magenta)$RELEASE$(green) ==="
magenta
echo "Hope you know what you're doing"
plain
if [ ! -z "$(git status --porcelain | grep -v make-release.sh | grep -v web)" ]; then
    red
    echo "Oy, there are uncommitted files!"
    echo "Not continuing."
    exit 1
fi

if [ "$(uname -s)" != "Darwin" ]; then
    red
    echo "This script must be run on Mac, otherwise how do we get the mac artifacts?"
    echo "Not continuing."
    exit 1
fi

if [ "$(git rev-parse --abbrev-ref HEAD)" != "master" ]; then
    red
    echo "Git branch isn't master, it's: $(git rev-parse --abbrev-ref HEAD)"
    echo "Not continuing."
    exit 1
fi

gsed -i 's/Version .*/Version '"$RELEASE/" battlecode-manager/web/run.html
if [ -z "$(git status --porcelain | grep -v make-release.sh)" ]; then
    red
    echo "Version on web page didn't update?"
    prompt "Should I continue?"
fi

#step cd bc18-scaffold
#if git checkout $RELEASE; then
#    true
#else
#    red
#    echo "Couldn't checkout $RELEASE in bc18-scaffold, have you run the windows build yet?"
#    echo "Not continuing."
#    exit 1
#fi
#step cd ..
#
#step make clean
#step make release
#step mv battlecode battlecode-mac
#step make clean
#step make linux-libs
#step make clean
#step make docker-sandbox
step make docker-manager
#
#step mv battlecode-mac battlecode
#step cp -R docker-artifacts/battlecode-linux/* battlecode/
#step ls battlecode/c/lib
#step ls battlecode/java/bc/
#step ls battlecode/python/battlecode/darwin
#step ls battlecode/python/battlecode/linux
#
#prompt "files look good?"
#
#blue
#echo "Please wait for the following matches to finish."
#plain
#step ./battlecode.sh -p1 examplefuncsplayer-python -p2 examplefuncsplayer-java -m bananas
#step ./battlecode.sh -p1 examplefuncsplayer-c -p2 examplefuncsplayer-c -m bananas
#echo
#blue
#echo "Please run matches between examplefuncsplayer-python, examplefuncsplayer-java, examplefuncsplayer-c, then terminate the manager with Stop Manager."
#plain
#step_ignore ./run_nodocker.sh
#prompt "Did it work?"

blue
echo "Please run matches between examplefuncsplayer-python, examplefuncsplayer-python-old, examplefuncsplayer-java, examplefuncsplayer-java-old, examplefuncsplayer-c, examplefuncsplayer-c-old, then terminate the manager with Stop Manager."
plain
step_ignore docker run -it --privileged -p 16147:16147 -p 6147:6147 -v $PWD:/player --rm battledaddy
prompt "Did it work?"

step git add battlecode-manager/web/run.html
step git add .
green
echo $ git commit -m "Release $RELEASE"
plain
git commit -m "Release $RELEASE"
step git tag $RELEASE
step git push origin master
step git push --tags origin $RELEASE

step make package
step cd bc18-scaffold
step git checkout -- battlecode/c/lib/libbattlecode-win32.lib
step git checkout -- battlecode/python/battlecode/win32/_bc.pyd
step git add .
step git status
prompt "Everything look good?"

green
echo $ git commit -m "$RELEASE Mac/Linux"
plain
git commit -m "$RELEASE Mac/Linux"

step cd ..
step docker tag battledaddy battlecode/battlecode-2018:$RELEASE
step docker tag battledaddy battlecode/battlecode-2018:latest

prompt "Ready for final push? No going back."

red
echo "=== MAKING FINAL RELEASE ==="
plain
step cd bc18-scaffold
step git push origin $RELEASE
step git checkout master
step git merge $RELEASE
step git push origin master
step cd ..
step docker push battlecode/battlecode-2018:$RELEASE
step docker push battlecode/battlecode-2018:latest

magenta
echo "Congratulations, release $RELEASE is complete."
plain

