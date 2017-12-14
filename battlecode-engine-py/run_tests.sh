#!/bin/bash
set -e

pushd $(dirname $0) >/dev/null

echo -e "\n--\033[35m Running python tests \033[0m--\n"
echo -e '\033[33m$ python setup.py build_ext --inplace \033[0m'
python3 setup.py build_ext --inplace

echo -e '\n\033[33m$ mypy battlecode test \033[0m'
mypy battlecode test
echo -e '\n\033[33m$ nosetests \033[0m'
nosetests
popd >/dev/null