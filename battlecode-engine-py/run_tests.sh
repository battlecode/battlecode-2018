#!/bin/sh
set -e

pushd $(dirname $0) >/dev/null

echo 'Running python tests...'
echo ''
echo 'python3 setup.py build_ext --inplace'
python3 setup.py build_ext --inplace

echo 'mypy battlecode test'
mypy battlecode test
echo 'nosetests'
nosetests
popd >/dev/null

echo 'done'