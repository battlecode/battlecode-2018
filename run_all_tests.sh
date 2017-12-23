#!/bin/bash

# Runs all the test by hand, in a shell.
# TODO: windows.

pushd $(dirname $0) >/dev/null

echo -e "\n--\033[32m Running rust tests \033[0m--\n"
echo -e '\033[33m$ cargo test \033[0m'
cargo test
RUST=$?
echo

echo
echo
echo '---------------'

if [ $RUST -ne 0 ]; then
    echo -e '\033[31mTests failed!\033[0m'
    exit 1
else
    echo
    echo -e '\033[32mAll tests passed.\033[0m'
fi

popd >/dev/null
