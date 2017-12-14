#!/bin/bash
set -e

pushd $(dirname $0) >/dev/null

echo -e "--\033[35m Running python tests \033[0m--\n"
echo ''
echo -e '\033[33m$ python setup.py build_ext --inplace \033[0m'
echo 'python3 setup.py build_ext --inplace'
python3 setup.py build_ext --inplace

echo -e '\033[33m$ mypy battlecode test \033[0m'
echo 'mypy battlecode test'
mypy battlecode test
echo -e '\033[33m$ nosetests \033[0m'
echo 'nosetests'
nosetests
popd >/dev/null

echo 'done'