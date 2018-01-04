#!/bin/bash

# Runs all the test by hand, in a shell.
# TODO: windows.

pushd $(dirname $0) >/dev/null

echo -e "\n--\033[32m Running rust tests \033[0m--\n"
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
echo
echo '---------------'

if [ $RUST -ne 0 ]; then
    echo -e '\033[31mRust failed!\033[0m'
fi
if [ $C -ne 0 ]; then
    echo -e '\033[31mC failed!\033[0m'
fi
if [ $PYTHON -ne 0 ]; then
    echo -e '\033[31mPython failed!\033[0m'
fi

if [ $RUST -ne 0 -o $C -ne 0 -o $PYTHON -ne 0 ]; then
    echo -e '\033[31mTests failed!\033[0m'
    exit 1
fi

python3 ./battlecode-manager/test.py
SERVER=$?
if [ $SERVER -ne 0 ]; then
    echo -e '\033[31mPython failed!\033[0m'
else
    echo
    echo -e '\033[32mAll tests passed.\033[0m'
fi

popd >/dev/null
