#!/usr/bin/env bash

set -e

# Build the move-stdlib bundle
pushd .
rm -rf MoveStdlib
git clone https://github.com/eigerco/move-stdlib.git MoveStdlib
cd MoveStdlib
smove bundle
popd

# Build the substrate-stdlib bundle
pushd .
rm -rf substrate-stdlib
git clone https://github.com/eigerco/substrate-stdlib.git
cd substrate-stdlib
smove bundle
popd
