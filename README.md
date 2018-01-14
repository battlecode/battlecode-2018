# Battlecode 2018 [![Build Status](https://travis-ci.com/battlecode/battlecode-2018.svg?token=xnYzex76nLR8psy8sjqJ&branch=master)](https://travis-ci.com/battlecode/battlecode-2018)[![Coverage](https://coveralls.io/repos/github/battlecode/battlecode-2018/badge.svg?t=3E3KU7)](https://coveralls.io/github/battlecode/battlecode-2018)
Everything you need to run Battlecode 2018, aside from thousands of hours of spare time and a reasonable amount of cash.

## Layout
- battlecode-engine: The core engine component.
- battlecode-manager: What runs matches, either as a CLI or in *the cloud*
- bindings: An elegant monstrosity, more details in `/bindings/README.md`
- battlecode-sandbox: The environment in which player code is executed.
- battlecode-maps: Map files for battlecode 2018
- examplefuncsplayer-*: examplefuncsplayers.
- examplefuncsplayer-*-old: examplefuncsplayers from initial release; kept around so we can monitor backwards compatibility.
- all the junk in this folder: it's necessary, I promise

## Building
You'll need:
- Rust and cargo 1.22.0
- Java 8
- SWIG 3.0
- docker
- python 3
    - cffi
    - eel
    - tqdm
    - werkzeug
    - ujson
    - psutil
    - docker
    - boto3
- gcc

To build the engine and bindings, run `make`. (You should also run `make clean` as a first step if you get strange errors.)
You can then run `./start_nodocker.sh` (or `battlecode.sh`) to run without docker.
You can also use `make release` to build in release mode (slower build, faster execution).

On windows, use `build.bat`, `build-release.bat`, and `start_nodocker.bat` / `battlecode.bat`. You'll need the same dependencies, but Visual Studio CE 2017 instead of gcc.

## Releasing
Clone the bc18-scaffold repo into this repo.
On windows, run `build-release.bat`, `copy-artifacts.bat`, then cd into `bc18-scaffold`, create a new branch with the name of the release, commit the new binaries (lol) and push it.
On mac os x, edit `make-release.sh` with the name of the release, run `./make-release.sh`, and follow the instructions.
