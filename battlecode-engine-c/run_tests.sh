#!/bin/bash

set -e

pushd $(dirname $0) >/dev/null
echo -e "--\033[34m Running c tests \033[0m--\n"

echo -e '\n\033[33m$ make -C tests \033[0m'
make -C ctests

popd >/dev/null

echo 'done'