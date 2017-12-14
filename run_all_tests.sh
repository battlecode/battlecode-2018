#!/bin/bash

# Runs all the test by hand, in a shell.
# TODO: windows.

pushd $(dirname $0) >/dev/null

echo -e "--\033[32m Running rust tests \033[0m--\n"
echo -e '\033[33m$ cargo test \033[0m'
cargo test
RUST=$?
echo

battlecode-engine-c/run_tests.sh
C=$?
echo

battlecode-engine-py/run_tests.sh
PYTHON=$?
echo

if [ $RUST -ne 0 -o $C -ne 0 -o $PYTHON -ne 0 ]; then
    echo
    echo 'Tests failed!'
    exit 1
else
    echo
    echo 'All tests passed.'
fi

popd >/dev/null
