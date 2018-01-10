# Battlecode 2018 [![Build Status](https://travis-ci.com/battlecode/battlecode-2018.svg?token=xnYzex76nLR8psy8sjqJ&branch=master)](https://travis-ci.com/battlecode/battlecode-2018)[![Coverage](https://coveralls.io/repos/github/battlecode/battlecode-2018/badge.svg?t=3E3KU7)](https://coveralls.io/github/battlecode/battlecode-2018)
Everything you need to run Battlecode 2018, aside from thousands of hours of spare time and a reasonable amount of cash.

# Projects
- battlecode-engine: The core engine component.
- battlecode-manager: What runs matches, either as a CLI or in *the cloud*
- bindings: An elegant monstrosity, more details in `/bindings/README.md`
- battlecode-sandbox: The environment in which player code is executed.
- battlecode-maps: Map files for battlecode 2018

# Build dependencies
For nodocker mode:
Rust and cargo 1.22.0
Java 8
SWIG 3.0
python 3
    cffi eel tqdm werkzeug ujson psutil
gcc
