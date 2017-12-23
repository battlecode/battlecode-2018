#!/bin/sh

set -e

pushd $(dirname $0) >/dev/null

echo -e "\n--\033[32m Running python tests \033[0m--\n"
echo -e '\033[33m$ python setup.py build_ext --inplace \033[0m'
python setup.py build_ext --inplace
echo -e '\033[33m$ nosetests \033[0m'
nosetests
echo

popd >/dev/null