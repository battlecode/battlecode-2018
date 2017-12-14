#!/bin/sh
set -e

pushd $(dirname $0) >/dev/null

echo 'Running rust tests...'
echo ''
echo 'cargo test'
cargo test

battlecode-engine-py/run_tests.sh

popd >/dev/null
