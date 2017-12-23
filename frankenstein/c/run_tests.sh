#!/bin/sh

set -e

pushd $(dirname $0) >/dev/null

echo -e "\n--\033[32m Running c tests \033[0m--\n"
echo -e '\033[33m$ make \033[0m'
python setup.py build_ext --inplace
echo

popd >/dev/null
make